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

use navitia_model::collection::CollectionWithId;
use navitia_model::model::Collections;
use navitia_model::objects::Coord;
use navitia_model::Result;
use osm_transit_extractor::*;

use std::collections::BTreeSet;

fn get_centroid_from_coordinates(coordinates: Vec<Coord>) -> Coord {
    let sum = coordinates
        .iter()
        .fold(Coord { lon: 0.0, lat: 0.0 }, |a, b| Coord {
            lon: a.lon + b.lon,
            lat: a.lat + b.lat,
        });
    Coord {
        lon: sum.lon / coordinates.len() as f64,
        lat: sum.lat / coordinates.len() as f64,
    }
}

pub fn improve_with_pbf(
    osm_pbf_path: &str,
    collections: &mut Collections,
    min_distance: f64,
) -> Result<()> {
    let mut parsed_pbf = parse_osm_pbf(osm_pbf_path);
    let osm_stop_points = get_stop_points_from_osm(&mut parsed_pbf);
    let sq_min_distance = min_distance * min_distance;
    let mut stop_points = collections.stop_points.take();
    let mut stop_area_ids_to_update = BTreeSet::new();
    for stop_point in stop_points.iter_mut().filter(|sp| !sp.codes.is_empty()) {
        let osm_coords: Vec<Coord> = stop_point
            .codes
            .iter()
            .filter_map(|(code_type, code_value)| match code_type.as_str() {
                "osm_stop_points_id" => osm_stop_points
                    .iter()
                    .find(|sp| &sp.id == code_value)
                    .and_then(|osm_stop_point| {
                        Some(Coord {
                            lon: osm_stop_point.coord.lon,
                            lat: osm_stop_point.coord.lat,
                        })
                    }),
                _ => None,
            })
            .collect();
        if osm_coords.is_empty() {
            continue;
        }
        let new_coords = get_centroid_from_coordinates(osm_coords);
        let sq_distance = stop_point.coord.approx().sq_distance_to(&new_coords);
        if sq_distance > sq_min_distance {
            stop_point.coord = new_coords.clone();
            stop_area_ids_to_update.insert(stop_point.stop_area_id.clone());
        }
    }
    let mut stop_areas = collections.stop_areas.take();
    for stop_area_id in stop_area_ids_to_update {
        let coords: Vec<_> = stop_points
            .iter()
            .filter(|sp| sp.stop_area_id == stop_area_id)
            .map(|sp| sp.coord)
            .collect();
        stop_areas
            .iter_mut()
            .find(|sa| sa.id == stop_area_id)
            .unwrap()
            .coord = get_centroid_from_coordinates(coords);
    }
    collections.stop_points = CollectionWithId::new(stop_points)?;
    collections.stop_areas = CollectionWithId::new(stop_areas)?;
    Ok(())
}
