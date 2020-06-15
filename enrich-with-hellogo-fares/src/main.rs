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

use chrono::{DateTime, FixedOffset};
use log::info;
use std::path::PathBuf;
use structopt::StructOpt;
use transit_model::Result;

mod hellogo_fares;

#[derive(Debug, StructOpt)]
#[structopt(
    name = "enrich_with_hellogo_fares",
    about = "Enrich the data of an NTFS with HelloGo fares."
)]
struct Opt {
    /// input directory.
    #[structopt(short, long, parse(from_os_str), default_value = ".")]
    input: PathBuf,

    /// HelloGo fares directory.
    #[structopt(short, long, parse(from_os_str), default_value = ".")]
    fares: PathBuf,

    /// output directory
    #[structopt(short, long, parse(from_os_str))]
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
    info!("Launching enrich_with_hellogo_fares.");
    let model = transit_model::ntfs::read(opt.input)?;
    let mut collections = model.into_collections();
    hellogo_fares::enrich_with_hellogo_fares(&mut collections, opt.fares)?;
    let model = transit_model::Model::new(collections)?;
    transit_model::ntfs::write(&model, opt.output, opt.current_datetime)?;

    Ok(())
}

fn main() {
    tartare_tools::runner::launch_run(run);
}
