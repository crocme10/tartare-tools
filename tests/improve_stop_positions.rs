use std::path::Path;
use transit_model::ntfs;
use transit_model::test_utils::*;
use transit_model::Model;

#[test]
fn test_global() {
    test_in_tmp_dir(|path| {
        let input_dir = "./tests/fixtures/improve-stop-positions/input";
        let model = ntfs::read(input_dir).unwrap();
        let mut collections = model.into_collections();
        tartare_tools::improve_stop_positions::improve_with_pbf(
            Path::new("./tests/fixtures/improve-stop-positions/lemans-nodes.osm.pbf"),
            &mut collections,
            100.00,
        )
        .unwrap();
        let model = Model::new(collections).unwrap();
        ntfs::write(&model, path, get_test_datetime()).unwrap();
        compare_output_dir_with_expected(
            &path,
            Some(vec!["stops.txt"]),
            "./tests/fixtures/improve-stop-positions/output",
        );
    });
}
