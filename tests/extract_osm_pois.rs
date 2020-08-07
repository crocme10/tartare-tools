use osm_utils::poi::PoiConfig;
use std::fs::File;
use std::io;
use tartare_tools::poi::osm::extract_pois;
use transit_model::test_utils::*;

#[test]
fn test_export_pois() {
    test_in_tmp_dir(|path| {
        let osm_pbf = "./tests/fixtures/extract_osm_pois/input/osm_fixture.osm.pbf";
        let pois_config = "./tests/fixtures/extract_osm_pois/input/pois_config.json";

        let r = File::open(pois_config).unwrap();
        let matcher = PoiConfig::from_reader(r).unwrap();

        let model = extract_pois(osm_pbf, matcher).unwrap();
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
            "./tests/fixtures/extract_osm_pois/output",
        );
    });
}
