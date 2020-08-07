mod transxchange;

use chrono::{DateTime, FixedOffset, NaiveDate};
use log::info;
use std::path::PathBuf;
use structopt::StructOpt;
use transit_model::{transfers::generates_transfers, Result};

#[derive(Debug, StructOpt)]
#[structopt(
    name = "transxchange2ntfs",
    about = "Convert a TransXChange to an NTFS."
)]
struct Opt {
    /// input directory or ZIP file containing TransXChange files
    /// the files must be UTF-8 encoded
    #[structopt(long, short, parse(from_os_str), default_value = ".")]
    input: PathBuf,

    /// input directory or ZIP file containing NaPTAN files
    /// the files must be UTF-8 encoded
    #[structopt(long, short, parse(from_os_str), default_value = ".")]
    naptan: PathBuf,

    /// output directory for the NTFS files
    #[structopt(short, long, parse(from_os_str))]
    output: PathBuf,

    /// source of bank holidays as a path to a JSON
    /// available on https://www.gov.uk/bank-holidays/northern-ireland.json,
    /// https://www.gov.uk/bank-holidays/scotland.json and https://www.gov.uk/bank-holidays/england-and-wales.json
    #[structopt(short, long, parse(from_os_str))]
    bank_holidays: Option<PathBuf>,

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

    /// limit the data in the future
    #[structopt(short, long, parse(try_from_str))]
    max_end_date: NaiveDate,

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
    info!("Launching transxchange2ntfs...");

    let model = transxchange::read(
        opt.input,
        opt.naptan,
        opt.bank_holidays,
        opt.config,
        opt.prefix,
        opt.max_end_date,
    )?;
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
