use super::ReportCategory;
use derivative::Derivative;
use failure::ResultExt;
use log::info;
use serde::Deserialize;
use std::{
    collections::{BTreeSet, HashMap, HashSet},
    fs,
    path::{Path, PathBuf},
};
use tartare_tools::report::Report;
/// transfers rules
use transit_model::{
    objects::{Contributor, StopArea, StopPoint, Transfer},
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

#[derive(Debug, Deserialize)]
#[serde(rename_all = "snake_case")]
enum StopType {
    StopPoint,
    StopArea,
}

#[derivative(Default)]
#[derive(Derivative, Deserialize, Debug)]
#[serde(default)]
struct Rule {
    #[derivative(Default(value = "StopType::StopPoint"))]
    from_stop_type: StopType,
    from_stop_id: String,
    #[derivative(Default(value = "StopType::StopPoint"))]
    to_stop_type: StopType,
    to_stop_id: String,
    transfer_time: Option<u32>,
    waiting_time: Option<u32>,
}

impl Rule {
    // Expand a `Rule` about StopAreas into all combinations of StopPoints
    fn expand(self, model: &Model, report: &mut Report<ReportCategory>) -> Vec<Rule> {
        fn get_stop_point_ids<'a>(
            stop_type: StopType,
            stop_id: &'a str,
            model: &'a Model,
            report: &mut Report<ReportCategory>,
        ) -> Vec<&'a str> {
            match stop_type {
                StopType::StopPoint => vec![stop_id],
                StopType::StopArea => {
                    if let Some(stop_area_idx) = model.stop_areas.get_idx(stop_id) {
                        model
                            .get_corresponding_from_idx::<StopArea, StopPoint>(stop_area_idx)
                            .iter()
                            .map(|stop_point_idx| model.stop_points[*stop_point_idx].id.as_str())
                            .collect()
                    } else {
                        report.add_warning(
                            format!(
                                "manual transfer references a non-existent stop area ({})",
                                stop_id
                            ),
                            ReportCategory::OnNonExistentStop,
                        );
                        Vec::new()
                    }
                }
            }
        }
        let from_stop_points = get_stop_point_ids(
            self.from_stop_type,
            self.from_stop_id.as_str(),
            model,
            report,
        );
        let to_stop_points =
            get_stop_point_ids(self.to_stop_type, self.to_stop_id.as_str(), model, report);
        let mut rules = Vec::new();
        for from_stop_point in &from_stop_points {
            for to_stop_point in &to_stop_points {
                rules.push(Rule {
                    from_stop_type: StopType::StopPoint,
                    from_stop_id: from_stop_point.to_string(),
                    to_stop_type: StopType::StopPoint,
                    to_stop_id: to_stop_point.to_string(),
                    transfer_time: self.transfer_time,
                    waiting_time: self.waiting_time,
                });
            }
        }
        rules
    }
}

pub fn apply_rules<P: AsRef<Path>>(
    model: Model,
    waiting_time: u32,
    only_inter: bool,
    rule_files: Vec<P>,
    report_path: Option<PathBuf>,
) -> Result<Model> {
    let mut transfers_map = transfers_map(&model, model.transfers.clone());
    let mut report = Report::default();
    let rules = read_rules(rule_files, &model, only_inter, &mut report)?;

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

fn transfers_map(model: &Model, transfers: Collection<Transfer>) -> TransferMap {
    transfers
        .into_iter()
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

pub fn stop_points_need_transfer(
    model: &Model,
    from_idx: Idx<StopPoint>,
    to_idx: Idx<StopPoint>,
    only_inter: bool,
    report_opt: Option<&mut Report<ReportCategory>>,
) -> bool {
    if !only_inter {
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

    from_contributor != to_contributor
}

fn check_and_insert_rule(
    rules: &mut HashMap<(Idx<StopPoint>, Idx<StopPoint>), Rule>,
    rule: Rule,
    model: &Model,
    only_inter: bool,
    report: &mut Report<ReportCategory>,
) {
    match (
        model.stop_points.get_idx(&rule.from_stop_id),
        model.stop_points.get_idx(&rule.to_stop_id),
    ) {
        (Some(from), Some(to)) => {
            if stop_points_need_transfer(model, from, to, only_inter, Some(report)) {
                // Last arrived rule is the winner, therefore, we remove and
                // report the existing one.
                if let Some(r) = rules.remove(&(from, to)) {
                    report.add_warning(
                        format!(
                            "transfer between stop point {} (stop area {}) and stop point {} (stop area {}) is ignored",
                            r.from_stop_id,
                            model.stop_points[from].stop_area_id,
                            r.to_stop_id,
                            model.stop_points[to].stop_area_id,
                        ),
                        ReportCategory::Ignored,
                    );
                }
                rules.insert((from, to), rule);
            } else {
                report.add_warning(
                    format!(
                        "transfer between stop point {} (stop area {}) and stop point {} (stop area {}) is ignored",
                        rule.from_stop_id,
                        model.stop_points[from].stop_area_id,
                        rule.to_stop_id,
                        model.stop_points[to].stop_area_id,
                    ),
                    ReportCategory::Ignored,
                );
            }
        }
        (Some(_), None) => {
            report.add_warning(
                format!(
                    "manual transfer references a non-existent stop point ({})",
                    rule.to_stop_id
                ),
                ReportCategory::OnNonExistentStop,
            );
        }
        (None, Some(_)) => {
            report.add_warning(
                format!(
                    "manual transfer references a non-existent stop point ({})",
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

fn read_rules<P: AsRef<Path>>(
    rule_files: Vec<P>,
    model: &Model,
    only_inter: bool,
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
            let stop_point_rules = rule.expand(model, report);
            for rule in stop_point_rules {
                check_and_insert_rule(&mut rules, rule, model, only_inter, report);
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
        // Time of walking, does not include the waiting time
        let min_transfer_time = r.transfer_time;
        // Total transfer time (including walking time and waiting time)
        let real_min_transfer_time = r
            .transfer_time
            .map(|t| t + r.waiting_time.unwrap_or(waiting_time));
        transfers_map
            .entry((
                stop_points.get_idx(&r.from_stop_id).unwrap(),
                stop_points.get_idx(&r.to_stop_id).unwrap(),
            ))
            .and_modify(|t| {
                t.min_transfer_time = min_transfer_time;
                t.real_min_transfer_time = real_min_transfer_time;
            })
            .or_insert_with(|| Transfer {
                from_stop_id: r.from_stop_id.clone(),
                to_stop_id: r.to_stop_id.clone(),
                min_transfer_time,
                real_min_transfer_time,
                equipment_id: None,
            });
    }
}
