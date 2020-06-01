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

use assert_cmd::prelude::*;
use lazy_static::lazy_static;
use pretty_assertions::assert_eq;
use std::{
    path::{Path, PathBuf},
    process::Command,
};
use tempfile::TempDir;
use transit_model::test_utils::*;

lazy_static! {
    static ref FILE_TO_COMPARE: std::vec::Vec<&'static str> = {
        vec![
            "commercial_modes.txt",
            "equipments.txt",
            "geometries.txt",
            "lines.txt",
            "networks.txt",
            "physical_modes.txt",
            "routes.txt",
            "stops.txt",
            "ticket_use_perimeters.txt",
            "trips.txt",
            "trip_properties.txt",
        ]
    };
}

fn compare_report(report_path: PathBuf, fixture_report_output: PathBuf) {
    let output_contents = get_file_content(report_path);
    let expected_contents = get_file_content(fixture_report_output);
    assert_eq!(expected_contents, output_contents);
}

fn test_apply_rules(
    cc_rules_dir: &str,
    p_rules_dir: &str,
    n_consolidation: &str,
    fixture_output_dir: &str,
    fixture_report_output: &str,
    mut file_to_compare: Vec<&str>,
) {
    let output_dir = TempDir::new().expect("create temp dir failed");
    let report_path = output_dir.path().join("report.json");
    let mut command =
        Command::cargo_bin("apply-rules").expect("Failed to find binary 'apply-rules'");
    let command = command
        .arg("--input")
        .arg("tests/fixtures/input")
        .arg("--output")
        .arg(output_dir.path().to_str().unwrap())
        .arg("--report")
        .arg(report_path.as_path());
    let command = if !cc_rules_dir.is_empty() {
        file_to_compare.push("object_codes.txt");
        command
            .arg("--complementary-code-rules")
            .arg(Path::new(cc_rules_dir))
    } else {
        command
    };
    let command = if !p_rules_dir.is_empty() {
        command.arg("--property-rules").arg(Path::new(p_rules_dir))
    } else {
        command
    };
    let command = if !n_consolidation.is_empty() {
        command
            .arg("--networks-consolidation")
            .arg(Path::new(n_consolidation))
    } else {
        command
    };
    command.assert().success();
    compare_output_dir_with_expected(&output_dir, Some(file_to_compare), fixture_output_dir);
    compare_report(report_path, Path::new(fixture_report_output).to_path_buf());
}

#[test]
fn test_no_property_rules() {
    test_apply_rules(
        "",
        "",
        "",
        "./tests/fixtures/output",
        "./tests/fixtures/output_report/report.json",
        FILE_TO_COMPARE.clone(),
    );
}

#[test]
fn test_apply_complementary_codes() {
    test_apply_rules(
        "./tests/fixtures/complementary_codes_rules.txt",
        "",
        "",
        "./tests/fixtures/output_apply_complementary_codes",
        "./tests/fixtures/output_report/report_apply_complementary_codes.json",
        FILE_TO_COMPARE.clone(),
    );
}

#[test]
fn test_apply_property() {
    test_apply_rules(
        "./tests/fixtures/complementary_codes_rules.txt",
        "./tests/fixtures/property_rules.txt",
        "",
        "./tests/fixtures/output_apply_property",
        "./tests/fixtures/output_report/report_apply_property.json",
        FILE_TO_COMPARE.clone(),
    );
}

#[test]
fn test_ntw_consolidation() {
    test_apply_rules(
        "",
        "",
        "./tests/fixtures/ntw_consolidation.json",
        "./tests/fixtures/output_ntw_consolidation",
        "./tests/fixtures/output_report/report.json",
        vec!["lines.txt", "networks.txt"],
    );
}

#[test]
#[should_panic]
fn test_ntw_consolidation_unvalid() {
    test_apply_rules(
        "",
        "",
        "./tests/fixtures/ntw_consolidation_unvalid.json",
        "",
        "",
        vec![],
    );
}

#[test]
fn test_ntw_consolidation_with_object_code() {
    test_apply_rules(
        "./tests/fixtures/complementary_codes_rules.txt",
        "./tests/fixtures/property_rules.txt",
        "./tests/fixtures/ntw_consolidation.json",
        "./tests/fixtures/output_consolidation_with_object_code",
        "./tests/fixtures/output_report/report_consolidation_with_object_code.json",
        FILE_TO_COMPARE.clone(),
    );
}

