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

extern crate env_logger;
#[macro_use]
extern crate log;
extern crate navitia_model;
extern crate osm_tools;
extern crate structopt;

use std::path::PathBuf;
use structopt::StructOpt;

use navitia_model::{ntfs, Model, Result};

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
}

fn run() -> Result<()> {
    info!("Launching improve-stop-positions.");

    let opt = Opt::from_args();
    let model = ntfs::read(opt.input)?;
    let mut collections = model.into_collections();
    osm_tools::improve_stop_positions::improve_with_pbf(
        &opt.pbf,
        &mut collections,
        opt.min_distance,
    )?;
    let model = Model::new(collections)?;
    navitia_model::ntfs::write(&model, opt.output)?;

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
