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

//! See function apply_rules

mod complementary_code;
mod object_rule;
mod property_rule;

use log::info;
use serde::Serialize;
use std::{fs, path::PathBuf};
use tartare_tools::report::{self, Report};
use transit_model::{Model, Result};

#[derive(Debug, Serialize, PartialEq)]
pub enum ReportCategory {
    ObjectNotFound,
    InvalidFile,
    UnknownPropertyName,
    UnknownPropertyValue,
    MultipleValue,
    OldPropertyValueDoesNotMatch,
    GeometryNotValid,
    NonConvertibleString,
}

impl report::ReportCategory for ReportCategory {}

/// Applying rules
///
/// - `complementary_code_rules_files` Csv files containing codes to add for certain objects
/// - `property_rules_files` Csv files containing rules applied on properties
/// - `object_rules_file` Json file containing rules for grouping objects
pub fn apply_rules(
    model: Model,
    complementary_code_rules_files: Vec<PathBuf>,
    property_rules_files: Vec<PathBuf>,
    object_rules_file: Option<PathBuf>,
    report_path: PathBuf,
) -> Result<Model> {
    let object_rule = object_rules_file
        .map(|path| object_rule::ObjectRule::new(path.as_path(), &model))
        .transpose()?;

    let mut collections = model.into_collections();
    let mut report = Report::default();
    if let Some(object_rule) = object_rule {
        info!("Applying object rules");
        object_rule.apply_rules(&mut collections, &mut report)?;
    }

    info!("Applying complementary code rules");
    complementary_code::apply_rules(
        complementary_code_rules_files,
        &mut collections,
        &mut report,
    )?;

    info!("Applying property rules");
    property_rule::apply_rules(property_rules_files, &mut collections, &mut report)?;

    let serialized_report = serde_json::to_string_pretty(&report)?;
    fs::write(report_path, serialized_report)?;

    Model::new(collections)
}
