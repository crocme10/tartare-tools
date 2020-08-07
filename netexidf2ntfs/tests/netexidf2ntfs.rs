use assert_cmd::prelude::*;
use std::process::Command;
use tempfile::TempDir;
use transit_model::test_utils::*;

#[test]
fn test_netexidf2ntfs() {
    let output_dir = TempDir::new().expect("create temp dir failed");
    Command::cargo_bin("netexidf2ntfs")
        .expect("Failed to find binary 'netexidf2ntfs'")
        .arg("--input")
        .arg("tests/fixtures/input/")
        .arg("--output")
        .arg(output_dir.path().to_str().unwrap())
        .arg("--config")
        .arg("tests/fixtures/config.json")
        .arg("--prefix")
        .arg("prefix")
        .arg("--current-datetime")
        .arg("2019-04-03T17:19:00+00:00")
        .arg("--max-distance")
        .arg("200")
        .arg("--walking-speed")
        .arg("0.8")
        .arg("--waiting-time")
        .arg("50")
        .assert()
        .success();
    compare_output_dir_with_expected(&output_dir, None, "tests/fixtures/output");
}
