//! PIV format management.

mod read;

use log::{info, Level as LogLevel};
use skip_error::skip_error_and_log;
use std::path::Path;
use transit_model::{
    model::{Collections, Model},
    AddPrefix, PrefixConfiguration, Result,
};
use typed_index_collection::CollectionWithId;
use walkdir::WalkDir;

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
    let mut collections = Collections::default();
    let (contributor, dataset, feed_infos) = transit_model::read_utils::read_config(config_path)?;
    collections.contributors = CollectionWithId::from(contributor);
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
            "Reading transportation plan in folder {:?}",
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

    collections.calendar_deduplication();
    Model::new(collections)
}
