use crate::apply_rules::ReportCategory;
use failure::{bail, ResultExt};
use log::info;
use serde::Deserialize;
use std::{
    collections::{BTreeMap, HashMap},
    fmt::{Display, Formatter},
    iter::FromIterator,
    path::Path,
};
use tartare_tools::report::Report;
use transit_model::{
    model::Collections,
    objects::{KeysValues, Route, VehicleJourney},
    Result,
};
use typed_index_collection::CollectionWithId;

#[derive(Deserialize, Debug, Ord, PartialOrd, Eq, PartialEq, Clone, Copy)]
#[serde(rename_all = "snake_case")]
enum ObjectType {
    Line,
    Network,
}
impl Display for ObjectType {
    fn fmt(&self, f: &mut Formatter) -> std::fmt::Result {
        match self {
            ObjectType::Line => write!(f, "line"),
            ObjectType::Network => write!(f, "network"),
        }
    }
}
#[derive(Deserialize, Debug, Ord, Eq, PartialOrd, PartialEq, Clone)]
struct RouteConsolidationConfiguration {
    object_type: ObjectType,
    object_id: String,
}
impl Display for RouteConsolidationConfiguration {
    fn fmt(&self, f: &mut Formatter) -> std::fmt::Result {
        write!(f, "{}:{}", self.object_type, self.object_id)
    }
}

fn reattach_codes_and_object_properties(
    routes: &mut CollectionWithId<Route>,
    routes_ids_to_regroup: &[String],
    route_id_target: &str,
    report: &mut Report<ReportCategory>,
) {
    let mut object_codes = KeysValues::default();
    let mut object_properties = routes
        .get(&route_id_target)
        .map(|route| route.object_properties.clone())
        .map(BTreeMap::from_iter)
        .unwrap_or_else(BTreeMap::new);
    for route_id_to_regroup in routes_ids_to_regroup {
        if let Some(route) = routes.get(route_id_to_regroup) {
            object_codes.extend(route.codes.clone());
            object_codes.insert(("ntfs_source".to_string(), route.id.clone()));
            for (key, value) in &route.object_properties {
                if !object_properties.contains_key(key) {
                    object_properties.insert(key.clone(), value.clone());
                } else {
                    report.add_warning(
                        format!(
                            "Route '{route_id}' already has an object property for '{key}'; object property '{key}:{value}' will be ignored",
                            route_id = route_id_target,
                            key = key,
                            value = value
                        ),
                        ReportCategory::MultipleValue,
                   );
                }
            }
        }
    }
    if let Some(mut route) = routes.get_mut(&route_id_target) {
        route.codes.extend(object_codes);
        route.object_properties.clear();
        route.object_properties.extend(object_properties);
    }
}

fn reattach_vehicle_journeys(
    vehicle_journeys: &mut CollectionWithId<VehicleJourney>,
    routes_ids_to_regroup: &[String],
    route_id_target: &str,
) {
    let vjs_idx = vehicle_journeys
        .iter()
        .filter(|(_, vj)| routes_ids_to_regroup.contains(&vj.route_id))
        .map(|(idx, _)| idx)
        .collect::<Vec<_>>();

    for vj_idx in &vjs_idx {
        vehicle_journeys.index_mut(*vj_idx).route_id = route_id_target.to_string();
    }
}

fn generate_route_id(line_id: &str, direction_type: &str) -> String {
    format!("{}-{}", line_id, direction_type)
}

fn get_routes_by_direction(
    routes: &mut CollectionWithId<Route>,
    line_id: &str,
) -> HashMap<String, Vec<String>> {
    routes
        .values()
        .filter(|r| r.line_id == line_id)
        .filter_map(|r| {
            r.direction_type
                .as_ref()
                .map(|direction_type| (r.id.as_str(), direction_type))
        })
        .filter(|(id, dt)| *id != generate_route_id(line_id, dt))
        .fold(HashMap::new(), |mut map, (id, dt)| {
            map.entry(dt.clone())
                .or_insert_with(Vec::new)
                .push(id.to_string());
            map
        })
}

