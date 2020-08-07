use std::collections::HashMap;
use std::fs::File;
use std::io;
use std::path::Path;
use tartare_tools::poi::sytral::extract_pois;
use transit_model::test_utils::*;

fn cover_all_fixtures() -> HashMap<&'static str, &'static str> {
    let mut map = HashMap::new();
    map.insert("main", tartare_tools::poi::sytral::MAIN_FILE);
    map.insert("pv", tartare_tools::poi::sytral::PV_FILE);
    map.insert("pr", tartare_tools::poi::sytral::PR_FILE);
    map
}

#[test]
fn test_export_sytral_pois_ok() {
    test_in_tmp_dir(|path| {
        let input_path = "./tests/fixtures/sytral2navitia-pois/input/OK";
        let poi_model = extract_pois(input_path).unwrap();
        let output_file = path.join("output.poi");
        poi_model.save_to_path(output_file.clone()).unwrap();

        // file extension should be .poi
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
            "./tests/fixtures/sytral2navitia-pois/output",
        );
    });
}

#[test]
fn test_export_sytral_pois_ko_csv_manquant() {
    let input_path_prefix =
        Path::new("./tests/fixtures/sytral2navitia-pois/input/sytral_poi_echec1_csv_manquant");
    for (suffix, file_name) in cover_all_fixtures() {
        let input_path = input_path_prefix.join(suffix);
        let poi_model = extract_pois(input_path);
        match poi_model {
            Ok(_) => panic!(),
            Err(e) => assert_eq!(format!("{}", e), format!("missing file {}", file_name)),
        };
    }
}

#[test]
fn test_export_sytral_pois_ko_poi_type_id_manquant() {
    let input_path =
        "./tests/fixtures/sytral2navitia-pois/input/sytral_poi_echec2_poi_type_id_manquant";
    let poi_model = extract_pois(input_path);
    match poi_model {
        Ok(_) => panic!(),
        Err(e) => assert_eq!(e.iter_chain().map(|err| format!("{}", err)).collect::<Vec<String>>(),
                             vec!["Error reading \"./tests/fixtures/sytral2navitia-pois/input/sytral_poi_echec2_poi_type_id_manquant/POI_TCL.csv\"",
                                  "CSV deserialize error: record 1 (line: 2, byte: 92): empty string not allowed in deserialization"]),
    };
}

#[test]
fn test_export_sytral_pois_ko_poi_id_manquant() {
    let input_path_prefix =
        Path::new("./tests/fixtures/sytral2navitia-pois/input/sytral_poi_echec3_poi_id_manquant");
    for (suffix, file_name) in cover_all_fixtures() {
        let input_path = input_path_prefix.join(suffix);
        let poi_model = extract_pois(input_path);
        assert!(poi_model.is_err());
        match poi_model {
            Ok(_) => panic!(),
            Err(e) => assert_eq!(
                format!("{}", e),
                format!("poi with undefined id found in file {}", file_name)
            ),
        };
    }
}

#[test]
fn test_export_sytral_pois_ko_poi_x_manquant() {
    let input_path = "./tests/fixtures/sytral2navitia-pois/input/sytral_poi_echec3_poi_x_manquant";
    let poi_model = extract_pois(input_path);
    assert!(poi_model.is_err());
    match poi_model {
        Ok(_) => panic!(),
        Err(e) => assert_eq!(e.iter_chain().map(|err| format!("{}", err)).collect::<Vec<String>>(),
                             vec!["Error reading \"./tests/fixtures/sytral2navitia-pois/input/sytral_poi_echec3_poi_x_manquant/POI_TCL.csv\"",
                                  "CSV deserialize error: record 1 (line: 2, byte: 92): cannot parse float from empty string"]),
    };
}

#[test]
fn test_export_sytral_pois_ko_poi_y_manquant() {
    let input_path = "./tests/fixtures/sytral2navitia-pois/input/sytral_poi_echec3_poi_y_manquant";
    let poi_model = extract_pois(input_path);
    assert!(poi_model.is_err());
    match poi_model {
        Ok(_) => panic!(),
        Err(e) => assert_eq!(e.iter_chain().map(|err| format!("{}", err)).collect::<Vec<String>>(),
                             vec!["Error reading \"./tests/fixtures/sytral2navitia-pois/input/sytral_poi_echec3_poi_y_manquant/parcs_velos.csv\"",
                                  "CSV deserialize error: record 2 (line: 3, byte: 160): cannot parse float from empty string"]),
    };
}

#[test]
fn test_export_sytral_poi_id_double() {
    let input_path_prefix =
        Path::new("./tests/fixtures/sytral2navitia-pois/input/sytral_poi_echec4_poi_id_double");
    for (suffix, file_name) in cover_all_fixtures() {
        let input_path = input_path_prefix.join(suffix);
        let poi_model = extract_pois(input_path);
        match poi_model {
            Ok(_) => panic!(),
            Err(e) => assert_eq!(
                format!("{}", e),
                format!(
                    "poi with id \"duplicated:id\" and type \"duplicated:type\" found at least twice in file \"{}\"",
                    file_name
                )
            ),
        };
    }
}
