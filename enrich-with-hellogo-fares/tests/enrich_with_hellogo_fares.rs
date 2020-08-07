use assert_cmd::prelude::*;
use std::process::Command;
use tempfile::TempDir;
use transit_model::test_utils::*;

#[test]
fn test_read_global_with_prefix() {
    let output_dir = TempDir::new().expect("create temp dir failed");
    Command::cargo_bin("enrich-with-hellogo-fares")
        .expect("Failed to find binary 'enrich-with-hellogo-fares'")
        .arg("--input")
        .arg("tests/fixtures/input/ntfs_with_prefix")
        .arg("--output")
        .arg(output_dir.path().to_str().unwrap())
        .arg("--fares")
        .arg("tests/fixtures/input/hellogo_fares_ok")
        .assert()
        .success();
    compare_output_dir_with_expected(
        &output_dir,
        Some(vec![
            "tickets.txt",
            "ticket_uses.txt",
            "ticket_prices.txt",
            "ticket_use_perimeters.txt",
            "ticket_use_restrictions.txt",
        ]),
        "tests/fixtures/output/ntfs_fares_with_prefix",
    );
}

#[test]
fn test_read_global_without_prefix() {
    let output_dir = TempDir::new().expect("create temp dir failed");
    Command::cargo_bin("enrich-with-hellogo-fares")
        .expect("Failed to find binary 'enrich-with-hellogo-fares'")
        .arg("--input")
        .arg("tests/fixtures/input/ntfs_without_prefix")
        .arg("--output")
        .arg(output_dir.path().to_str().unwrap())
        .arg("--fares")
        .arg("tests/fixtures/input/hellogo_fares_ok")
        .assert()
        .success();
    compare_output_dir_with_expected(
        &output_dir,
        Some(vec![
            "tickets.txt",
            "ticket_uses.txt",
            "ticket_prices.txt",
            "ticket_use_perimeters.txt",
            "ticket_use_restrictions.txt",
        ]),
        "tests/fixtures/output/ntfs_fares_without_prefix",
    );
}

#[test]
#[should_panic(expected = "Failed to find a \'UnitPrice\' fare frame in the Netex file")]
fn test_read_ko_no_unit_price() {
    let output_dir = TempDir::new().expect("create temp dir failed");
    Command::cargo_bin("enrich-with-hellogo-fares")
        .expect("Failed to find binary 'enrich-with-hellogo-fares'")
        .arg("--input")
        .arg("tests/fixtures/input/ntfs_with_prefix")
        .arg("--output")
        .arg(output_dir.path().to_str().unwrap())
        .arg("--fares")
        .arg("tests/fixtures/input/hellogo_fares_ko_no_unit")
        .assert()
        .success();
}

#[test]
#[should_panic(expected = "Failed to find a unique \'UnitPrice\' fare frame in the Netex file")]
fn test_read_ko_several_unit_prices() {
    let output_dir = TempDir::new().expect("create temp dir failed");
    Command::cargo_bin("enrich-with-hellogo-fares")
        .expect("Failed to find binary 'enrich-with-hellogo-fares'")
        .arg("--input")
        .arg("tests/fixtures/input/ntfs_with_prefix")
        .arg("--output")
        .arg(output_dir.path().to_str().unwrap())
        .arg("--fares")
        .arg("tests/fixtures/input/hellogo_fares_ko_several_unit")
        .assert()
        .success();
}
