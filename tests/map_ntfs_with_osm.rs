use log::Level as LogLevel;
use std::collections::HashMap;
use std::path::Path;
use tartare_tools::improve_stop_positions;
use transit_model::ntfs;
use transit_model::test_utils::*;

#[test]
fn test_map_no_force() {
    test_in_tmp_dir(|path| {
        let mut ntfs_network_to_osm = HashMap::new();
        ntfs_network_to_osm.insert("RTM", "RTM");
        let input_dir = "./tests/fixtures/map-ntfs-with-osm/input";
        let model = ntfs::read(input_dir).unwrap();
        let enriched_model = improve_stop_positions::enrich_object_codes(
            Path::new("./tests/fixtures/map-ntfs-with-osm/marseille-lite.osm.pbf"),
            model,
            ntfs_network_to_osm,
            false,
        )
        .unwrap();
        transit_model::ntfs::write(&enriched_model, path, get_test_datetime()).unwrap();
        compare_output_dir_with_expected(
            &path,
            Some(vec!["object_codes.txt"]),
            "./tests/fixtures/map-ntfs-with-osm/output/no_force",
        );
    });
}

#[test]
fn test_map_force() {
    test_in_tmp_dir(|path| {
        let mut ntfs_network_to_osm = HashMap::new();
        ntfs_network_to_osm.insert("RTM", "RTM");
        let input_dir = "./tests/fixtures/map-ntfs-with-osm/input";
        let model = ntfs::read(input_dir).unwrap();
        let enriched_model = improve_stop_positions::enrich_object_codes(
            Path::new("./tests/fixtures/map-ntfs-with-osm/marseille-lite.osm.pbf"),
            model,
            ntfs_network_to_osm,
            true,
        )
        .unwrap();
        transit_model::ntfs::write(&enriched_model, path, get_test_datetime()).unwrap();
        compare_output_dir_with_expected(
            &path,
            Some(vec!["object_codes.txt"]),
            "./tests/fixtures/map-ntfs-with-osm/output/force",
        );
    });
}

#[test]
fn test_unknown_network() {
    test_in_tmp_dir(|path| {
        let mut ntfs_network_to_osm = HashMap::new();
        ntfs_network_to_osm.insert("UNKNOWN", "UNKNOWN");
        ntfs_network_to_osm.insert("RTM", "RTM");
        let input_dir = "./tests/fixtures/map-ntfs-with-osm/input";
        let model = ntfs::read(input_dir).unwrap();
        testing_logger::setup();
        let enriched_model = improve_stop_positions::enrich_object_codes(
            Path::new("./tests/fixtures/map-ntfs-with-osm/marseille-lite.osm.pbf"),
            model,
            ntfs_network_to_osm,
            false,
        )
        .unwrap();
        transit_model::ntfs::write(&enriched_model, path, get_test_datetime()).unwrap();
        compare_output_dir_with_expected(
            &path,
            Some(vec!["object_codes.txt"]),
            "./tests/fixtures/map-ntfs-with-osm/output/no_force",
        );
        testing_logger::validate(|captured_logs| {
            assert!(captured_logs
                .iter()
                .filter(|log| log.level == LogLevel::Warn)
                .any(|log| log.body
                    == "The network (id=\"UNKNOWN\") doesn\'t exist in the provided NTFS"));
        });
    });
}

#[test]
fn test_osm_without_route() {
    test_in_tmp_dir(|path| {
        let mut ntfs_network_to_osm = HashMap::new();
        ntfs_network_to_osm.insert("RTM", "RTM");
        let input_dir = "./tests/fixtures/map-ntfs-with-osm/input";
        let model = ntfs::read(input_dir).unwrap();
        testing_logger::setup();
        let osm_pbf_path =
            Path::new("./tests/fixtures/map-ntfs-with-osm/lemans_no-PT-route.osm.pbf");
        let enriched_model = improve_stop_positions::enrich_object_codes(
            osm_pbf_path,
            model,
            ntfs_network_to_osm,
            false,
        )
        .unwrap();
        transit_model::ntfs::write(&enriched_model, path, get_test_datetime()).unwrap();
        let expected_log = format!(
            "no lines found in osm for file {}",
            osm_pbf_path.to_str().unwrap()
        );
        testing_logger::validate(|captured_logs| {
            assert!(captured_logs
                .iter()
                .filter(|log| log.level == LogLevel::Warn)
                .any(|log| log.body == expected_log));
        });
    });
}
