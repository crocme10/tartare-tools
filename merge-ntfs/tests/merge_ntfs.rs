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

use approx::assert_relative_eq;
use assert_cmd::prelude::*;
use pretty_assertions::assert_eq;
use std::{collections::HashMap, process::Command};
use tempfile::TempDir;
use transit_model::{
    objects::{CommentLinks, StopPoint, VehicleJourney},
    test_utils::*,
};
use typed_index_collection::{CollectionWithId, Idx};

#[test]
fn merge_collections_with_collisions() {
    let output_dir = TempDir::new().expect("create temp dir failed");
    Command::cargo_bin("merge-ntfs")
        .expect("Failed to find binary 'merge-ntfs'")
        .arg("--output")
        .arg(output_dir.path().to_str().unwrap())
        // Input folders
        .arg("tests/fixtures/ntfs")
        .arg("tests/fixtures/ntfs")
        .assert()
        .failure()
        .stderr(predicates::str::contains(
            "identifier RERACOM1 already exists",
        ));
}

#[test]
#[allow(clippy::cognitive_complexity)]
fn merge_collections_ok() {
    let output_dir = TempDir::new().expect("create temp dir failed");
    Command::cargo_bin("merge-ntfs")
        .expect("Failed to find binary 'merge-ntfs'")
        .arg("--output")
        .arg(output_dir.path().to_str().unwrap())
        // Putting 0 to avoid creation of transfers
        .arg("--max-distance")
        .arg("0.0")
        .arg("--inter-contributors-transfers-only")
        // Input folders
        .arg("tests/fixtures/ntfs")
        .arg("tests/fixtures/merge-ntfs/input")
        .assert()
        .success();
    let collections = transit_model::ntfs::read(output_dir).unwrap();
    assert_eq!(2, collections.contributors.len());
    assert_eq!(2, collections.datasets.len());
    assert_eq!(3, collections.networks.len());
    // check that commercial mode Bus appears once.
    let count_bus = collections
        .commercial_modes
        .values()
        .filter(|cm| cm.id == "Bus" && cm.name == "Bus")
        .count();
    assert_eq!(1, count_bus);
    // Check that the merge of CO2 emission keeps only the biggest value
    let bus_mode = collections.physical_modes.get("Bus").unwrap();
    assert_relative_eq!(132f32, bus_mode.co2_emission.unwrap());

    assert_eq!(5, collections.commercial_modes.len());
    // 4 + 3 automatically inserted 'Bike', 'BikeSharingService', and 'Car'
    assert_eq!(7, collections.physical_modes.len());
    assert_eq!(5, collections.lines.len());
    assert_eq!(8, collections.routes.len());
    assert_eq!(10, collections.vehicle_journeys.len());
    assert_eq!(2, collections.frequencies.len());
    assert_eq!(1, collections.stop_time_headsigns.len());
    assert_eq!(8, collections.stop_time_ids.len());
    assert_eq!(4, collections.levels.len());
    assert_eq!(3, collections.pathways.len());
    assert_eq!(1, collections.grid_calendars.len());
    assert_eq!(1, collections.grid_exception_dates.len());
    assert_eq!(1, collections.grid_periods.len());
    assert_eq!(1, collections.grid_rel_calendar_line.len());

    let mut headsigns = HashMap::<(String, u32), String>::new();
    headsigns.insert(("OIF:77100911-1_1420-1".into(), 3), "somewhere".into());
    headsigns.insert(("OIF:77100911-1_1420-1".into(), 3), "somewhere".into());
    assert_eq!(headsigns, collections.stop_time_headsigns);

    let mut stop_times_ids = HashMap::<(String, u32), String>::new();
    stop_times_ids.insert(
        ("OIF:77100911-1_1420-1".into(), 3),
        "StopTime:OIF:77100911-1_1420-1:1".into(),
    );
    stop_times_ids.insert(
        ("OIF:77100911-1_1420-1".into(), 0),
        "StopTime:OIF:77100911-1_1420-1:0".into(),
    );
    stop_times_ids.insert(
        ("OIF:77100911-1_1420-1".into(), 4),
        "StopTime:OIF:77100911-1_1420-1:2".into(),
    );
    stop_times_ids.insert(
        ("OIF:77100915-1_1424-1".into(), 0),
        "StopTime:OIF:77100915-1_1424-1:0".into(),
    );
    stop_times_ids.insert(
        ("OIF:77100921-1_1420-1".into(), 0),
        "StopTime:OIF:77100921-1_1420-1:0".into(),
    );
    stop_times_ids.insert(
        ("OIF:77100925-1_1424-1".into(), 0),
        "StopTime:OIF:77100925-1_1424-1:0".into(),
    );
    stop_times_ids.insert(("RERAB1".into(), 5), "StopTime:RERAB1-5:1".into());
    stop_times_ids.insert(("RERAB1".into(), 8), "StopTime:RERAB1-8:0".into());

    assert_eq!(stop_times_ids, collections.stop_time_ids);

    let mut stop_time_comments = HashMap::<(String, u32), String>::new();
    stop_time_comments.insert(("RERAB1".into(), 5), "RERACOM1".into());
    stop_time_comments.insert(("OIF:77100911-1_1420-1".into(), 4), "OIFCOM1".into());
    assert_eq!(stop_time_comments, collections.stop_time_comments);

    fn get_stop_point_idxs(
        col: &CollectionWithId<VehicleJourney>,
        id: &str,
    ) -> Vec<Idx<StopPoint>> {
        col.get(id)
            .unwrap()
            .stop_times
            .iter()
            .map(|st| st.stop_point_idx)
            .collect()
    }

    assert_eq!(
        vec![
            collections.stop_points.get_idx("DEFR").unwrap(),
            collections.stop_points.get_idx("CDGR").unwrap(),
            collections.stop_points.get_idx("GDLR").unwrap(),
            collections.stop_points.get_idx("NATR").unwrap(),
        ],
        get_stop_point_idxs(&collections.vehicle_journeys, "RERAB1")
    );
    assert_eq!(
        vec![
            collections.stop_points.get_idx("OIF:SP:10:10").unwrap(),
            collections.stop_points.get_idx("OIF:SP:10:100").unwrap(),
            collections.stop_points.get_idx("OIF:SP:10:200").unwrap(),
        ],
        get_stop_point_idxs(&collections.vehicle_journeys, "OIF:77100911-1_1420-1")
    );
    assert_eq!(7, collections.stop_areas.len());
    assert_eq!(14, collections.stop_points.len());
    assert_eq!(6, collections.feed_infos.len());
    assert_eq!(
        261,
        collections.calendars.values().next().unwrap().dates.len()
    );
    assert_eq!(
        6,
        collections.calendars.values().nth(1).unwrap().dates.len()
    );
    assert_eq!(3, collections.companies.len());
    assert_eq!(7, collections.comments.len());
    assert_eq!(0, collections.equipments.len());
    assert_eq!(0, collections.transfers.len());
    assert_eq!(0, collections.trip_properties.len());
    assert_eq!(0, collections.geometries.len());
    assert_eq!(0, collections.admin_stations.len());

    fn assert_comment_ids<T: CommentLinks>(
        collection: &CollectionWithId<T>,
        obj_id: &str,
        comment_id: &str,
    ) {
        assert_eq!(
            comment_id,
            collection
                .get(obj_id)
                .unwrap()
                .comment_links()
                .iter()
                .next()
                .unwrap()
        );
    }

    assert_comment_ids(&collections.stop_points, "OIF:SP:10:10", "OIFCOM2");
    assert_comment_ids(&collections.stop_areas, "OIF:SA:10:1002", "OIFCOM3");
}

