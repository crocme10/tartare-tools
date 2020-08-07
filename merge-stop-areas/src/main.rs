use chrono::{DateTime, FixedOffset};
use log::info;
use std::path::PathBuf;
use structopt::StructOpt;
use transit_model::Result;

mod merge_stop_areas;

#[derive(Debug, StructOpt)]
#[structopt(
    name = "merge-stop-areas",
    about = "Automatic and/or manual merge of ntfs stop areas."
)]
struct Opt {
    /// input directory.
    #[structopt(short = "i", long = "input", parse(from_os_str))]
    input: PathBuf,

    /// configuration csv rules path.
    #[structopt(short = "c", long = "config", parse(from_os_str))]
    rules: Vec<PathBuf>,

    /// maximum distance in meters used to merge stop areas
    #[structopt(short = "d", long = "distance")]
    automatic_max_distance: u32,

    /// output report file path
    #[structopt(short = "r", long = "report", parse(from_os_str))]
    report: PathBuf,

    /// output directory
    #[structopt(short = "o", long = "output", parse(from_os_str))]
    output: PathBuf,

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
    info!("Launching merge-stop-areas...");

    let model = transit_model::ntfs::read(opt.input)?;
    let new_model = merge_stop_areas::merge_stop_areas(
        model,
        opt.rules,
        opt.automatic_max_distance,
        opt.report,
    )?;

    transit_model::ntfs::write(&new_model, opt.output, opt.current_datetime)?;
    Ok(())
}

fn main() {
    tartare_tools::runner::launch_run(run);
}
