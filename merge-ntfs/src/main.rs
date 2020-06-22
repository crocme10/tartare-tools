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
use std::{collections::BTreeMap, fs::File, path::PathBuf};
use structopt::StructOpt;
use transfers::{transfers, TransfersMode};
use transit_model::{model::Collections, Result};

mod merge_collections;

#[derive(Debug, StructOpt)]
#[structopt(
    name = "merge-ntfs",
    about = "Merge several ntfs into one",
    rename_all = "kebab-case"
)]
struct Opt {
    /// Input directories to process
    #[structopt(name = "INPUTS", parse(from_os_str))]
    input_directories: Vec<PathBuf>,

    /// output directory
    #[structopt(short, long, parse(from_os_str))]
    output: PathBuf,

    /// config csv rule files.
    #[structopt(short = "c", long = "config", parse(from_os_str))]
    rule_files: Vec<PathBuf>,

    /// json file for feed_infos
    #[structopt(short, long, parse(from_os_str))]
    feed_infos: Option<PathBuf>,

    /// output report file path
    #[structopt(short, long, parse(from_os_str))]
    report: Option<PathBuf>,

    // The max distance in meters to compute the tranfer
    #[structopt(long, short = "d", default_value = transit_model::TRANSFER_MAX_DISTANCE)]
    max_distance: f64,

    // The walking speed in meters per second.
    // You may want to divide your initial speed by sqrt(2) to simulate Manhattan distances
    #[structopt(long, short = "s", default_value = transit_model::TRANSFER_WAKING_SPEED)]
    walking_speed: f64,

    // Waiting time at stop in second
    #[structopt(long, short = "t", default_value = transit_model::TRANSFER_WAITING_TIME)]
    waiting_time: u32,

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
    info!("Launching merge...");

    if opt.input_directories.len() < 2 {
        bail!("merge-ntfs process should have at least two input directories")
    } else {
        let mut collections = Collections::default();
        for input_directory in opt.input_directories {
            let to_append_model = transit_model::ntfs::read(input_directory)?;
            collections = merge_collections::try_merge_collections(
                collections,
                to_append_model.into_collections(),
            )?;
        }

        if let Some(config_feed_infos) = opt.feed_infos {
            info!("Reading feed_infos from {:?}", config_feed_infos);
            let json_file = File::open(config_feed_infos)?;
            let mut feed_infos: BTreeMap<String, String> = serde_json::from_reader(json_file)?;
            collections.feed_infos.append(&mut feed_infos);
        }

        let model = transit_model::Model::new(collections)?;
        let model = transfers(
            model,
            opt.max_distance,
            opt.walking_speed,
            opt.waiting_time,
            &TransfersMode::InterContributor,
            opt.rule_files,
            opt.report,
        )?;
        transit_model::ntfs::write(&model, opt.output, opt.current_datetime)?;
        Ok(())
    }
}

fn main() {
    tartare_tools::runner::launch_run(run);
}
