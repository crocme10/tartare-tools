use navitia_poi_model::Model;
use std::path::Path;

use crate::Result;

/// Attempts to merge the models found in poi files into a single model.
/// This may result in an error for many reasons, including but not limited to
/// - if the files are not in the proper format (expected .poi files),
/// - or if POI exist with the same id.
pub fn merge<I, T>(paths: &mut I) -> Result<Model>
where
    I: Iterator<Item = T>,
    T: AsRef<Path>,
{
    paths.try_fold(Model::default(), |acc, path| {
        acc.try_merge(Model::try_from_path(path.as_ref())?)
    })
}
