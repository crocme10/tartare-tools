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

use std::path::Path;
use tartare_tools::transfers::{transfers, TransfersMode};
use transit_model::test_utils::*;

#[test]
fn test_generates_all_transfers() {
    test_in_tmp_dir(|path| {
        let input_dir = "./tests/fixtures/transfers/input";
        let model = transit_model::ntfs::read(input_dir).unwrap();
        let rules: Vec<Box<Path>> = vec![];

        let model = transfers(model, 100.0, 0.785, 120, &TransfersMode::All, rules, None).unwrap();

        transit_model::ntfs::write(&model, path, get_test_datetime()).unwrap();
        compare_output_dir_with_expected(
            &path,
            Some(vec!["transfers.txt"]),
            "./tests/fixtures/transfers/output_all",
        );
    });
}

#[test]
fn test_generates_transfers_intra_contributors() {
    test_in_tmp_dir(|path| {
        let input_dir = "./tests/fixtures/transfers/input";
        let model = transit_model::ntfs::read(input_dir).unwrap();
        let rules: Vec<Box<Path>> = vec![];

        let model = transfers(
            model,
            100.0,
            0.785,
            120,
            &TransfersMode::IntraContributor,
            rules,
            None,
        )
        .unwrap();

        transit_model::ntfs::write(&model, path, get_test_datetime()).unwrap();
        compare_output_dir_with_expected(
            &path,
            Some(vec!["transfers.txt"]),
            "./tests/fixtures/transfers/output_intra_contributors",
        );
    });
}

#[test]
fn test_generates_transfers_inter_contributors() {
    test_in_tmp_dir(|path| {
        let input_dir = "./tests/fixtures/transfers/input";
        let model = transit_model::ntfs::read(input_dir).unwrap();
        let rules: Vec<Box<Path>> = vec![];

        let model = transfers(
            model,
            100.0,
            0.785,
            120,
            &TransfersMode::InterContributor,
            rules,
            None,
        )
        .unwrap();

        transit_model::ntfs::write(&model, path, get_test_datetime()).unwrap();
        compare_output_dir_with_expected(
            &path,
            Some(vec!["transfers.txt"]),
            "./tests/fixtures/transfers/output_inter_contributors",
        );
    });
}

#[test]
fn test_generates_transfers_with_modification_rules() {
    test_in_tmp_dir(|path| {
        let input_dir = "tests/fixtures/transfers/input";
        let model = transit_model::ntfs::read(input_dir).unwrap();
        let rules = vec![Path::new("./tests/fixtures/transfers/rules.txt").to_path_buf()];
        let report_path = path.join("report.json");

        let model = transfers(
            model,
            100.0,
            0.785,
            120,
            &TransfersMode::All,
            rules,
            Some(report_path),
        )
        .unwrap();

        transit_model::ntfs::write(&model, path, get_test_datetime()).unwrap();
        compare_output_dir_with_expected(
            &path,
            Some(vec!["transfers.txt", "report.json"]),
            "./tests/fixtures/transfers/output_rules",
        );
    });
}
