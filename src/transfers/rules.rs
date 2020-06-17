// Copyright 2017 Kisio Digital and/or its affiliates.
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

use super::ReportCategory;
use crate::report::Report;
use failure::ResultExt;
use log::info;
use serde::Deserialize;
use std::{
    collections::{hash_map::Entry::*, BTreeSet, HashMap, HashSet},
    fs,
    path::{Path, PathBuf},
};
/// transfers rules
use transit_model::{
    objects::{Contributor, StopPoint, Transfer},
    Model, Result,
};
use typed_index_collection::{Collection, CollectionWithId, Idx};

type TransferMap = HashMap<(Idx<StopPoint>, Idx<StopPoint>), Transfer>;

/// Represents the type of transfers to generate
#[derive(PartialEq, Debug)]
pub enum TransfersMode {
    /// `All` will generate all transfers
    All,
    /// `IntraContributor` will generate transfers between stop points belonging to the
    /// same contributor
    IntraContributor,
    /// `InterContributor` will generate transfers between stop points belonging to
    /// differents contributors only
    InterContributor,
}

#[derive(Deserialize, Debug)]
struct Rule {
    from_stop_id: String,
    to_stop_id: String,
    transfer_time: Option<u32>,
}

/// Filter and add/remove transfers.
pub fn apply_rules<P: AsRef<Path>>(
    model: Model,
    waiting_time: u32,
    rule_files: Vec<P>,
    transfers_mode: &TransfersMode,
    report_path: Option<PathBuf>,
) -> Result<Model> {
    let mut transfers_map = filtered_transfers_map(&model, model.transfers.clone(), transfers_mode);
    let mut report = Report::default();
    let rules = read_rules(rule_files, &model, transfers_mode, &mut report)?;
    if !rules.is_empty() {
        remove_unwanted_transfers(&mut transfers_map, &model.stop_points, &rules);
        add_missing_transfers(&mut transfers_map, &model.stop_points, &rules, waiting_time);
    }
    if let Some(report_path) = report_path {
        let serialized_report = serde_json::to_string(&report)?;
        fs::write(report_path, serialized_report)?;
    }

    let mut new_transfers: Vec<_> = transfers_map.into_iter().map(|(_, v)| v).collect();
    new_transfers.sort_unstable_by(|t1, t2| {
        (&t1.from_stop_id, &t1.to_stop_id).cmp(&(&t2.from_stop_id, &t2.to_stop_id))
    });

    let mut collections = model.into_collections();
    collections.transfers = Collection::new(new_transfers);
    Ok(Model::new(collections)?)
}

fn filtered_transfers_map(
    model: &Model,
    transfers: Collection<Transfer>,
    transfers_mode: &TransfersMode,
) -> TransferMap {
    transfers
        .into_iter()
        .filter(|t| {
            stop_points_need_transfer(
                model,
                model.stop_points.get_idx(&t.from_stop_id).unwrap(),
                model.stop_points.get_idx(&t.to_stop_id).unwrap(),
                transfers_mode,
                None,
            )
        })
        .map(|t| {
            (
                (
                    model.stop_points.get_idx(&t.from_stop_id).unwrap(),
                    model.stop_points.get_idx(&t.to_stop_id).unwrap(),
                ),
                t,
            )
        })
        .collect()
}

fn stop_points_need_transfer(
    model: &Model,
    from_idx: Idx<StopPoint>,
    to_idx: Idx<StopPoint>,
    transfers_mode: &TransfersMode,
    report_opt: Option<&mut Report<ReportCategory>>,
) -> bool {
    if *transfers_mode == TransfersMode::All {
        return true;
    }
    let from_contributor: BTreeSet<Idx<Contributor>> = model.get_corresponding_from_idx(from_idx);
    let to_contributor: BTreeSet<Idx<Contributor>> = model.get_corresponding_from_idx(to_idx);

    if from_contributor.is_empty() {
        if let Some(report) = report_opt {
            report.add_warning(
                format!(
                    "stop point {} belongs to none of the trips and will not generate any transfer",
                    model.stop_points[from_idx].id
                ),
                ReportCategory::OnUnreferencedStop,
            );
        }
        return false;
    }
    if to_contributor.is_empty() {
        if let Some(report) = report_opt {
            report.add_warning(
                format!(
                    "stop point {} belongs to none of the trips and will not generate any transfer",
                    model.stop_points[to_idx].id
                ),
                ReportCategory::OnUnreferencedStop,
            );
        }
        return false;
    }

    match *transfers_mode {
        TransfersMode::All => unreachable!(),
        TransfersMode::IntraContributor => from_contributor == to_contributor,
        TransfersMode::InterContributor => from_contributor != to_contributor,
    }
}

