//! PIV format management.

mod read;

use chrono::naive::{MAX_DATE, MIN_DATE};
use log::{info, Level as LogLevel};
use skip_error::skip_error_and_log;
use std::path::Path;
use transit_model::{
    model::{Collections, Model},
    objects::{Dataset, StopPoint, Transfer},
    AddPrefix, PrefixConfiguration, Result,
};
use typed_index_collection::{CollectionWithId, Id};
use walkdir::WalkDir;

fn generate_transfers(collections: &mut Collections) -> Result<()> {
    for (stop_area_idx, _) in collections.stop_areas.iter() {
        let stop_area_id = &collections.stop_areas[stop_area_idx].id();
        let list_stop_points: Vec<&StopPoint> = collections
            .stop_points
            .values()
            .filter(|stop_point| &stop_point.stop_area_id == stop_area_id)
            .collect();
        let min_transfer_time = 300;
        let waiting_time = 120;
        for from_stop in list_stop_points.iter() {
            for to_stop in list_stop_points.iter() {
                let transfer = if from_stop == to_stop {
                    Transfer {
                        from_stop_id: from_stop.id.clone(),
                        to_stop_id: to_stop.id.clone(),
                        min_transfer_time: Some(0),
                        real_min_transfer_time: Some(waiting_time),
                        equipment_id: None,
                    }
                } else {
                    Transfer {
                        from_stop_id: from_stop.id.clone(),
                        to_stop_id: to_stop.id.clone(),
                        min_transfer_time: Some(min_transfer_time),
                        real_min_transfer_time: Some(min_transfer_time + waiting_time),
                        equipment_id: None,
                    }
                };
                collections.transfers.push(transfer);
            }
        }
    }
    Ok(())
}

/// Imports a `Model` from the PIV files in the `path` directory.
///
/// The `config_path` argument allows you to give a path to a file
/// containing a json representing the contributor and dataset used
/// for this PIV. If not given, default values will be created.
///
/// The `prefix` argument is a string that will be prepended to every
/// identifiers, allowing to namespace the dataset. By default, no
/// prefix will be added to the identifiers.
pub fn read<P>(piv_path: P, config_path: Option<P>, prefix: Option<String>) -> Result<Model>
where
    P: AsRef<Path>,
{
    fn init_dataset_validity_period(dataset: &mut Dataset) {
        dataset.start_date = MAX_DATE;
        dataset.end_date = MIN_DATE;
    }

    let mut collections = Collections::default();
    let (contributor, mut dataset, feed_infos) =
        transit_model::read_utils::read_config(config_path)?;
    collections.contributors = CollectionWithId::from(contributor);
    init_dataset_validity_period(&mut dataset);
    collections.datasets = CollectionWithId::from(dataset);
    collections.feed_infos = feed_infos;

    let path = piv_path.as_ref();
    for transportation_plan in WalkDir::new(path)
        .min_depth(1)
        .max_depth(1)
        .into_iter()
        .filter_map(std::result::Result::ok)
        .filter(|dir_entry| dir_entry.file_type().is_dir())
    {
        info!(
            "Reading daily transportation plan in folder {:?}",
            transportation_plan.path()
        );
        skip_error_and_log!(
            read::read_daily_transportation_plan(transportation_plan.path(), &mut collections,),
            LogLevel::Warn
        );
    }

    if let Some(prefix) = prefix {
        let mut prefix_conf = PrefixConfiguration::default();
        prefix_conf.set_data_prefix(prefix);
        collections.prefix(&prefix_conf);
    }

    generate_transfers(&mut collections)?;
    collections.calendar_deduplication();
    Model::new(collections)
}

#[cfg(test)]
mod tests {
    use super::*;
    use transit_model::objects::StopArea;

    #[test]
    fn test_generate_transfers() {
        let mut collections = Collections::default();
        let sa = StopArea {
            id: String::from("SA"),
            ..Default::default()
        };
        let sp1 = StopPoint {
            id: String::from("SP1"),
            stop_area_id: String::from("SA"),
            ..Default::default()
        };
        let sp2 = StopPoint {
            id: String::from("SP2"),
            stop_area_id: String::from("SA"),
            ..Default::default()
        };
        collections.stop_areas.push(sa).unwrap();
        collections.stop_points.push(sp1).unwrap();
        collections.stop_points.push(sp2).unwrap();
        generate_transfers(&mut collections).unwrap();

        assert_eq!(4, collections.transfers.len());
        let mut transfers = collections.transfers.values();
        let transfer1 = transfers.next().unwrap();
        assert_eq!("SP1", transfer1.from_stop_id);
        assert_eq!("SP1", transfer1.to_stop_id);
        assert_eq!(0, transfer1.min_transfer_time.unwrap());
        assert_eq!(120, transfer1.real_min_transfer_time.unwrap());
        let transfer2 = transfers.next().unwrap();
        assert_eq!("SP1", transfer2.from_stop_id);
        assert_eq!("SP2", transfer2.to_stop_id);
        assert_eq!(300, transfer2.min_transfer_time.unwrap());
        assert_eq!(420, transfer2.real_min_transfer_time.unwrap());
    }
}
