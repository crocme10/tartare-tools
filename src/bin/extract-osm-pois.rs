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

use failure::ResultExt;
use log::info;
use osm_utils::poi::PoiConfig;
use std::path::PathBuf;
use structopt::StructOpt;
use tartare_tools::{poi::osm, Result};

/// Extract POIs from OSM.
#[derive(Debug, StructOpt)]
#[structopt(name = "extract_osm_pois", rename_all = "kebab-case")]
struct Opt {
    /// OSM PBF file.
    #[structopt(short, long, parse(from_os_str))]
    input: PathBuf,

    /// POIs configuration.
    #[structopt(short = "c", long, parse(from_os_str))]
    poi_config: Option<PathBuf>,

    /// Output poi file.
    #[structopt(short, long, parse(from_os_str))]
    output: PathBuf,
}

fn run(opt: Opt) -> Result<()> {
    info!("Launching extract_osm_pois.");
    let matcher = match opt.poi_config {
        None => PoiConfig::default(),
        Some(path) => {
            let r = std::fs::File::open(&path)
                .with_context(|_| format!("Error while opening configuration file {:?}", path))?;
            PoiConfig::from_reader(r)?
        }
    };

    let poi_model = osm::extract_pois(opt.input, matcher)?;
    poi_model.save_to_path(opt.output)
}

fn main() {
    tartare_tools::runner::launch_run(run);
}