fn get_route_id_target(
    routes: &mut CollectionWithId<Route>,
    line_id: &str,
    direction_type: &str,
) -> Result<String> {
    let id = generate_route_id(line_id, direction_type);
    if !routes.contains_id(&id) {
        let route = Route {
            id: id.to_string(),
            name: "".to_string(),
            line_id: line_id.to_string(),
            direction_type: Some(direction_type.to_string()),
            ..Default::default()
        };
        routes.push(route)?;
    } else if let Some(existing_route) = routes.get(&id) {
        if existing_route.line_id != line_id {
            bail!(
                "Route \"{}\" already exists in line \"{}\"",
                existing_route.id,
                existing_route.line_id
            );
        } else if let Some(existing_direction_type) = &existing_route.direction_type {
            if existing_direction_type != direction_type {
                bail!(
                    "Route \"{}\" already exists in direction \"{}\"",
                    existing_route.id,
                    existing_direction_type
                );
            }
        }
    };
    Ok(id)
}

fn apply_route_consolidation(
    collections: &mut Collections,
    lines_ids: &[String],
    route_consolidation: &RouteConsolidationConfiguration,
    report: &mut Report<ReportCategory>,
) {
    for line_id in lines_ids {
        let routes_by_direction = get_routes_by_direction(&mut collections.routes, &line_id);
        if routes_by_direction.is_empty() {
            report.add_warning(
                format!(
                    "No route consolidation needed on line id \"{}\" for rule \"{}\"",
                    line_id, route_consolidation
                ),
                ReportCategory::ConsolidationNotApplied,
            );
        } else {
            for (direction_type, routes_ids_to_regroup) in routes_by_direction {
                let route_id_target =
                    match get_route_id_target(&mut collections.routes, line_id, &direction_type) {
                        Ok(val) => val,
                        Err(e) => {
                            report.add_warning(
                                format!(
                                    "Route consolidation impossible for rule \"{}\". {}",
                                    route_consolidation, e
                                ),
                                ReportCategory::ConsolidationNotApplied,
                            );
                            continue;
                        }
                    };
                reattach_vehicle_journeys(
                    &mut collections.vehicle_journeys,
                    &routes_ids_to_regroup,
                    &route_id_target,
                );
                reattach_codes_and_object_properties(
                    &mut collections.routes,
                    &routes_ids_to_regroup,
                    &route_id_target,
                    report,
                );
                collections
                    .routes
                    .retain(|route| !routes_ids_to_regroup.contains(&route.id));
            }
        }
    }
}

fn read_route_consolidation_file<P: AsRef<Path>>(
    route_consolidation_file: P,
    report: &mut Report<ReportCategory>,
) -> Result<Vec<RouteConsolidationConfiguration>> {
    info!("Reading route consolidation rules");
    let mut codes = Vec::new();
    let path = route_consolidation_file.as_ref();
    let mut rdr = csv::ReaderBuilder::new()
        .trim(csv::Trim::All)
        .from_path(&path)
        .with_context(|_| format!("Error reading {:?}", path))?;
    for c in rdr.deserialize() {
        let c: RouteConsolidationConfiguration = match c {
            Ok(val) => val,
            Err(e) => {
                report.add_warning(
                    format!("Error reading {:?}: {}", path.file_name().unwrap(), e),
                    ReportCategory::InvalidFile,
                );
                continue;
            }
        };
        codes.push(c);
    }
    Ok(codes)
}