fn read_rules<P: AsRef<Path>>(
    rule_files: Vec<P>,
    model: &Model,
    transfers_mode: &TransfersMode,
    report: &mut Report<ReportCategory>,
) -> Result<Vec<Rule>> {
    info!("Reading modificaton rules.");
    let mut rules = HashMap::new();
    for rule_path in rule_files {
        let path = rule_path.as_ref();
        let mut rdr =
            csv::Reader::from_path(&path).with_context(|_| format!("Error reading {:?}", path))?;

        for rule in rdr.deserialize() {
            let rule: Rule = rule.with_context(|_| format!("Error reading {:?}", path))?;
            match (
                model.stop_points.get_idx(&rule.from_stop_id),
                model.stop_points.get_idx(&rule.to_stop_id),
            ) {
                (Some(from), Some(to)) => {
                    if stop_points_need_transfer(model, from, to, transfers_mode, Some(report)) {
                        match rules.entry((from, to)) {
                            Occupied(_) => report.add_warning(
                                format!(
                                    "transfer between stops {} and {} is already declared",
                                    rule.from_stop_id, rule.to_stop_id
                                ),
                                ReportCategory::AlreadyDeclared,
                            ),
                            Vacant(v) => {
                                v.insert(rule);
                            }
                        }
                    } else {
                        let category = match *transfers_mode {
                            TransfersMode::IntraContributor => ReportCategory::InterIgnored,
                            TransfersMode::InterContributor => ReportCategory::IntraIgnored,
                            TransfersMode::All => ReportCategory::AllIgnored,
                        };
                        report.add_warning(
                            format!(
                                "transfer between stops {} and {} is ignored",
                                rule.from_stop_id, rule.to_stop_id
                            ),
                            category,
                        );
                    }
                }
                (Some(_), None) => {
                    report.add_warning(
                        format!(
                            "manual transfer references an non-existent stop point ({})",
                            rule.to_stop_id
                        ),
                        ReportCategory::OnNonExistentStop,
                    );
                }
                (None, Some(_)) => {
                    report.add_warning(
                        format!(
                            "manual transfer references an non-existent stop point ({})",
                            rule.from_stop_id
                        ),
                        ReportCategory::OnNonExistentStop,
                    );
                }
                _ => {
                    report.add_warning(
                        format!(
                            "manual transfer references non-existent stop points ({} and {})",
                            rule.from_stop_id, rule.to_stop_id
                        ),
                        ReportCategory::OnNonExistentStop,
                    );
                }
            }
        }
    }
    Ok(rules.into_iter().map(|(_, rule)| rule).collect())
}

fn remove_unwanted_transfers(
    transfers_map: &mut TransferMap,
    stop_points: &CollectionWithId<StopPoint>,
    rules: &[Rule],
) {
    info!("Removing unwanted transfers.");
    let rules_to_remove: HashSet<(Idx<StopPoint>, Idx<StopPoint>)> = rules
        .iter()
        .map(|r| {
            (
                stop_points.get_idx(&r.from_stop_id).unwrap(),
                stop_points.get_idx(&r.to_stop_id).unwrap(),
            )
        })
        .collect();
    transfers_map.retain(|_, t| {
        !rules_to_remove.contains(&(
            stop_points.get_idx(&t.from_stop_id).unwrap(),
            stop_points.get_idx(&t.to_stop_id).unwrap(),
        ))
    });
}

fn add_missing_transfers(
    transfers_map: &mut TransferMap,
    stop_points: &CollectionWithId<StopPoint>,
    rules: &[Rule],
    waiting_time: u32,
) {
    info!("Adding missing transfers.");
    for r in rules.iter().filter(|r| r.transfer_time.is_some()) {
        transfers_map
            .entry((
                stop_points.get_idx(&r.from_stop_id).unwrap(),
                stop_points.get_idx(&r.to_stop_id).unwrap(),
            ))
            .and_modify(|t| {
                t.min_transfer_time = r.transfer_time;
                t.real_min_transfer_time = r.transfer_time;
            })
            .or_insert_with(|| Transfer {
                from_stop_id: r.from_stop_id.clone(),
                to_stop_id: r.to_stop_id.clone(),
                min_transfer_time: r.transfer_time,
                real_min_transfer_time: r.transfer_time.map(|t| t + waiting_time),
                equipment_id: None,
            });
    }
}
