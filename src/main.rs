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
#[macro_use]
extern crate structopt;

use structopt::StructOpt;

use navitia_model::Result;

#[derive(Debug, StructOpt)]
#[structopt(
    name = "improve_ntfs_with_osm",
    about = "Improve stop positions with OpenStreetMap."
)]
struct Opt {}

fn run() -> Result<()> {
    info!("Launching improve_ntfs_with_osm.");

    let _ = Opt::from_args();

    println!("I'm not doing anything.");

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
