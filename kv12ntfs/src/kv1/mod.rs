// Copyright (C) 2017 Kisio Digital and/or its affiliates.
//
// This program is free software: you can redistribute it and/or modify it
// under the terms of the GNU Affero General Public License as published by the
// Free Software Foundation, version 3.

// This program is distributed in the hope that it will be useful, but WITHOUT
// ANY WARRANTY; without even the implied warranty of MERCHANTABILITY or FITNESS
// FOR A PARTICULAR PURPOSE. See the GNU Affero General Public License for more
// details.

// You should have received a copy of the GNU Affero General Public License
// along with this program. If not, see <https://www.gnu.org/licenses/>

//! KV1 format management.

mod read;

use std::path::Path;
use transit_model::{
    model::{Collections, Model},
    read_utils, validity_period, AddPrefix, PrefixConfiguration, Result,
};
use typed_index_collection::CollectionWithId;

/// Imports a `Model` from the KV1 files in the `path` directory.
///
/// The `config_path` argument allows you to give a path to a file
/// containing a json representing the contributor and dataset used
/// for this KV1. If not given, default values will be created.
///
/// The `prefix` argument is a string that will be prepended to every
/// identifiers, allowing to namespace the dataset. By default, no
/// prefix will be added to the identifiers.
pub fn read<P: AsRef<Path>, Q: AsRef<Path>>(
    path: P,
    config_path: Option<Q>,
    prefix: Option<String>,
) -> Result<Model> {
    let mut collections = Collections::default();

    read::read_operday(&path, &mut collections)?;

    let (contributor, mut dataset, feed_infos) = read_utils::read_config(config_path)?;
    validity_period::compute_dataset_validity_period(&mut dataset, &collections.calendars)?;

    collections.contributors = CollectionWithId::new(vec![contributor])?;
    collections.datasets = CollectionWithId::new(vec![dataset])?;
    collections.feed_infos = feed_infos;

    read::read_usrstop_point(&path, &mut collections)?;
    read::read_notice(&path, &mut collections)?;
    read::read_jopa_pujopass_line(&path, &mut collections)?;

    //add prefixes
    if let Some(prefix) = prefix {
        let mut prefix_conf = PrefixConfiguration::default();
        prefix_conf.set_data_prefix(prefix);
        collections.prefix(&prefix_conf);
    }

    collections.calendar_deduplication();
    Model::new(collections)
}