#[test]
fn merge_collections_with_transfers_ok() {
    let output_dir = TempDir::new().expect("create temp dir failed");
    let report_path = output_dir.path().join("report.json");
    Command::cargo_bin("merge-ntfs")
        .expect("Failed to find binary 'merge-ntfs'")
        .arg("--output")
        .arg(output_dir.path().to_str().unwrap())
        .arg("--config")
        .arg("tests/fixtures/merge-ntfs/transfer_rules.csv")
        .arg("--max-distance")
        .arg("100.0")
        .arg("--walking-speed")
        .arg("0.785")
        .arg("--waiting-time")
        .arg("60")
        .arg("--inter-contributors-transfers-only")
        .arg("--report")
        .arg(report_path.to_str().unwrap())
        // Input folders
        .arg("tests/fixtures/minimal_ntfs")
        .arg("tests/fixtures/merge-ntfs/input")
        .assert()
        .success();
    compare_output_dir_with_expected(
        &output_dir,
        Some(vec!["transfers.txt", "report.json"]),
        "./tests/fixtures/merge-ntfs/output",
    );
}

#[test]
fn merge_collections_with_feed_infos() {
    let output_dir = TempDir::new().expect("create temp dir failed");
    Command::cargo_bin("merge-ntfs")
        .expect("Failed to find binary 'merge-ntfs'")
        .arg("--output")
        .arg(output_dir.path().to_str().unwrap())
        .arg("--feed-infos")
        .arg("tests/fixtures/merge-ntfs/feed_infos.json")
        .arg("--current-datetime")
        .arg("2019-04-03T17:19:00+00:00")
        // Input folders
        .arg("tests/fixtures/minimal_ntfs")
        .arg("tests/fixtures/merge-ntfs/input")
        .assert()
        .success();
    compare_output_dir_with_expected(
        &output_dir,
        Some(vec!["feed_infos.txt"]),
        "./tests/fixtures/merge-ntfs/output_feedinfos",
    );
}