pub(crate) fn apply_rules<P: AsRef<Path>>(
    route_consolidation_file: Option<P>,
    collections: &mut Collections,
    report: &mut Report<ReportCategory>,
) -> Result<()> {
    if let Some(route_consolidation_file) = route_consolidation_file {
        let route_consolidations = read_route_consolidation_file(route_consolidation_file, report)?;
        for route_consolidation in route_consolidations {
            match route_consolidation.object_type {
                ObjectType::Line => {
                    if let Some(line) = collections.lines.get(&route_consolidation.object_id) {
                        let line_id = line.id.clone();
                        apply_route_consolidation(
                            collections,
                            &[line_id],
                            &route_consolidation,
                            report,
                        );
                    } else {
                        report.add_error(
                            format!(
                                "The line \"{}\" doesn't exist",
                                route_consolidation.object_id
                            ),
                            ReportCategory::ObjectNotFound,
                        );
                    }
                }
                ObjectType::Network => {
                    if let Some(network) = collections.networks.get(&route_consolidation.object_id)
                    {
                        let line_ids = collections
                            .lines
                            .values()
                            .filter(|l| l.network_id == network.id)
                            .map(|l| l.id.clone())
                            .collect::<Vec<_>>();
                        apply_route_consolidation(
                            collections,
                            &line_ids,
                            &route_consolidation,
                            report,
                        );
                    } else {
                        report.add_error(
                            format!(
                                "The network \"{}\" doesn't exist",
                                route_consolidation.object_id
                            ),
                            ReportCategory::ObjectNotFound,
                        );
                    }
                }
            }
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    mod object_properties {
        use super::*;

        #[test]
        fn reattach_all_object_properties() {
            let op1 = ("key1".to_string(), "value1".to_string());
            let op2 = ("key2".to_string(), "value2".to_string());
            let mut keys_values1 = KeysValues::new();
            keys_values1.insert(op1.clone());
            let mut keys_values2 = KeysValues::new();
            keys_values2.insert(op2.clone());
            let mut routes = CollectionWithId::new(vec![
                Route {
                    id: "route_0".into(),
                    ..Default::default()
                },
                Route {
                    id: "route_1".into(),
                    object_properties: keys_values1,
                    ..Default::default()
                },
                Route {
                    id: "route_2".into(),
                    object_properties: keys_values2,
                    ..Default::default()
                },
            ])
            .unwrap();
            let to_regroup = vec!["route_1".into(), "route_2".into()];
            let mut report = Report::default();
            reattach_codes_and_object_properties(&mut routes, &to_regroup, "route_0", &mut report);
            let object_properties = &routes.get("route_0").unwrap().object_properties;
            assert_eq!(2, object_properties.len());
            assert!(object_properties.contains(&op1));
            assert!(object_properties.contains(&op2));
        }

        #[test]
        fn ignore_duplicate_object_properties() {
            let op1 = ("key1".to_string(), "value1".to_string());
            let op2 = ("key1".to_string(), "value2".to_string());
            let mut keys_values1 = KeysValues::new();
            keys_values1.insert(op1.clone());
            let mut keys_values2 = KeysValues::new();
            keys_values2.insert(op2);
            let mut routes = CollectionWithId::new(vec![
                Route {
                    id: "route_0".into(),
                    ..Default::default()
                },
                Route {
                    id: "route_1".into(),
                    object_properties: keys_values1,
                    ..Default::default()
                },
                Route {
                    id: "route_2".into(),
                    object_properties: keys_values2,
                    ..Default::default()
                },
            ])
            .unwrap();
            let to_regroup = vec!["route_1".into(), "route_2".into()];
            let mut report = Report::default();
            reattach_codes_and_object_properties(&mut routes, &to_regroup, "route_0", &mut report);
            let object_properties = &routes.get("route_0").unwrap().object_properties;
            assert_eq!(1, object_properties.len());
            assert!(object_properties.contains(&op1));
        }

        #[test]
        fn reattach_object_properties_from_regrouped_on_route() {
            let op0 = ("key0".to_string(), "value0".to_string());
            let op1 = ("key1".to_string(), "value1".to_string());
            let mut keys_values0 = KeysValues::new();
            keys_values0.insert(op0.clone());
            let mut keys_values1 = KeysValues::new();
            keys_values1.insert(op1.clone());
            let mut routes = CollectionWithId::new(vec![
                Route {
                    id: "route_0".into(),
                    object_properties: keys_values0,
                    ..Default::default()
                },
                Route {
                    id: "route_1".into(),
                    object_properties: keys_values1,
                    ..Default::default()
                },
            ])
            .unwrap();
            let to_regroup = vec!["route_1".into()];
            let mut report = Report::default();
            reattach_codes_and_object_properties(&mut routes, &to_regroup, "route_0", &mut report);
            let object_properties = &routes.get("route_0").unwrap().object_properties;
            assert_eq!(2, object_properties.len());
            assert!(object_properties.contains(&op0));
            assert!(object_properties.contains(&op1));
        }
    }

    mod route_id_regrouping {
        use super::*;

        #[test]
        fn get_new_route_id_regrouping() {
            let mut routes = CollectionWithId::new(vec![Route {
                id: "route_0".into(),
                line_id: "line_0".into(),
                direction_type: Some("clockwise".into()),
                ..Default::default()
            }])
            .unwrap();
            let route_id_regrouping =
                get_route_id_target(&mut routes, "line_0", "clockwise").unwrap();
            let route_regrouping = &routes.get(&route_id_regrouping).unwrap();
            assert_eq!("line_0-clockwise", route_regrouping.id);
            assert_eq!("line_0", route_regrouping.line_id);
            assert_eq!(
                Some(String::from("clockwise")),
                route_regrouping.direction_type
            );
            assert_eq!("", route_regrouping.name);
        }

        #[test]
        fn get_existing_route_id_regrouping() {
            let mut routes = CollectionWithId::new(vec![
                Route {
                    id: "route_0".into(),
                    line_id: "line_0".into(),
                    direction_type: Some("clockwise".into()),
                    ..Default::default()
                },
                Route {
                    id: "line_0-clockwise".into(),
                    line_id: "line_0".into(),
                    name: "original_line_to_keep".into(),
                    direction_type: Some("clockwise".into()),
                    ..Default::default()
                },
            ])
            .unwrap();
            let route_id_regrouping =
                get_route_id_target(&mut routes, "line_0", "clockwise").unwrap();
            let route_regrouping = &routes.get(&route_id_regrouping).unwrap();
            assert_eq!("line_0-clockwise", route_regrouping.id);
            assert_eq!("line_0", route_regrouping.line_id);
            assert_eq!(
                Some(String::from("clockwise")),
                route_regrouping.direction_type
            );
            assert_eq!("original_line_to_keep", route_regrouping.name);
        }

        #[test]
        #[should_panic(
            expected = "Route \\\"line_0-clockwise\\\" already exists in line \\\"line_1\\\""
        )]
        fn get_existing_route_id_regrouping_on_other_line() {
            let mut routes = CollectionWithId::new(vec![
                Route {
                    id: "route_0".into(),
                    line_id: "line_0".into(),
                    direction_type: Some("clockwise".into()),
                    ..Default::default()
                },
                Route {
                    id: "line_0-clockwise".into(),
                    line_id: "line_1".into(),
                    ..Default::default()
                },
            ])
            .unwrap();
            get_route_id_target(&mut routes, "line_0", "clockwise").unwrap();
        }

        #[test]
        #[should_panic(
            expected = "Route \\\"line_0-clockwise\\\" already exists in direction \\\"anticlockwise\\\""
        )]
        fn get_existing_route_id_regrouping_in_other_direction() {
            let mut routes = CollectionWithId::new(vec![
                Route {
                    id: "route_0".into(),
                    line_id: "line_0".into(),
                    direction_type: Some("clockwise".into()),
                    ..Default::default()
                },
                Route {
                    id: "line_0-clockwise".into(),
                    line_id: "line_0".into(),
                    direction_type: Some("anticlockwise".into()),
                    ..Default::default()
                },
            ])
            .unwrap();
            get_route_id_target(&mut routes, "line_0", "clockwise").unwrap();
        }
    }

    mod routes_by_direction {
        use super::*;

        #[test]
        fn basic_routes_by_direction() {
            let mut routes = CollectionWithId::new(vec![
                Route {
                    id: "route_0".into(),
                    line_id: "line_0".into(),
                    direction_type: Some("clockwise".into()),
                    ..Default::default()
                },
                Route {
                    id: "route_1".into(),
                    line_id: "line_0".into(),
                    direction_type: Some("clockwise".into()),
                    ..Default::default()
                },
                Route {
                    id: "route_2".into(),
                    line_id: "line_0".into(),
                    direction_type: Some("anticlockwise".into()),
                    ..Default::default()
                },
            ])
            .unwrap();
            let routes_by_direction = get_routes_by_direction(&mut routes, "line_0");
            assert_eq!(2, routes_by_direction.len());
            assert_eq!(true, routes_by_direction.contains_key("clockwise"));
            assert_eq!(true, routes_by_direction.contains_key("anticlockwise"));
            assert_eq!(
                "route_0",
                routes_by_direction
                    .get("clockwise")
                    .unwrap()
                    .get(0)
                    .unwrap()
            );
            assert_eq!(
                "route_1",
                routes_by_direction
                    .get("clockwise")
                    .unwrap()
                    .get(1)
                    .unwrap()
            );
            assert_eq!(
                "route_2",
                routes_by_direction
                    .get("anticlockwise")
                    .unwrap()
                    .get(0)
                    .unwrap()
            );
        }

        #[test]
        fn routes_by_direction_with_existing_regroup_route() {
            let mut routes = CollectionWithId::new(vec![
                Route {
                    id: "route_0".into(),
                    line_id: "line_0".into(),
                    direction_type: Some("clockwise".into()),
                    ..Default::default()
                },
                Route {
                    id: "line_0-clockwise".into(),
                    line_id: "line_0".into(),
                    direction_type: Some("clockwise".into()),
                    ..Default::default()
                },
            ])
            .unwrap();
            let routes_by_direction = get_routes_by_direction(&mut routes, "line_0");
            assert_eq!(1, routes_by_direction.len());
            assert_eq!(true, routes_by_direction.contains_key("clockwise"));
            assert_eq!(
                "route_0",
                routes_by_direction
                    .get("clockwise")
                    .unwrap()
                    .get(0)
                    .unwrap()
            );
        }

        #[test]
        fn routes_by_direction_route_without_direction() {
            let mut routes = CollectionWithId::new(vec![
                Route {
                    id: "route_0".into(),
                    line_id: "line_0".into(),
                    direction_type: Some("clockwise".into()),
                    ..Default::default()
                },
                Route {
                    id: "route_1".into(),
                    line_id: "line_0".into(),
                    direction_type: None,
                    ..Default::default()
                },
            ])
            .unwrap();
            let routes_by_direction = get_routes_by_direction(&mut routes, "line_0");
            assert_eq!(1, routes_by_direction.len());
            assert_eq!(
                "route_0",
                routes_by_direction
                    .get("clockwise")
                    .unwrap()
                    .get(0)
                    .unwrap()
            );
        }
    }

    mod vehicle_journeys {
        use super::*;

        #[test]
        fn reattach_vjs() {
            let mut vehicle_journeys = CollectionWithId::new(vec![
                VehicleJourney {
                    id: "vj_0".into(),
                    route_id: "route_0".into(),
                    ..Default::default()
                },
                VehicleJourney {
                    id: "vj_1".into(),
                    route_id: "route_0".into(),
                    ..Default::default()
                },
                VehicleJourney {
                    id: "vj_2".into(),
                    route_id: "route_1".into(),
                    ..Default::default()
                },
            ])
            .unwrap();
            reattach_vehicle_journeys(
                &mut vehicle_journeys,
                &["route_0".to_string()],
                "line0-forward",
            );
            assert_eq!(3, vehicle_journeys.len());

            assert_eq!(
                "line0-forward",
                vehicle_journeys.get("vj_0").unwrap().route_id
            );
            assert_eq!(
                "line0-forward",
                vehicle_journeys.get("vj_1").unwrap().route_id
            );
            assert_eq!("route_1", vehicle_journeys.get("vj_2").unwrap().route_id);
        }
    }
}
