// Copyright 2017 Kisio Digital and/or its affiliates.
//
// This program is free software: you can redistribute it and/or
// modify it under the terms of the GNU General Public License as
// published by the Free Software Foundation, either version 3 of the
// License, or (at your option) any later version.
//
// This program is distributed in the hope that it will be useful, but
// WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the GNU
// General Public License for more details.
//
// You should have received a copy of the GNU General Public License
// along with this program.  If not, see
// <http://www.gnu.org/licenses/>.

use osm_utils::poi::PoiConfig;
use std::fs::File;
use std::io;
use tartare_tools::poi::osm::extract_pois;
use transit_model::test_utils::*;

#[test]
fn test_export_pois() {
    test_in_tmp_dir(|path| {
        let osm_pbf = "./fixtures/extract_osm_pois/input/osm_fixture.osm.pbf";
        let pois_config = "./fixtures/extract_osm_pois/input/pois_config.json";

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
            "./fixtures/extract_osm_pois/output",
        );
    });
}
