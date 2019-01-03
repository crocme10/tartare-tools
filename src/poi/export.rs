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

use crate::poi::Model;
use crate::Result;
use failure::format_err;
use log::info;
use osm_utils::objects;
use serde::{Serialize, Serializer};
use serde_derive::{Deserialize, Serialize};
use std::io::Write;
use std::path::Path;

pub fn ser_from_bool<S>(v: &bool, serializer: S) -> std::result::Result<S::Ok, S::Error>
where
    S: Serializer,
{
    serializer.serialize_u8(*v as u8)
}

#[derive(Debug, Deserialize, Serialize)]
pub struct ExportPoi {
    #[serde(rename = "poi_id")]
    pub id: String,
    #[serde(rename = "poi_type_id")]
    pub type_id: String,
    #[serde(rename = "poi_name")]
    pub name: String,
    #[serde(rename = "poi_lat")]
    pub lat: f64,
    #[serde(rename = "poi_lon")]
    pub lon: f64,
    #[serde(rename = "poi_weight")]
    pub weight: f64,
    #[serde(rename = "poi_visible", serialize_with = "ser_from_bool")]
    visible: bool,
}

impl From<&objects::Poi> for ExportPoi {
    fn from(poi: &objects::Poi) -> ExportPoi {
        ExportPoi {
            id: poi.id.clone(),
            type_id: poi.poi_type_id.clone(),
            name: poi.name.clone(),
            lat: poi.coord.lat(),
            lon: poi.coord.lon(),
            weight: 0.,
            visible: true,
        }
    }
}

#[derive(Debug, Deserialize, Serialize, Ord, PartialOrd, Eq, PartialEq)]
pub struct ExportPoiType {
    #[serde(rename = "poi_type_id")]
    pub id: String,
    #[serde(rename = "poi_type_name")]
    pub name: String,
}

impl From<&objects::PoiType> for ExportPoiType {
    fn from(poi_type: &objects::PoiType) -> ExportPoiType {
        ExportPoiType {
            id: poi_type.id.clone(),
            name: poi_type.name.clone(),
        }
    }
}

#[derive(Debug, Deserialize, Serialize, Ord, PartialOrd, Eq, PartialEq)]
pub struct ExportPoiProperty {
    pub poi_id: String,
    pub key: String,
    pub value: String,
}

fn get_csv_content<I, T>(items: I) -> Result<Vec<u8>>
where
    I: IntoIterator<Item = T>,
    T: Serialize,
{
    let mut wtr = csv::WriterBuilder::new()
        .delimiter(b';')
        .from_writer(vec![]);
    for i in items.into_iter() {
        wtr.serialize(i)?;
    }
    wtr.into_inner()
        .map_err(|err| format_err!("Error while getting csv data: {}", err))
}

fn write_data_to_zip<W: ::std::io::Write + ::std::io::Seek>(
    zip: &mut zip::ZipWriter<W>,
    filename: &str,
    data: &[u8],
) -> Result<()> {
    zip.start_file(filename, zip::write::FileOptions::default())?;
    zip.write_all(&data)?;

    Ok(())
}

/// Export POIs to a zip file with extension .poi.
///
/// The exported file contains:
/// - poi.txt: a csv file containing the list of this POIs
/// - poi_type.txt: a csv file containing the list of all the POI types, even
/// POI types that do not contain POIs
/// - poi_properties.txt: a csv file containing the list of POI properties
pub fn export<P: AsRef<Path>>(output: P, model: &Model) -> Result<()> {
    info!("Exporting OSM POIs to poi files");
    let output = output.as_ref().with_extension("poi");
    let file = std::fs::File::create(output)?;
    let mut zip = zip::ZipWriter::new(file);

    let mut export_pois: Vec<ExportPoi> = model.pois.iter().map(ExportPoi::from).collect();
    export_pois.sort_unstable_by(|a, b| a.id.cmp(&b.id));
    let data = get_csv_content(export_pois)?;
    write_data_to_zip(&mut zip, "poi.txt", &data)?;

    let mut export_poi_types: Vec<ExportPoiType> =
        model.poi_types.iter().map(ExportPoiType::from).collect();
    export_poi_types.sort_unstable_by(|a, b| a.id.cmp(&b.id));
    let data = get_csv_content(export_poi_types)?;
    write_data_to_zip(&mut zip, "poi_type.txt", &data)?;

    let mut export_poi_properties: Vec<ExportPoiProperty> = model
        .pois
        .iter()
        .flat_map(|p| {
            p.properties.iter().map(move |prop| ExportPoiProperty {
                poi_id: p.id.clone(),
                key: prop.key.clone(),
                value: prop.value.clone(),
            })
        })
        .collect();
    export_poi_properties
        .sort_unstable_by(|a, b| (&a.poi_id, &a.key, &a.value).cmp(&(&b.poi_id, &b.key, &b.value)));
    let data = get_csv_content(export_poi_properties)?;
    write_data_to_zip(&mut zip, "poi_properties.txt", &data)?;

    zip.finish()?;
    Ok(())
}
