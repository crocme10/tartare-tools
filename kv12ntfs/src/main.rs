use chrono::{DateTime, FixedOffset};
use failure::bail;
use log::info;
use std::path::PathBuf;
use structopt::StructOpt;
use transit_model::{transfers::generates_transfers, Result};

mod kv1;

#[derive(Debug, StructOpt)]
#[structopt(name = "kv12ntfs", about = "Convert a KV1 to an NTFS.")]
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

    // The max distance in meters to compute the tranfer
    #[structopt(long, short = "d", default_value = transit_model::TRANSFER_MAX_DISTANCE)]
    max_distance: f64,

    // The walking speed in meters per second.
    // You may want to divide your initial speed by sqrt(2) to simulate Manhattan distances
    #[structopt(long, short = "s", default_value = transit_model::TRANSFER_WAKING_SPEED)]
    walking_speed: f64,

    // Waiting time at stop in second
    #[structopt(long, short = "t", default_value = transit_model::TRANSFER_WAITING_TIME)]
    waiting_time: u32,
}

fn run(opt: Opt) -> Result<()> {
    info!("Launching kv12ntfs...");

    let model = if opt.input.is_dir() {
        kv1::read(opt.input, opt.config, opt.prefix)?
    } else {
        bail!("Invalid input data: must be an existing directory");
    };

    let model = generates_transfers(
        model,
        opt.max_distance,
        opt.walking_speed,
        opt.waiting_time,
        None,
    )?;

    transit_model::ntfs::write(&model, opt.output, opt.current_datetime)?;
    Ok(())
}

fn main() {
    tartare_tools::runner::launch_run(run);
}
