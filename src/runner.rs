use crate::Result;
use slog::slog_o;
use slog::Drain;
use slog_async::OverflowStrategy;
use structopt::StructOpt;

fn init_logger() -> slog_scope::GlobalLoggerGuard {
    let decorator = slog_term::TermDecorator::new().stdout().build();
    let drain = slog_term::CompactFormat::new(decorator).build().fuse();
    let mut builder = slog_envlogger::LogBuilder::new(drain).filter(None, slog::FilterLevel::Info);
    if let Ok(s) = std::env::var("RUST_LOG") {
        builder = builder.parse(&s);
    }
    let drain = slog_async::Async::new(builder.build())
        .chan_size(256) // Double the default size
        .overflow_strategy(OverflowStrategy::Block)
        .build()
        .fuse();
    let logger = slog::Logger::root(drain, slog_o!());

    let scope_guard = slog_scope::set_global_logger(logger);
    slog_stdlog::init().unwrap();
    scope_guard
}

fn wrapper_launch_run<O, F>(run: F) -> Result<()>
where
    F: FnOnce(O) -> Result<()>,
    O: StructOpt,
{
    let _log_guard = init_logger();
    if let Err(err) = run(O::from_args()) {
        for cause in err.iter_chain() {
            eprintln!("{}", cause);
        }
        return Err(err);
    }

    Ok(())
}

pub fn launch_run<O, F>(run: F)
where
    F: FnOnce(O) -> Result<()>,
    O: StructOpt,
{
    // the destruction of the logger
    // This allows to not loose any messages
    if wrapper_launch_run(run).is_err() {
        std::process::exit(1);
    }
}