#[test]
fn merge_collections_fares_v2() {
    let output_dir = TempDir::new().expect("create temp dir failed");
    Command::cargo_bin("merge-ntfs")
        .expect("Failed to find binary 'merge-ntfs'")
        .arg("--output")
        .arg(output_dir.path().to_str().unwrap())
        // Input folders
        .arg("tests/fixtures/ntfs")
        .arg("tests/fixtures/merge-ntfs/input")
        .assert()
        .success();
    compare_output_dir_with_expected(
        &output_dir,
        Some(vec![
            "tickets.txt",
            "ticket_prices.txt",
            "ticket_uses.txt",
            "ticket_use_perimeters.txt",
            "ticket_use_restrictions.txt",
            "prices.csv",
            "fares.csv",
            "od_fares.csv",
        ]),
        "./tests/fixtures/merge-ntfs/output_merge_fares",
    );
}

#[test]
fn merge_collections_fares_v2_with_collisions() {
    let output_dir = TempDir::new().expect("create temp dir failed");
    Command::cargo_bin("merge-ntfs")
        .expect("Failed to find binary 'merge-ntfs'")
        .arg("--output")
        .arg(output_dir.path().to_str().unwrap())
        // Input folders
        .arg("tests/fixtures/ntfs")
        .arg("tests/fixtures/merge-ntfs/input_farev2_conflicts")
        .assert()
        .success();
    let output_model = transit_model::ntfs::read(output_dir).unwrap();
    assert_eq!(4, output_model.tickets.len());
    assert_eq!(5, output_model.ticket_prices.len());
    assert_eq!(4, output_model.ticket_uses.len());
    assert_eq!(8, output_model.ticket_use_perimeters.len());
    assert_eq!(4, output_model.ticket_use_restrictions.len());
}

#[test]
fn merge_collections_fares_v2_not_convertible_in_v1() {
    let output_dir = TempDir::new().expect("create temp dir failed");
    Command::cargo_bin("merge-ntfs")
        .expect("Failed to find binary 'merge-ntfs'")
        .arg("--output")
        .arg(output_dir.path().to_str().unwrap())
        // Input folders
        .arg("tests/fixtures/ntfs")
        .arg("tests/fixtures/merge-ntfs/input_faresv2_without_euro_currency")
        .assert()
        .success();
    let output_model = transit_model::ntfs::read(output_dir).unwrap();
    assert!(output_model.prices_v1.is_empty());
    assert!(output_model.od_fares_v1.is_empty());
    assert!(output_model.fares_v1.is_empty());
}

#[test]
fn merge_collections_fares_v2_with_ntfs_only_farev1() {
    let output_dir = TempDir::new().expect("create temp dir failed");
    Command::cargo_bin("merge-ntfs")
        .expect("Failed to find binary 'merge-ntfs'")
        .arg("--output")
        .arg(output_dir.path().to_str().unwrap())
        // Input folders
        .arg("tests/fixtures/ntfs")
        .arg("tests/fixtures/merge-ntfs/input_only_farev1")
        .assert()
        .success();
    compare_output_dir_with_expected(
        &output_dir,
        Some(vec![
            "tickets.txt",
            "ticket_prices.txt",
            "ticket_uses.txt",
            "ticket_use_perimeters.txt",
            "ticket_use_restrictions.txt",
            "prices.csv",
            "fares.csv",
            "od_fares.csv",
        ]),
        "./tests/fixtures/merge-ntfs/output_merge_fares_only_one_farev2",
    );
}

#[test]
fn merge_intra_contributor() {
    let output_dir = TempDir::new().expect("create temp dir failed");
    Command::cargo_bin("merge-ntfs")
        .expect("Failed to find binary 'merge-ntfs'")
        .arg("--output")
        .arg(output_dir.path().to_str().unwrap())
        // Input folders
        .arg("tests/fixtures/complete_subprefixed/input_ntfs1")
        .arg("tests/fixtures/complete_subprefixed/input_ntfs2")
        .assert()
        .success();
    compare_output_dir_with_expected(
        &output_dir,
        Some(vec![
            "calendar.txt",
            "comment_links.txt",
            "comments.txt",
            "commercial_modes.txt",
            "companies.txt",
            "contributors.txt",
            "datasets.txt",
            "equipments.txt",
            "frequencies.txt",
            "geometries.txt",
            "levels.txt",
            "lines.txt",
            "networks.txt",
            "object_codes.txt",
            "object_properties.txt",
            "pathways.txt",
            "physical_modes.txt",
            "routes.txt",
            "stops.txt",
            "stop_times.txt",
            "transfers.txt",
            "trip_properties.txt",
            "trips.txt",
        ]),
        "./tests/fixtures/complete_subprefixed/output_ntfs",
    );
}
