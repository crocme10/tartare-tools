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
use std::process::Command;
use tempfile::TempDir;
use transit_model::test_utils::*;

#[test]
fn test_generates_transfers_intra_contributors() {
    let output_dir = TempDir::new().expect("create temp dir failed");
    let report_path = output_dir.path().join("report.json");
    Command::cargo_bin("transfers")
        .expect("Failed to find binary 'transfers'")
        .arg("--input")
        .arg("tests/fixtures/input/")
        .arg("--max-distance")
        .arg("100.0")
        .arg("--walking-speed")
        .arg("0.785")
        .arg("--waiting-time")
        .arg("120")
        .arg("--rules-file")
        .arg("tests/fixtures/rules.txt")
        .arg("--report")
        .arg(report_path.to_str().unwrap())
        .arg("--output")
        .arg(output_dir.path().to_str().unwrap())
        .assert()
        .success();
    compare_output_dir_with_expected(
        &output_dir,
        Some(vec!["transfers.txt", "report.json"]),
        "tests/fixtures/output",
    );
}
