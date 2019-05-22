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

use std::path::Path;
use transit_model::ntfs;
use transit_model::test_utils::*;
use transit_model::Model;

#[test]
fn test_read_shapes_from_osm() {
    test_in_tmp_dir(|path| {
        let input_dir = "./fixtures/read-shapes-from-osm/input/ok";
        let model = ntfs::read(input_dir).unwrap();
        let mut collections = model.into_collections();
        tartare_tools::read_shapes::from_osm(
            Path::new("./fixtures/read-shapes-from-osm/sample-lite.osm.pbf"),
            &mut collections,
        )
        .unwrap();
        let model = Model::new(collections).unwrap();
        ntfs::write(&model, path, get_test_datetime()).unwrap();
        compare_output_dir_with_expected(
            &path,
            Some(vec!["lines.txt", "routes.txt", "geometries.txt"]),
            "./fixtures/read-shapes-from-osm/output",
        );
    });
}

#[test]
#[should_panic(expected = "relation relation:unknown not found in osm")]
fn test_read_shapes_relation_not_found() {
    let input_dir = "./fixtures/read-shapes-from-osm/input/unknown_relation_osm";
    let model = ntfs::read(input_dir).unwrap();
    let mut collections = model.into_collections();
    tartare_tools::read_shapes::from_osm(
        Path::new("./fixtures/read-shapes-from-osm/sample-lite.osm.pbf"),
        &mut collections,
    )
    .unwrap();
}
