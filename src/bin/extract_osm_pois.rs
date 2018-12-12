// Copyright 2017-2018 Kisio Digital and/or its affiliates.
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

#[macro_use]
extern crate log;

use osm_tools::Result;
use std::path::PathBuf;
use structopt::StructOpt;

#[derive(Debug, StructOpt)]
#[structopt(
    name = "extract_osm_pois",
    about = "Extract POIs from OSM."
)]
struct Opt {
    /// OSM PBF file.
    #[structopt(short = "i", long = "input", parse(from_os_str))]
    input: PathBuf,
    /// POI configuration.
    #[structopt(short = "j", long = "poi-config", parse(from_os_str))]
    poi_config: Option<PathBuf>,

    /// output poi file
    #[structopt(short = "o", long = "output", parse(from_os_str))]
    output: PathBuf,
}

fn run() -> Result<()> {
    info!("Launching extract_osm_pois.");

    let opt = Opt::from_args();

    println!("{:#?}", opt);

    Ok(())
}

fn main() {
    env_logger::init();
    if let Err(err) = run() {
        for cause in err.iter_chain() {
            eprintln!("{}", cause);
        }
        std::process::exit(1);
    }
}
