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

pub mod objects;
pub mod poi;

use geo::centroid::Centroid;
use geo::MultiPolygon;
use std::collections::BTreeMap;
use std::fs::File;
#[macro_use]
extern crate log;
extern crate serde;
#[macro_use]
extern crate serde_derive;
use failure;
use osmpbfreader;
use std::path::Path;

pub type Error = failure::Error;

pub type OsmPbfReader = osmpbfreader::OsmPbfReader<File>;

pub fn make_osm_reader(path: &Path) -> Result<OsmPbfReader, Error> {
    Ok(osmpbfreader::OsmPbfReader::new(File::open(&path)?))
}

pub fn get_way_coord(
    obj_map: &BTreeMap<osmpbfreader::OsmId, osmpbfreader::OsmObj>,
    way: &osmpbfreader::objects::Way,
) -> objects::Coord {
    /*
        Returns arbitrary Coord on the way.
        A middle node is chosen as a better marker on a street
        than the first node.
    */
    let nb_nodes = way.nodes.len();
    way.nodes
        .iter()
        .skip(nb_nodes / 2)
        .filter_map(|node_id| obj_map.get(&(*node_id).into()))
        .filter_map(|obj| obj.node())
        .map(|node| objects::Coord::new(node.lon(), node.lat()))
        .next()
        .unwrap_or_else(objects::Coord::default)
}

pub fn make_centroid(boundary: &Option<MultiPolygon<f64>>) -> objects::Coord {
    let coord = boundary
        .as_ref()
        .and_then(|b| b.centroid().map(|c| objects::Coord::new(c.x(), c.y())))
        .unwrap_or_else(objects::Coord::default);
    if coord.is_valid() {
        coord
    } else {
        objects::Coord::default()
    }
}
