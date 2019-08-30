// Copyright 2017 Kisio Digital and/or its affiliates.
//
// This program is free software: you can redistribute it and/or
// modify it under the terms of the GNU General Public License as
// published by the Free Software Foundation, either version 3 of the
// License, or (at your option) any later version.
//
// This program is distributed in the hope that it will be useful, but
// WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the GNU
// General Public License for more details.
//
// You should have received a copy of the GNU General Public License
// along with this program.  If not, see
// <http://www.gnu.org/licenses/>.

use crate::Result;
use slog::slog_o;
use slog::Drain;
use structopt::StructOpt;

fn init_logger() -> (slog_scope::GlobalLoggerGuard, ()) {
    let decorator = slog_term::TermDecorator::new().stdout().build();
    let drain = slog_term::CompactFormat::new(decorator).build().fuse();
    let mut builder = slog_envlogger::LogBuilder::new(drain).filter(None, slog::FilterLevel::Info);
    if let Ok(s) = std::env::var("RUST_LOG") {
        builder = builder.parse(&s);
    }
    let drain = slog_async::Async::new(builder.build()).build().fuse();
    let logger = slog::Logger::root(drain, slog_o!());

    let scope_guard = slog_scope::set_global_logger(logger);
    let log_guard = slog_stdlog::init().unwrap();
    (scope_guard, log_guard)
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
