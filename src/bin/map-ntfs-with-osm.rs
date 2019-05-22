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

use chrono::NaiveDateTime;
use failure::bail;
use log::info;
use std::collections::HashMap;
use std::path::PathBuf;
use structopt::StructOpt;
use tartare_tools::{improve_stop_positions, Result};
use transit_model::ntfs;

#[derive(Debug, StructOpt)]
#[structopt(
    name = "map-ntfs-with-osm",
    about = "Map ntfs object with osm ones",
    rename_all = "kebab-case"
)]
struct Opt {
    /// input directory.
    #[structopt(short, long, parse(from_os_str), default_value = ".")]
    input: PathBuf,

    /// osm pbf file.
    #[structopt(short, long, parse(from_os_str))]
    pbf: PathBuf,

    /// networks mapping
    #[structopt(short, long)]
    networks: Vec<String>,

    /// output directory
    #[structopt(short, long, parse(from_os_str))]
    output: PathBuf,

    /// force double matching between ntfs and osm stop_point
    #[structopt(short, long)]
    force_double_stop_point_matching: bool,

    /// current datetime
    #[structopt(
        short = "x",
        long,
        parse(try_from_str),
        raw(default_value = "&transit_model::CURRENT_DATETIME")
    )]
    current_datetime: NaiveDateTime,
}

fn run() -> Result<()> {
    info!("Launching map-ntfs-with-osm.");

    let opt = Opt::from_args();
    let model = ntfs::read(opt.input)?;
    let mut ntfs_network_to_osm = HashMap::new();
    for network_map in opt.networks.iter() {
        let split: Vec<_> = network_map.split('=').collect();
        match split.len() {
            2 => ntfs_network_to_osm.insert(split[0], split[1]),
            _ => bail!("networks mapping should be like ntfs_network_id=osm_network_label"),
        };
    }
    if ntfs_network_to_osm.is_empty() {
        bail!("networks mapping should contain at least one mapping");
    }
    let enriched_model = improve_stop_positions::enrich_object_codes(
        &opt.pbf,
        model,
        ntfs_network_to_osm,
        opt.force_double_stop_point_matching,
    )?;
    transit_model::ntfs::write(&enriched_model, opt.output, opt.current_datetime)?;

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
