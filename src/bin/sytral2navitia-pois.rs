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
