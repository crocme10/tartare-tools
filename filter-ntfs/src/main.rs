use chrono::{DateTime, FixedOffset};
use failure::format_err;
use log::info;
use std::path::PathBuf;
use structopt::{clap::arg_enum, StructOpt};
use transit_model::Result;

mod filter;

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

    /// Extract or remove networks and / or lines
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
        let (property, value) = f
            .find(':')
            .map(|pos| (&f[0..pos], &f[pos + 1..]))
            .ok_or_else(|| {
                format_err!(
                    "expected filter should be \"property:value\", \"{}\" given",
                    f
                )
            })?;

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

    let model = filter::filter(model, &filter)?;
    transit_model::ntfs::write(&model, opt.output, opt.current_datetime)?;

    Ok(())
}

fn main() {
    tartare_tools::runner::launch_run(run);
}
