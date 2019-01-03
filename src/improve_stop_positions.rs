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

use crate::Result;
use failure::format_err;
use geo::algorithm::centroid::Centroid;
use geo::{MultiPoint, Point};
use navitia_model::collection::CollectionWithId;
use navitia_model::model::Collections;
use navitia_model::objects::Coord;
use osm_transit_extractor::*;
use std::collections::BTreeSet;
use std::collections::HashMap;
use std::path::Path;

fn point_list_to_centroid_coord(point_list: Vec<Point<f64>>) -> Coord {
    let multi_point: MultiPoint<_> = point_list.into();
    let centroid = multi_point.centroid().unwrap();
    Coord {
        lon: centroid.x(),
        lat: centroid.y(),
    }
}

pub fn improve_with_pbf(
    osm_pbf_path: &Path,
    collections: &mut Collections,
    min_distance: f64,
) -> Result<()> {
    let mut parsed_pbf = parse_osm_pbf(
        osm_pbf_path
            .to_str()
            .ok_or_else(|| format_err!("osm pbf path is not valid"))?,
    );
    let osm_stop_points_map: HashMap<_, _> = get_stop_points_from_osm(&mut parsed_pbf)
        .into_iter()
        .map(|sp| (sp.id.clone(), sp))
        .collect();
    let mut stop_points = collections.stop_points.take();
    let mut stop_area_ids_to_update = BTreeSet::new();
    for stop_point in stop_points.iter_mut().filter(|sp| !sp.codes.is_empty()) {
        let osm_coords: Vec<Point<f64>> = stop_point
            .codes
            .iter()
            .filter_map(|(code_type, code_value)| match code_type.as_str() {
                "osm_stop_points_id" => osm_stop_points_map.get(code_value).map(|osm_stop_point| {
                    Point::new(osm_stop_point.coord.lon, osm_stop_point.coord.lat)
                }),
                _ => None,
            })
            .collect();
        if osm_coords.is_empty() {
            continue;
        }
        let new_coords = point_list_to_centroid_coord(osm_coords);
        if stop_point.coord.distance_to(&new_coords) > min_distance {
            stop_point.coord = new_coords;
            stop_area_ids_to_update.insert(stop_point.stop_area_id.clone());
        }
    }
    for stop_area_id in stop_area_ids_to_update {
        // @TODO if performance issue, use get_corresponding from navitia_model or find
        // something else
        let osm_coords: Vec<Point<f64>> = stop_points
            .iter()
            .filter(|sp| sp.stop_area_id == stop_area_id)
            .map(|sp| Point::new(sp.coord.lon, sp.coord.lat))
            .collect();
        let new_coords = point_list_to_centroid_coord(osm_coords);
        collections.stop_areas.get_mut(&stop_area_id).unwrap().coord = new_coords;
    }
    collections.stop_points = CollectionWithId::new(stop_points)?;
    Ok(())
}
