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
use std::path::PathBuf;
use structopt;
use structopt::StructOpt;
use transit_model;
use transit_model::Result;

#[derive(Debug, StructOpt)]
#[structopt(name = "kv12ntfs", about = "Convert a KV1 to an NTFS.")]
struct Opt {
    /// input directory.
    #[structopt(short = "i", long = "input", parse(from_os_str), default_value = ".")]
    input: PathBuf,

    /// output directory
    #[structopt(short = "o", long = "output", parse(from_os_str))]
    output: PathBuf,

    /// config file
    #[structopt(short = "c", long = "config", parse(from_os_str))]
    config_path: Option<PathBuf>,

    /// prefix
    #[structopt(short = "p", long = "prefix")]
    prefix: Option<String>,

    /// current datetime
    #[structopt(
        short = "x",
        long,
        parse(try_from_str),
        default_value = &transit_model::CURRENT_DATETIME
    )]
    current_datetime: NaiveDateTime,
}

fn run(opt: Opt) -> Result<()> {
    info!("Launching kv12ntfs...");

    let objects = if opt.input.is_file() {
        transit_model::kv1::read_from_zip(opt.input, opt.config_path, opt.prefix)?
    } else if opt.input.is_dir() {
        transit_model::kv1::read_from_path(opt.input, opt.config_path, opt.prefix)?
    } else {
        bail!("Invalid input data: must be an existing directory or a ZIP archive");
    };

    transit_model::ntfs::write(&objects, opt.output, opt.current_datetime)?;
    Ok(())
}

fn main() {
    tartare_tools::runner::launch_run(run);
}
