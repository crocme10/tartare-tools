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

mod netexidf;

#[derive(Debug, StructOpt)]
#[structopt(name = "netexidf2ntfs", about = "Convert a Netex IDF to an NTFS.")]
struct Opt {
    /// input directory containing Netex IDF files
    #[structopt(long, short, parse(from_os_str), default_value = ".")]
    input: PathBuf,

    /// output directory for the NTFS files
    #[structopt(short, long, parse(from_os_str))]
    output: PathBuf,

    /// config file
    #[structopt(short, long, parse(from_os_str))]
    config: Option<PathBuf>,

    /// prefix
    #[structopt(short, long)]
    prefix: Option<String>,

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
    info!("Launching netexidf2ntfs...");

    let model = netexidf::read(opt.input, opt.config, opt.prefix)?;

    transit_model::ntfs::write(&model, opt.output, opt.current_datetime)?;
    Ok(())
}

fn main() {
    tartare_tools::runner::launch_run(run);
}
