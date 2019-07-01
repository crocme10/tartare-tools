// Copyright 2019 Kisio Digital and/or its affiliates.
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
