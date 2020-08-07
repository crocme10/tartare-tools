use std::fs::File;
use std::io;
use tartare_tools::poi::merge::merge;
use transit_model::test_utils::*;

#[test]
fn test_merge_pois() {
    test_in_tmp_dir(|path| {
        let poi1 = "./tests/fixtures/merge_pois/input/poi1.poi";
        let poi2 = "./tests/fixtures/merge_pois/input/poi2.poi";

        let model = merge(&mut [poi1, poi2].iter()).unwrap();
        model.save_to_path(path.join("pois.zip")).unwrap();

        // file extension should be .poi
        let output_file = path.join("pois.poi");
        assert!(output_file.is_file());

        let file = File::open(&output_file).unwrap();
        let mut archive = zip::ZipArchive::new(file).unwrap();

        for i in 0..archive.len() {
            let mut file = archive.by_index(i).unwrap();
            let outpath = file.sanitized_name();
            let mut outfile = File::create(&path.join(outpath)).unwrap();
            io::copy(&mut file, &mut outfile).unwrap();
        }

        compare_output_dir_with_expected(
            &path,
            Some(vec!["poi.txt", "poi_properties.txt", "poi_type.txt"]),
            "./tests/fixtures/merge_pois/output",
        );
    });
}