#[test]
fn test_ntw_consolidation_2_ntw() {
    test_apply_rules(
        "",
        "",
        "./tests/fixtures/ntw_consolidation_2_ntw.json",
        "./tests/fixtures/output_consolidation_2_ntw",
        "./tests/fixtures/output_report/report.json",
        vec!["lines.txt", "networks.txt"],
    );
}

#[test]
fn test_ntw_consolidation_2_diff_ntw() {
    test_apply_rules(
        "",
        "",
        "./tests/fixtures/ntw_consolidation_2_diff_ntw.json",
        "./tests/fixtures/output_consolidation_2_diff_ntw",
        "./tests/fixtures/output_report/report.json",
        vec!["lines.txt", "networks.txt"],
    );
}

#[test]
fn test_ntw_consolidation_unknown_id() {
    test_apply_rules(
        "",
        "",
        "./tests/fixtures/ntw_consolidation_unknown_id.json",
        "./tests/fixtures/output",
        "./tests/fixtures/output_report/report_consolidation_unknown_id.json",
        vec!["lines.txt", "networks.txt"],
    );
}

#[test]
#[should_panic]
fn test_ntw_consolidation_duplicate_id() {
    test_apply_rules(
        "",
        "",
        "./tests/fixtures/ntw_consolidation_duplicate_id.json",
        "",
        "",
        vec![],
    );
}

#[test]
#[should_panic]
fn test_ntw_consolidation_unvalid_network() {
    test_apply_rules(
        "",
        "",
        "./tests/fixtures/ntw_consolidation_unvalid_network.json",
        "",
        "",
        vec![],
    );
}

#[test]
fn test_ntw_consolidation_no_grouped_from() {
    test_apply_rules(
        "",
        "",
        "./tests/fixtures/ntw_consolidation_no_grouped_from.json",
        "./tests/fixtures/output_update_network",
        "./tests/fixtures/output_report/report_consolidation_empty_no_grouped_from.json",
        vec!["lines.txt", "networks.txt"],
    );
}

#[test]
fn test_ntw_consolidation_empty_grouped_from() {
    test_apply_rules(
        "",
        "",
        "./tests/fixtures/ntw_consolidation_empty_grouped_from.json",
        "./tests/fixtures/output_update_network",
        "./tests/fixtures/output_report/report_consolidation_empty_no_grouped_from.json",
        vec!["lines.txt", "networks.txt"],
    );
}

#[test]
fn test_commercial_mode_consolidation() {
    test_apply_rules(
        "",
        "",
        "./tests/fixtures/commercial_mode_consolidation.json",
        "./tests/fixtures/output_commercial_mode_consolidation",
        "./tests/fixtures/output_report/report.json",
        vec!["lines.txt", "commercial_modes.txt"],
    );
}

#[test]
fn test_physical_mode_consolidation() {
    test_apply_rules(
        "",
        "",
        "./tests/fixtures/physical_mode_consolidation.json",
        "./tests/fixtures/output_physical_mode_consolidation",
        "./tests/fixtures/output_report/report.json",
        vec!["trips.txt", "physical_modes.txt"],
    );
}

#[test]
fn test_global_consolidation() {
    test_apply_rules(
        "",
        "",
        "./tests/fixtures/global_consolidation.json",
        "./tests/fixtures/output_global_consolidation",
        "./tests/fixtures/output_report/report.json",
        vec![
            "lines.txt",
            "networks.txt",
            "commercial_modes.txt",
            "trips.txt",
            "physical_modes.txt",
        ],
    );
}

#[test]
fn test_global_consolidation_with_new_objects() {
    test_apply_rules(
        "",
        "./tests/fixtures/property_rules_with_new_objects.txt",
        "./tests/fixtures/global_consolidation_with_new_objects.json",
        "./tests/fixtures/output_global_consolidation",
        "./tests/fixtures/output_report/report_global_consolidation_with_new_objects.json",
        vec![
            "lines.txt",
            "networks.txt",
            "commercial_modes.txt",
            "trips.txt",
            "physical_modes.txt",
        ],
    );
}
