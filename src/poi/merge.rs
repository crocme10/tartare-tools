// Copyright 2017-2018 Kisio Digital and/or its affiliates.
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
use std::{
    collections::{hash_map::Entry::*, HashMap},
    fs::File,
};

use csv;
use failure::bail;
use osm_utils::objects::{Coord, Poi, PoiType, Property};
use zip;

use crate::{
    poi::{
        export::{ExportPoi, ExportPoiProperty, ExportPoiType},
        Model,
    },
    Result,
};

fn make_csv_reader_from_zip<'a>(
    zip: &'a mut zip::ZipArchive<File>,
    file: &str,
) -> Result<csv::Reader<zip::read::ZipFile<'a>>> {
    let file = zip.by_name(file)?;
    Ok(csv::ReaderBuilder::new().delimiter(b';').from_reader(file))
}

fn merge_poi_types(
    zip: &mut zip::ZipArchive<File>,
    poi_types: &mut HashMap<String, PoiType>,
) -> Result<()> {
    let mut rdr = make_csv_reader_from_zip(zip, "poi_type.txt")?;

    for export_poi_type in rdr.deserialize() {
        let export_poi_type: ExportPoiType = export_poi_type?;
        let poi_type = PoiType {
            id: export_poi_type.id.to_string(),
            name: export_poi_type.name.to_string(),
        };
        match poi_types.entry(poi_type.id.to_string()) {
            Occupied(v) => {
                if v.get().name != poi_type.name {
                    bail!("POI type \"{}\" already found but with 2 different labels \"{}\" and \"{}\"", v.get().id, v.get().name, poi_type.name)
                }
            }
            Vacant(v) => {
                v.insert(poi_type);
            }
        }
    }

    Ok(())
}

fn add_props(zip: &mut zip::ZipArchive<File>, pois: &mut HashMap<String, Poi>) -> Result<()> {
    let pois_file = zip.by_name("poi_properties.txt")?;
    let mut rdr = csv::ReaderBuilder::new()
        .delimiter(b';')
        .from_reader(pois_file);

    for export_poi_prop in rdr.deserialize() {
        let export_poi_prop: ExportPoiProperty = export_poi_prop?;

        let prop = Property {
            key: export_poi_prop.key.to_string(),
            value: export_poi_prop.value.to_string(),
        };

        pois.entry(export_poi_prop.poi_id).and_modify(|p| {
            if !p.properties.contains(&prop) {
                p.properties.push(prop);
            }
        });
    }
    Ok(())
}

fn merge_pois(zip: &mut zip::ZipArchive<File>, pois: &mut HashMap<String, Poi>) -> Result<()> {
    let pois_file = zip.by_name("poi.txt")?;
    let mut rdr = csv::ReaderBuilder::new()
        .delimiter(b';')
        .from_reader(pois_file);

    for export_poi in rdr.deserialize() {
        let export_poi: ExportPoi = export_poi?;
        let p = Poi {
            id: export_poi.id.to_string(),
            name: export_poi.name.to_string(),
            coord: Coord::new(export_poi.lon, export_poi.lat),
            poi_type_id: export_poi.type_id.to_string(),
            properties: vec![],
        };
        match pois.entry(export_poi.id.to_string()) {
            Occupied(_) => bail!("POI {} already found", p.id),
            Vacant(v) => v.insert(p),
        };
    }

    Ok(())
}

pub fn merge<P: AsRef<Path>>(paths: &[P]) -> Result<Model> {
    let mut pois = HashMap::<String, Poi>::new();
    let mut poi_types = HashMap::<String, PoiType>::new();

    for poi_file in paths {
        let file = File::open(poi_file.as_ref())?;
        let mut zip = zip::ZipArchive::new(file)?;
        merge_poi_types(&mut zip, &mut poi_types)?;
        merge_pois(&mut zip, &mut pois)?;
        add_props(&mut zip, &mut pois)?;
    }
    Ok(Model {
        pois: pois.into_iter().map(|(_, p)| p).collect(),
        poi_types: poi_types.into_iter().map(|(_, p)| p).collect(),
    })
}
