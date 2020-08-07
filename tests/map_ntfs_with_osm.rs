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
