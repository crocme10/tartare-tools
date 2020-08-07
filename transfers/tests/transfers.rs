use assert_cmd::prelude::*;
use std::{path::Path, process::Command};
use tempfile::TempDir;
use transfers::transfers;
use transit_model::test_utils::*;

#[test]
fn test_generates_all_transfers() {
    test_in_tmp_dir(|path| {
        let input_dir = "./tests/fixtures/input";
        let model = transit_model::ntfs::read(input_dir).unwrap();
        let rules: Vec<Box<Path>> = vec![];

        let model = transfers(model, 100.0, 0.785, 120, false, rules, None).unwrap();

        transit_model::ntfs::write(&model, path, get_test_datetime()).unwrap();
        compare_output_dir_with_expected(
            &path,
            Some(vec!["transfers.txt"]),
            "./tests/fixtures/output_all",
        );
    });
}

#[test]
fn test_generates_transfers_inter_contributors() {
    test_in_tmp_dir(|path| {
        let input_dir = "./tests/fixtures/input";
        let model = transit_model::ntfs::read(input_dir).unwrap();
        let rules: Vec<Box<Path>> = vec![];

        let model = transfers(model, 100.0, 0.785, 120, true, rules, None).unwrap();

        transit_model::ntfs::write(&model, path, get_test_datetime()).unwrap();
        compare_output_dir_with_expected(
            &path,
            Some(vec!["transfers.txt"]),
            "./tests/fixtures/output_inter_contributors",
        );
    });
}

#[test]
fn test_generates_transfers_with_modification_rules() {
    test_in_tmp_dir(|path| {
        let input_dir = "tests/fixtures/input";
        let model = transit_model::ntfs::read(input_dir).unwrap();
        let rules = vec![Path::new("./tests/fixtures/rules.txt").to_path_buf()];
        let report_path = path.join("report.json");

        let model = transfers(model, 100.0, 0.785, 120, false, rules, Some(report_path)).unwrap();

        transit_model::ntfs::write(&model, path, get_test_datetime()).unwrap();
        compare_output_dir_with_expected(
            &path,
            Some(vec!["transfers.txt", "report.json"]),
            "./tests/fixtures/output_rules",
        );
    });
}

#[test]
fn test_binary_generates_all_transfers_with_default_parameters() {
    let output_dir = TempDir::new().expect("create temp dir failed");
    Command::cargo_bin("transfers")
        .expect("Failed to find binary 'transfers'")
        .arg("--input")
        .arg("tests/fixtures/input/")
        .arg("--output")
        .arg(output_dir.path().to_str().unwrap())
        .assert()
        .success();
    compare_output_dir_with_expected(
        &output_dir,
        Some(vec!["transfers.txt"]),
        "tests/fixtures/output_default",
    );
}

#[test]
fn test_binary_generates_all_transfers_with_rules() {
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
        "tests/fixtures/output_all_with_rules",
    );
}

#[test]
fn test_binary_generates_inter_contributors_transfers() {
    let output_dir = TempDir::new().expect("create temp dir failed");
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
        .arg("--inter-contributors-transfers-only")
        .arg("--output")
        .arg(output_dir.path().to_str().unwrap())
        .assert()
        .success();
    compare_output_dir_with_expected(
        &output_dir,
        Some(vec!["transfers.txt"]),
        "tests/fixtures/output_inter_contributors",
    );
}
