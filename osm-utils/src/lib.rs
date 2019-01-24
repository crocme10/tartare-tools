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

pub mod poi;

use failure;
use failure::format_err;
use geo::centroid::Centroid;
use osm_boundaries_utils::build_boundary;
use osmpbfreader;
use std::collections::BTreeMap;
use std::fs::File;
use navitia_poi_model::objects;

pub type Error = failure::Error;
pub type Result<T> = std::result::Result<T, Error>;

pub type OsmPbfReader = osmpbfreader::OsmPbfReader<File>;

/// Returns arbitrary Coord on the way.
/// A middle node is chosen as a better marker on a street
/// than the first node.
pub fn get_way_coord(
    obj_map: &BTreeMap<osmpbfreader::OsmId, osmpbfreader::OsmObj>,
    way: &osmpbfreader::objects::Way,
) -> Result<objects::Coord> {
    let nb_nodes = way.nodes.len();
    way.nodes
        .iter()
        .skip(nb_nodes / 2)
        .filter_map(|node_id| obj_map.get(&(*node_id).into()))
        .filter_map(|obj| obj.node())
        .map(|node| objects::Coord::new(node.lon(), node.lat()))
        .next()
        .ok_or_else(|| {
            format_err!(
                "Imposible to get the coordinate of the median node of the way {:?}",
                way.id.0
            )
        })
}

/// Returns Coord on the relation.
pub fn get_relation_coord(
    obj_map: &BTreeMap<osmpbfreader::OsmId, osmpbfreader::OsmObj>,
    relation: &osmpbfreader::objects::Relation,
) -> Result<objects::Coord> {
    let boundary = build_boundary(relation, obj_map);
    boundary
        .as_ref()
        .and_then(|b| b.centroid().map(|c| objects::Coord::new(c.x(), c.y())))
        .ok_or_else(|| {
            format_err!(
                "Imposible to get the centroid coordinates of the relation {:?}",
                relation.id.0
            )
        })
}
