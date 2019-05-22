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
fn test_global() {
    test_in_tmp_dir(|path| {
        let input_dir = "./fixtures/improve-stop-positions/input";
        let model = ntfs::read(input_dir).unwrap();
        let mut collections = model.into_collections();
        tartare_tools::improve_stop_positions::improve_with_pbf(
            Path::new("./fixtures/improve-stop-positions/lemans-nodes.osm.pbf"),
            &mut collections,
            100.00,
        )
        .unwrap();
        let model = Model::new(collections).unwrap();
        ntfs::write(&model, path, get_test_datetime()).unwrap();
        compare_output_dir_with_expected(
            &path,
            Some(vec!["stops.txt"]),
            "./fixtures/improve-stop-positions/output",
        );
    });
}
