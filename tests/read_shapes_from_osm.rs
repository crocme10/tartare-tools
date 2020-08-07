use std::path::Path;
use transit_model::ntfs;
use transit_model::test_utils::*;
use transit_model::Model;

#[test]
fn test_read_shapes_from_osm() {
    test_in_tmp_dir(|path| {
        let input_dir = "./tests/fixtures/read-shapes-from-osm/input/ok";
        let model = ntfs::read(input_dir).unwrap();
        let mut collections = model.into_collections();
        tartare_tools::read_shapes::from_osm(
            Path::new("./tests/fixtures/read-shapes-from-osm/sample-lite.osm.pbf"),
            &mut collections,
        )
        .unwrap();
        let model = Model::new(collections).unwrap();
        ntfs::write(&model, path, get_test_datetime()).unwrap();
        compare_output_dir_with_expected(
            &path,
            Some(vec!["lines.txt", "routes.txt", "geometries.txt"]),
            "./tests/fixtures/read-shapes-from-osm/output",
        );
    });
}

#[test]
fn test_read_shapes_relation_not_found() {
    let input_dir = "./tests/fixtures/read-shapes-from-osm/input/unknown_relation_osm";
    let model = ntfs::read(input_dir).unwrap();
    let mut collections = model.into_collections();
    tartare_tools::read_shapes::from_osm(
        Path::new("./tests/fixtures/read-shapes-from-osm/sample-lite.osm.pbf"),
        &mut collections,
    )
    .unwrap();
    // No new geometry created since the relation is incorrect
    assert_eq!(0, collections.geometries.len());
}
