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

use crate::apply_rules::ReportCategory;
use failure::{bail, format_err};
use log::info;
use relational_types::IdxSet;
use serde::{de::DeserializeOwned, Deserialize};
use serde_json::Value;
use std::{
    collections::{BTreeSet, HashMap},
    convert::TryFrom,
    fs::File,
    ops::IndexMut,
    path::Path,
};
use tartare_tools::report::Report;
use transit_model::{
    model::{Collections, Model},
    objects::{Line, ObjectType as ModelObjectType, TicketUsePerimeter, VehicleJourney},
    Result,
};
use typed_index_collection::{CollectionWithId, Id};

#[derive(Debug, Deserialize)]
pub struct ObjectProperties {
    properties: Value,
    #[serde(default)]
    grouped_from: Vec<String>,
}

#[derive(Debug, Deserialize)]
struct ObjectRuleConfiguration {
    #[serde(rename = "networks")]
    pub networks_rules: Option<Vec<ObjectProperties>>,
    #[serde(rename = "commercial_modes")]
    pub commercial_modes_rules: Option<Vec<ObjectProperties>>,
    #[serde(rename = "physical_modes")]
    pub physical_modes_rules: Option<Vec<ObjectProperties>>,
}

impl TryFrom<&Path> for ObjectRuleConfiguration {
    type Error = failure::Error;
    fn try_from(path: &Path) -> Result<Self> {
        info!("Reading object rules");
        File::open(path)
            .map_err(|e| format_err!("{}", e))
            .and_then(|file| {
                serde_json::from_reader::<_, ObjectRuleConfiguration>(file)
                    .map_err(|e| format_err!("{}", e))
            })
    }
}

#[derive(Debug)]
pub struct ObjectRule {
    configuration: ObjectRuleConfiguration,
    lines_by_network: Option<HashMap<String, IdxSet<Line>>>,
    ticket_use_perimeters_by_network: Option<HashMap<String, IdxSet<TicketUsePerimeter>>>,
    lines_by_commercial_mode: Option<HashMap<String, IdxSet<Line>>>,
    vjs_by_physical_mode: Option<HashMap<String, IdxSet<VehicleJourney>>>,
}

impl ObjectRule {
    fn check_configuration(configuration: &ObjectRuleConfiguration) -> Result<()> {
        fn check_rules<'a>(rules: &'a [ObjectProperties], id_key: &str) -> Result<()> {
            let mut used_identifiers = BTreeSet::new();
            let mut insert_id = |s: &'a str| -> Result<()> {
                if used_identifiers.insert(s) {
                    Ok(())
                } else {
                    bail!("The {} \"{}\" is present multiple times in the configuration file which is invalid.", id_key, s)
                }
            };
            for rule in rules {
                let id = rule.check(id_key)?;
                insert_id(id)?;
                for regroup_id in &rule.grouped_from {
                    insert_id(regroup_id.as_str())?;
                }
            }
            Ok(())
        }
        if let Some(ref networks_rules) = configuration.networks_rules {
            check_rules(&networks_rules, "network_id")?;
        }
        if let Some(ref commercial_modes_rules) = configuration.commercial_modes_rules {
            check_rules(&commercial_modes_rules, "commercial_mode_id")?;
        }
        if let Some(ref physical_modes_rules) = configuration.physical_modes_rules {
            check_rules(&physical_modes_rules, "physical_mode_id")?;
        }
        Ok(())
    }

    pub(crate) fn new(path: &Path, model: &Model) -> Result<Self> {
        let configuration = ObjectRuleConfiguration::try_from(path)?;
        ObjectRule::check_configuration(&configuration)?;
        let lines_by_network = if configuration.networks_rules.is_some() {
            Some(
                model
                    .networks
                    .iter()
                    .filter_map(|(idx, obj)| {
                        let lines = model.get_corresponding_from_idx(idx);
                        if lines.is_empty() {
                            None
                        } else {
                            Some((obj.id.clone(), lines))
                        }
                    })
                    .collect(),
            )
        } else {
            None
        };
        let ticket_use_perimeters_by_network = if configuration.networks_rules.is_some() {
            Some(
                model
                    .ticket_use_perimeters
                    .iter()
                    .filter(|(_, tup)| tup.object_type == ModelObjectType::Network)
                    .fold(HashMap::new(), |mut map, (idx, tup)| {
                        map.entry(tup.object_id.clone())
                            .or_insert_with(IdxSet::new)
                            .insert(idx);
                        map
                    }),
            )
        } else {
            None
        };
        let lines_by_commercial_mode = if configuration.commercial_modes_rules.is_some() {
            Some(
                model
                    .commercial_modes
                    .iter()
                    .filter_map(|(idx, obj)| {
                        let lines = model.get_corresponding_from_idx(idx);
                        if lines.is_empty() {
                            None
                        } else {
                            Some((obj.id.clone(), lines))
                        }
                    })
                    .collect(),
            )
        } else {
            None
        };
        let vjs_by_physical_mode = if configuration.physical_modes_rules.is_some() {
            Some(
                model
                    .physical_modes
                    .iter()
                    .filter_map(|(idx, obj)| {
                        let vjs = model.get_corresponding_from_idx(idx);
                        if vjs.is_empty() {
                            None
                        } else {
                            Some((obj.id.clone(), vjs))
                        }
                    })
                    .collect(),
            )
        } else {
            None
        };
        let object_rule = ObjectRule {
            configuration,
            lines_by_network,
            ticket_use_perimeters_by_network,
            lines_by_commercial_mode,
            vjs_by_physical_mode,
        };
        Ok(object_rule)
    }
}

