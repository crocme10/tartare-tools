use chrono::{DateTime, FixedOffset};
use log::info;
use std::path::PathBuf;
use structopt::StructOpt;
use transit_model::{transfers::generates_transfers, Result};

mod netexidf;

#[derive(Debug, StructOpt)]
#[structopt(name = "netexidf2ntfs", about = "Convert a Netex IDF to an NTFS.")]
struct Opt {
    /// input directory containing Netex IDF files
    #[structopt(long, short, parse(from_os_str), default_value = ".")]
    input: PathBuf,

    /// output directory for the NTFS files
    #[structopt(short, long, parse(from_os_str))]
    output: PathBuf,

    /// config file
    #[structopt(short, long, parse(from_os_str))]
    config: Option<PathBuf>,

    /// prefix
    #[structopt(short, long)]
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
    info!("Launching netexidf2ntfs...");

    let model = netexidf::read(opt.input, opt.config, opt.prefix)?;
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
