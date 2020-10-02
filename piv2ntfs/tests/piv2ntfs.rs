use assert_cmd::prelude::*;
use std::process::Command;
use tempfile::TempDir;
use transit_model::test_utils::*;

#[test]
fn test_piv2ntfs() {
    let output_dir = TempDir::new().expect("create temp dir failed");
    Command::cargo_bin("piv2ntfs")
        .expect("Failed to find binary 'piv2ntfs'")
        .arg("--input")
        .arg("tests/fixtures/input_scrap_piv")
        .arg("--output")
        .arg(output_dir.path().to_str().unwrap())
        .arg("--config")
        .arg("tests/fixtures/config.json")
        .arg("--prefix")
        .arg("PFX")
        .arg("--current-datetime")
        .arg("2020-09-20T06:00:00+00:00")
        .assert()
        .success();
    compare_output_dir_with_expected(&output_dir, None, "tests/fixtures/output_ntfs");
}
