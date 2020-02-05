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
use failure::bail;
use log::info;
use std::path::PathBuf;
use structopt::{clap::arg_enum, StructOpt};
use transit_model::{ntfs::filter, Result};

arg_enum! {
    #[derive(Debug)]
    enum Action {
        Extract,
        Remove,
    }
}

impl Into<filter::Action> for Action {
    fn into(self) -> filter::Action {
        match self {
            Action::Extract => filter::Action::Extract,
            Action::Remove => filter::Action::Remove,
        }
    }
}

#[derive(Debug, StructOpt)]
#[structopt(
    name = "filter_ntfs",
    about = "Remove or extract objects from an NTFS. "
)]
struct Opt {
    /// Input directory
    #[structopt(short, long, parse(from_os_str), default_value = ".")]
    input: PathBuf,

    /// Extract or remove networks
    #[structopt(possible_values = &Action::variants(), case_insensitive = true)]
    action: Action,

    /// Network filters
    #[structopt(short, long)]
    networks: Vec<String>,

    /// Line filters
    #[structopt(short, long)]
    lines: Vec<String>,

    /// Current datetime
    #[structopt(
        short = "x",
        long,
        parse(try_from_str),
        default_value = &transit_model::CURRENT_DATETIME
    )]
    current_datetime: DateTime<FixedOffset>,

    /// Output directory
    #[structopt(short, long, parse(from_os_str))]
    output: PathBuf,
}

fn add_filters(
    filter: &mut filter::Filter,
    object_type: filter::ObjectType,
    filters: Vec<String>,
) -> Result<()> {
    for f in filters {
        let mut params = f.split(':');
        let (property, value) = match (params.next(), params.next(), params.next()) {
            (Some(p), Some(v), None) => (p, v),
            _ => bail!(
                "expected filter should be \"property:value\", \"{}\" given",
                f
            ),
        };

        filter.add(object_type, property, value);
    }
    Ok(())
}

fn run(opt: Opt) -> Result<()> {
    info!("Launching filter-ntfs.");

    let model = transit_model::ntfs::read(opt.input)?;

    let mut filter = filter::Filter::new(opt.action.into());
    add_filters(&mut filter, filter::ObjectType::Network, opt.networks)?;
    add_filters(&mut filter, filter::ObjectType::Line, opt.lines)?;

    let model = filter::filter(model, filter)?;
    transit_model::ntfs::write(&model, opt.output, opt.current_datetime)?;

    Ok(())
}

fn main() {
    tartare_tools::runner::launch_run(run);
}
