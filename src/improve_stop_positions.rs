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
use failure::bail;
use failure::format_err;
use geo::algorithm::centroid::Centroid;
use geo::{MultiPoint, Point};
use log::info;
use navitia_model::collection::{CollectionWithId, Idx};
use navitia_model::model::{Collections, Model};
use navitia_model::objects::{Coord, StopPoint as NtfsStopPoint, VehicleJourney};
use osm_transit_extractor::*;
use std::collections::{BTreeSet, HashMap, HashSet};
use std::path::Path;
use unidecode::unidecode;

fn point_list_to_centroid_coord(point_list: Vec<Point<f64>>) -> Coord {
    let multi_point: MultiPoint<_> = point_list.into();
    let centroid = multi_point.centroid().unwrap();
    Coord {
        lon: centroid.x(),
        lat: centroid.y(),
    }
}

type StopPointMap = HashMap<String, BTreeSet<String>>;

fn compare_almost_equal(a: &str, b: &str) -> bool {
    sanitize(a) == sanitize(b)
}

fn sanitize(broken: &str) -> String {
    unidecode(&broken).to_lowercase()
}

pub fn enrich_object_codes(
    osm_pbf_path: &Path,
    model: Model,
    ntfs_network_to_osm: HashMap<&str, &str>,
    force_double_stop_point_matching: bool,
) -> Result<Model> {
    for (ntfs_network_id, _) in ntfs_network_to_osm.iter() {
        if model.networks.get(&ntfs_network_id).is_none() {
            bail!(
                "ntfs network id {:?} from mapping does not exist in ntfs",
                &ntfs_network_id
            )
        }
    }
    let mut parsed_pbf = parse_osm_pbf(
        osm_pbf_path
            .to_str()
            .ok_or_else(|| format_err!("osm pbf path is not valid"))?,
    );
    let objects = get_osm_tcobjects(&mut parsed_pbf, false);
    let mut ntfs_lines = model.lines.clone().take();
    let mut ntfs_routes = model.routes.clone();
    let mut ntfs_stop_points = model.stop_points.clone();
    let osm_lines = match &objects.lines {
        Some(lines) => lines,
        None => {
            bail!(
                "no lines found in osm for file {}",
                osm_pbf_path.to_str().unwrap()
            );
        }
    };
    let osm_stops_map = objects
        .stop_points
        .iter()
        .map(|sp| (&sp.id, sp))
        .collect::<HashMap<_, _>>();
    let osm_routes_map = match &objects.routes {
        Some(routes) => routes.iter().map(|r| (&r.id, r)).collect::<HashMap<_, _>>(),
        None => {
            bail!(
                "no routes found in osm for file {}",
                osm_pbf_path.to_str().unwrap()
            );
        }
    };
    let mut map_ntfs_to_osm_points: StopPointMap = HashMap::new();
    let mut map_osm_to_ntfs_points: StopPointMap = HashMap::new();
    for line in ntfs_lines.iter_mut() {
        let osm_network_of_line = match ntfs_network_to_osm.get(&line.network_id as &str) {
            Some(osm_network) => osm_network,
            None => continue,
        };
        if let Some(code) = &line.code {
            let corresponding_osm_lines = osm_lines
                .iter()
                .filter(|l| {
                    compare_almost_equal(&l.network, &osm_network_of_line)
                        && l.all_osm_tags.contains("ref", &code)
                })
                .collect::<Vec<_>>();
            if corresponding_osm_lines.len() == 1 {
                let corresponding_osm_line = corresponding_osm_lines[0];
                line.codes
                    .insert(("osm_line_id".to_string(), corresponding_osm_line.id.clone()));
                line.codes.insert((
                    "osm_network".to_string(),
                    corresponding_osm_line.network.clone(),
                ));
                line.codes.insert((
                    "osm_company".to_string(),
                    corresponding_osm_line.operator.clone(),
                ));
                let routes_of_line = osm_routes_map
                    .iter()
                    .filter_map(|(id, route)| {
                        if corresponding_osm_line.routes_id.contains(id) {
                            Some(route)
                        } else {
                            None
                        }
                    })
                    .collect::<Vec<_>>();
                let vjs_idx: BTreeSet<Idx<VehicleJourney>> =
                    model.get_corresponding_from_idx(model.lines.get_idx(&line.id).unwrap());
                let vjs: HashMap<Idx<VehicleJourney>, &VehicleJourney> = vjs_idx
                    .iter()
                    .map(|vj_idx| (*vj_idx, &model.vehicle_journeys[*vj_idx]))
                    .collect();
                let mut vj_patterns: HashSet<(Vec<Idx<NtfsStopPoint>>, &str, &str)> =
                    HashSet::new();
                for vj in vjs.values() {
                    vj_patterns.insert((
                        vj.stop_times.iter().map(|st| st.stop_point_idx).collect(),
                        &model.stop_points[vj.stop_times.last().unwrap().stop_point_idx].name,
                        &vj.route_id,
                    ));
                }
                for route in routes_of_line {
                    let route_points = route
                        .ordered_stops_id
                        .iter()
                        .filter_map(|stop_id| osm_stops_map.get(stop_id))
                        .collect::<Vec<_>>();
                    let corresponding_vj_patterns = vj_patterns
                        .iter()
                        .filter(|(stops, destination, _)| {
                            stops.len() == route_points.len()
                                && compare_almost_equal(destination, &route.destination)
                        })
                        .collect::<Vec<_>>();
                    for (stop_points_idx, _, ntfs_route_id) in corresponding_vj_patterns {
                        let mut ntfs_route = ntfs_routes.get_mut(ntfs_route_id).unwrap();
                        ntfs_route
                            .codes
                            .insert(("osm_route_id".to_string(), route.id.clone()));
                        for route_point in &route_points {
                            for stop_point_idx in stop_points_idx {
                                let ntfs_stop_point = ntfs_stop_points.index_mut(*stop_point_idx);
                                if compare_almost_equal(&ntfs_stop_point.name, &route_point.name) {
                                    map_ntfs_to_osm_points
                                        .entry(ntfs_stop_point.id.clone())
                                        .or_insert(BTreeSet::new())
                                        .insert(route_point.id.clone());
                                    map_osm_to_ntfs_points
                                        .entry(route_point.id.clone())
                                        .or_insert(BTreeSet::new())
                                        .insert(ntfs_stop_point.id.clone());
                                }
                            }
                        }
                    }
                }
            } else {
                info!(
                    "found {} osm lines corresponding to line {:?}",
                    corresponding_osm_lines.len(),
                    &line.id
                );
            }
        }
    }
    clean_up_multiple_mappings(
        &mut map_ntfs_to_osm_points,
        &mut map_osm_to_ntfs_points,
        force_double_stop_point_matching,
    );
    clean_up_multiple_mappings(
        &mut map_osm_to_ntfs_points,
        &mut map_ntfs_to_osm_points,
        force_double_stop_point_matching,
    );
    for (ntfs_point_id, osm_points) in map_ntfs_to_osm_points {
        for osm_stop_id in osm_points {
            ntfs_stop_points
                .get_mut(&ntfs_point_id)
                .unwrap()
                .codes
                .insert(("osm_stop_points_id".to_string(), osm_stop_id));
        }
    }
    let mut collections = model.into_collections();
    collections.stop_points = ntfs_stop_points;
    collections.routes = ntfs_routes;
    collections.lines = CollectionWithId::new(ntfs_lines)?;
    Ok(Model::new(collections)?)
}

fn clean_up_multiple_mappings(
    map: &mut StopPointMap,
    reverse_map: &mut StopPointMap,
    force_double: bool,
) {
    map.retain(|key, points_vec| match (force_double, points_vec.len()) {
        (_, 1) | (true, 2) => true,
        _ => {
            reverse_map.retain(|rev_key, _| !points_vec.contains(rev_key));
            info!("mapping {:?} => {:?} removed", key, points_vec);
            false
        }
    });
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
