use chrono::{DateTime, FixedOffset};
use failure::bail;
use log::info;
use std::path::PathBuf;
use structopt::StructOpt;
use transit_model::Result;

mod piv;

#[derive(Debug, StructOpt)]
#[structopt(name = "piv2ntfs", about = "Convert a PIV to an NTFS.")]
struct Opt {
    /// input directory.
    #[structopt(short, long, parse(from_os_str), default_value = ".")]
    input: PathBuf,

    /// output directory
    #[structopt(short, long, parse(from_os_str))]
    output: PathBuf,

    /// config file
    #[structopt(short, long, parse(from_os_str))]
    config: Option<PathBuf>,

    /// prefix
    #[structopt(short = "p", long = "prefix")]
    prefix: Option<String>,

    /// current datetime
    #[structopt(
        short = "x",
        long,
        parse(try_from_str),
        default_value = &transit_model::CURRENT_DATETIME
    )]
    current_datetime: DateTime<FixedOffset>,
}

fn run(opt: Opt) -> Result<()> {
    info!("Launching piv2ntfs...");

    let model = if opt.input.is_dir() {
        piv::read(opt.input, opt.config, opt.prefix)?
    } else {
        bail!("Invalid input data: must be an existing directory");
    };

    transit_model::ntfs::write(&model, opt.output, opt.current_datetime)?;
    Ok(())
}

fn main() {
    tartare_tools::runner::launch_run(run);
}
