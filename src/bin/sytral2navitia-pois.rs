use log::info;
use std::path::PathBuf;
use structopt::StructOpt;
use tartare_tools::{poi::sytral::extract_pois, Result};

/// Convert Sytral POIs to Navitia POIs
#[derive(Debug, StructOpt)]
#[structopt(name = "sytral2navitia-pois", rename_all = "kebab-case")]
struct Opt {
    /// Sytral POIs file
    #[structopt(short, long, parse(from_os_str))]
    input: PathBuf,

    /// Navitia POIs file.
    #[structopt(short, long, parse(from_os_str))]
    output: PathBuf,
}

fn run(opt: Opt) -> Result<()> {
    info!("Launching sytral2navitia-pois.");
    let poi_model = extract_pois(opt.input)?;
    poi_model.save_to_path(opt.output)
}

fn main() {
    tartare_tools::runner::launch_run(run);
}
