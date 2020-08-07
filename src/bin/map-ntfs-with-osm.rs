use chrono::{DateTime, FixedOffset};
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
        default_value = &transit_model::CURRENT_DATETIME
    )]
    current_datetime: DateTime<FixedOffset>,
}

fn run(opt: Opt) -> Result<()> {
    info!("Launching map-ntfs-with-osm.");

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
    tartare_tools::runner::launch_run(run);
}
