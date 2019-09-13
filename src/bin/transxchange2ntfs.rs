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

use chrono::{NaiveDate, NaiveDateTime};
use log::info;
use std::path::PathBuf;
use structopt;
use structopt::StructOpt;
use transit_model::Result;

#[derive(Debug, StructOpt)]
#[structopt(
    name = "transxchange2ntfs",
    about = "Convert a TransXChange to an NTFS."
)]
struct Opt {
    /// input directory or ZIP file containing TransXChange files
    /// the files must be UTF-8 encoded
    #[structopt(long, short, parse(from_os_str), default_value = ".")]
    input: PathBuf,

    /// input directory or ZIP file containing NaPTAN files
    /// the files must be UTF-8 encoded
    #[structopt(long, short, parse(from_os_str), default_value = ".")]
    naptan: PathBuf,

    /// output directory for the NTFS files
    #[structopt(short, long, parse(from_os_str))]
    output: PathBuf,

    /// source of bank holidays as a path to a JSON
    #[structopt(short, long, parse(from_os_str))]
    bank_holidays: Option<PathBuf>,

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
        raw(default_value = "&transit_model::CURRENT_DATETIME")
    )]
    current_datetime: NaiveDateTime,

    /// limit the data in the future
    #[structopt(short, long, parse(try_from_str))]
    max_end_date: NaiveDate,
}

fn run(opt: Opt) -> Result<()> {
    info!("Launching transxchange2ntfs...");

    let model = transit_model::transxchange::read(
        opt.input,
        opt.naptan,
        opt.bank_holidays,
        opt.config,
        opt.prefix,
        opt.max_end_date,
    )?;

    transit_model::ntfs::write(&model, opt.output, opt.current_datetime)?;
    Ok(())
}

fn main() {
    tartare_tools::runner::launch_run(run);
}
