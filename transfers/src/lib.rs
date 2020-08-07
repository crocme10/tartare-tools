mod rules;

pub use rules::TransfersMode;
use serde::Serialize;
use std::path::{Path, PathBuf};
use tartare_tools::report;
use transit_model::{transfers::generates_transfers, Model, Result};

/// Type of the report
#[derive(Debug, Serialize, PartialEq)]
pub enum ReportCategory {
    Ignored,
    OnUnreferencedStop,
    OnNonExistentStop,
    AlreadyDeclared,
}

impl report::ReportCategory for ReportCategory {}

/// Generates transfers
///
/// 1. Generates missing transfers
/// 2. Adds/removes transfers with rules files
pub fn transfers<P: AsRef<Path>>(
    model: Model,
    max_distance: f64,
    walking_speed: f64,
    waiting_time: u32,
    only_inter: bool,
    rule_files: Vec<P>,
    report_path: Option<PathBuf>,
) -> Result<Model> {
    let need_transfer = Box::new(|model: &Model, from_idx, to_idx| -> bool {
        rules::stop_points_need_transfer(model, from_idx, to_idx, only_inter, None)
    });

    let model = generates_transfers(
        model,
        max_distance,
        walking_speed,
        waiting_time,
        Some(need_transfer),
    )?;

    let model = rules::apply_rules(model, waiting_time, only_inter, rule_files, report_path)?;

    Ok(model)
}
