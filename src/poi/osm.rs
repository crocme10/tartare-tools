// Copyright 2017-2018 Kisio Digital and/or its affiliates.
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

use crate::{poi::Model, Result};
use log::info;
use osm_utils::poi::{extract_pois as extract_osm_pois, PoiConfig};
use osmpbfreader;
use std::fs::File;
use std::path::Path;

pub fn extract_pois<P: AsRef<Path>>(osm_path: P, matcher: PoiConfig) -> Result<Model> {
    info!("Extracting pois from osm");
    let mut osm_reader = osmpbfreader::OsmPbfReader::new(File::open(osm_path.as_ref())?);
    let pois = extract_osm_pois(&mut osm_reader, &matcher);

    Ok(Model {
        pois,
        poi_types: matcher.poi_types,
    })
}
