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
use log::info;
use std::path::PathBuf;
use structopt::{clap::arg_enum, StructOpt};
use transit_model::{ntfs, Result};

arg_enum! {
    #[derive(Debug)]
    enum Action {
        Extract,
        Remove,
    }
}

impl Into<ntfs::filter::Action> for Action {
    fn into(self) -> ntfs::filter::Action {
        match self {
            Action::Extract => ntfs::filter::Action::Extract,
            Action::Remove => ntfs::filter::Action::Remove,
        }
    }
}

#[derive(Debug, StructOpt)]
#[structopt(
    name = "filter_ntfs",
    about = "Remove or extract networks from an NTFS. "
)]
struct Opt {
    /// Input directory
    #[structopt(short, long, parse(from_os_str), default_value = ".")]
    input: PathBuf,

    /// Extract or remove networks
    #[structopt(possible_values = &Action::variants(), case_insensitive = true)]
    action: Action,

    /// Network ids
    #[structopt(short, long)]
    networks: Vec<String>,

    /// Current datetime
    #[structopt(
        short = "x",
        long,
        parse(try_from_str),
        default_value = &transit_model::CURRENT_DATETIME
    )]
    current_datetime: NaiveDateTime,

    /// Output directory
    #[structopt(short, long, parse(from_os_str))]
    output: PathBuf,
}

fn run(opt: Opt) -> Result<()> {
    info!("Launching filter-ntfs.");

    let model = transit_model::ntfs::read(opt.input)?;
    info!("{:?} networks {:?}", opt.action, opt.networks);
    let model = ntfs::filter::filter(model, opt.action.into(), opt.networks)?;
    transit_model::ntfs::write(&model, opt.output, opt.current_datetime)?;

    Ok(())
}

fn main() {
    tartare_tools::runner::launch_run(run);
}