impl ObjectProperties {
    fn check(&self, id_key: &'_ str) -> Result<&str> {
        let id = self
            .properties
            .get(id_key)
            .ok_or_else(|| format_err!("Key \"{}\" is required", id_key))?
            .as_str()
            .ok_or_else(|| format_err!("Value for \"{}\" must be filled in", id_key))?;

        Ok(id)
    }

    fn regroup<T, F>(
        &self,
        id: &'_ str,
        collection: &CollectionWithId<T>,
        report: &mut Report<ReportCategory>,
        mut update: F,
    ) -> Result<bool>
    where
        F: FnMut(&str, &str) -> bool,
    {
        let mut changed = false;
        for grouped_id in &self.grouped_from {
            if !collection.contains_id(&grouped_id) {
                report.add_error(
                    format!("The identifier \"{}\" doesn't exist, and therefore cannot be regrouped in \"{}\"", grouped_id, id),
                    ReportCategory::ObjectNotFound,
                );
            } else {
                changed = update(id, grouped_id) || changed;
            }
        }
        Ok(changed)
    }
    fn apply<T, F>(
        &self,
        id_key: &'_ str,
        collection: &mut CollectionWithId<T>,
        report: &mut Report<ReportCategory>,
        update: F,
    ) -> Result<()>
    where
        T: DeserializeOwned + Id<T>,
        F: FnMut(&str, &str) -> bool,
    {
        let id = self.check(id_key)?;
        let created = if !collection.contains_id(id) {
            collection.push(serde_json::from_value(self.properties.clone())?)?;
            true
        } else {
            false
        };

        let rule_applied = self.regroup(id, collection, report, update)?;
        if rule_applied {
            collection.retain(|object| {
                !self
                    .grouped_from
                    .iter()
                    .any(|grouped_id| object.id() == grouped_id)
            });
        } else if created {
            report.add_warning(
                    format!(
                        "Object with {} \"{}\" was created but must be used (through properties_rules) or else it will be deleted",
                        id_key,
                        id
                    ),
                    ReportCategory::UnknownPropertyValue,
                );
        } else {
            report.add_error(
                format!("The rule on {} \"{}\" was not applied", id_key, id),
                ReportCategory::ObjectNotFound,
            );
        }
        Ok(())
    }
}

impl ObjectRule {
    pub(crate) fn apply_rules(
        &self,
        collections: &mut Collections,
        report: &mut Report<ReportCategory>,
    ) -> Result<()> {
        if let (
            Some(networks_rules),
            Some(lines_by_network),
            Some(ticket_use_perimeters_by_network),
        ) = (
            &self.configuration.networks_rules,
            &self.lines_by_network,
            &self.ticket_use_perimeters_by_network,
        ) {
            info!("Checking networks rules.");
            for rule in networks_rules {
                let lines = &mut collections.lines;
                let ticket_use_perimeters = &mut collections.ticket_use_perimeters;
                let regroup_update = |network_id: &str, removed_id: &str| {
                    let lines_modified =
                        if let Some(line_indexes) = lines_by_network.get(removed_id) {
                            for line_idx in line_indexes {
                                lines.index_mut(*line_idx).network_id = network_id.to_string();
                            }
                            true
                        } else {
                            false
                        };
                    let tup_modified = if let Some(tup_indexes) =
                        ticket_use_perimeters_by_network.get(removed_id)
                    {
                        for tup_idx in tup_indexes {
                            ticket_use_perimeters.index_mut(*tup_idx).object_id =
                                network_id.to_string();
                        }
                        true
                    } else {
                        false
                    };
                    lines_modified || tup_modified
                };
                rule.apply(
                    "network_id",
                    &mut collections.networks,
                    report,
                    regroup_update,
                )?;
            }
        };
        if let (Some(commercial_modes_rules), Some(lines_by_commercial_mode)) = (
            &self.configuration.commercial_modes_rules,
            &self.lines_by_commercial_mode,
        ) {
            info!("Checking commercial modes rules.");
            for rule in commercial_modes_rules {
                let lines = &mut collections.lines;
                let regroup_update = |commercial_mode_id: &str, removed_id: &str| {
                    if let Some(line_indexes) = lines_by_commercial_mode.get(removed_id) {
                        for line_idx in line_indexes {
                            lines.index_mut(*line_idx).commercial_mode_id =
                                commercial_mode_id.to_string();
                        }
                        true
                    } else {
                        false
                    }
                };
                rule.apply(
                    "commercial_mode_id",
                    &mut collections.commercial_modes,
                    report,
                    regroup_update,
                )?;
            }
        };
        if let (Some(physical_modes_rules), Some(vjs_by_physical_mode)) = (
            &self.configuration.physical_modes_rules,
            &self.vjs_by_physical_mode,
        ) {
            info!("Checking physical modes rules.");
            for rule in physical_modes_rules {
                let vehicle_journeys = &mut collections.vehicle_journeys;
                let regroup_update = |physical_mode_id: &str, removed_id: &str| {
                    if let Some(vehicle_journey_indexes) = vjs_by_physical_mode.get(removed_id) {
                        for vehicle_journey_idx in vehicle_journey_indexes {
                            vehicle_journeys
                                .index_mut(*vehicle_journey_idx)
                                .physical_mode_id = physical_mode_id.to_string();
                        }
                        true
                    } else {
                        false
                    }
                };
                rule.apply(
                    "physical_mode_id",
                    &mut collections.physical_modes,
                    report,
                    regroup_update,
                )?;
            }
        };
        Ok(())
    }
}
