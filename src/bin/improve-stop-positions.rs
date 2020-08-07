use chrono::{DateTime, FixedOffset};
use log::info;
use std::path::PathBuf;
use structopt::StructOpt;
use tartare_tools::{improve_stop_positions, Result};
use transit_model::{ntfs, Model};

#[derive(Debug, StructOpt)]
#[structopt(
    name = "improve-stop-positions",
    about = "Improve stop positions with OpenStreetMap.",
    rename_all = "kebab-case"
)]
struct Opt {
    /// input directory.
    #[structopt(short, long, parse(from_os_str), default_value = ".")]
    input: PathBuf,

    /// osm pbf file.
    #[structopt(short, long, parse(from_os_str))]
    pbf: PathBuf,

    /// output directory
    #[structopt(short, long, parse(from_os_str))]
    output: PathBuf,

    // The min distance in meters to update the coordinates
    #[structopt(long, short = "d", default_value = "20")]
    min_distance: f64,

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
    info!("Launching improve-stop-positions.");

    let model = ntfs::read(opt.input)?;
    let mut collections = model.into_collections();
    improve_stop_positions::improve_with_pbf(&opt.pbf, &mut collections, opt.min_distance)?;
    let model = Model::new(collections)?;
    transit_model::ntfs::write(&model, opt.output, opt.current_datetime)?;

    Ok(())
}

fn main() {
    tartare_tools::runner::launch_run(run);
}
