use log::info;
use std::path::PathBuf;
use structopt::StructOpt;
use tartare_tools::{poi::merge::merge, Result};

#[derive(Debug, StructOpt)]
#[structopt(
    name = "merge-pois",
    about = "Merge NAvitia POI files.",
    rename_all = "kebab-case"
)]
struct Opt {
    /// Navitia POI files
    #[structopt(name = "INPUTS", required = true, min_values = 2, parse(from_os_str))]
    pois: Vec<PathBuf>,

    /// Output poi file.
    #[structopt(short, long, parse(from_os_str))]
    output: PathBuf,
}

fn run(opt: Opt) -> Result<()> {
    info!("Launching merge-pois.");
    let model = merge(&mut opt.pois.into_iter())?;
    model.save_to_path(opt.output)
}

fn main() {
    tartare_tools::runner::launch_run(run);
}
