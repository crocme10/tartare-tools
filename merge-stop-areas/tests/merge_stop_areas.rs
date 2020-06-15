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
fn test_merge_stop_areas_multi_steps() {
    let output_dir = TempDir::new().expect("create temp dir failed");
    let report_path = output_dir.path().join("report.json");
    Command::cargo_bin("merge-stop-areas")
        .expect("Failed to find binary 'merge-stop-areas'")
        .arg("--input")
        .arg("tests/fixtures/ntfs-to-merge")
        .arg("--output")
        .arg(output_dir.path().to_str().unwrap())
        .arg("--config")
        .arg("tests/fixtures/rule1.csv")
        .arg("--config")
        .arg("tests/fixtures/rule2.csv")
        .arg("--report")
        .arg(report_path.as_path())
        .arg("--distance")
        .arg("200")
        .arg("--current-datetime")
        .arg("2019-04-03T17:19:00+00:00")
        .assert()
        .success();
    compare_output_dir_with_expected(
        &output_dir,
        Some(vec![
            "comment_links.txt",
            "comments.txt",
            "geometries.txt",
            "feed_infos.txt",
            "lines.txt",
            "object_codes.txt",
            "object_properties.txt",
            "stops.txt",
            "ticket_use_restrictions.txt",
            "report.json",
            "routes.txt",
        ]),
        "./tests/fixtures/output",
    );
}
