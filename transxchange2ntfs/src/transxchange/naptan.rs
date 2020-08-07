//! Module to help parsing and reading NaPTAN files
//! https://en.wikipedia.org/wiki/NaPTAN

use failure::{format_err, ResultExt};
use geo_types::Point;
use log::{info, warn};
use proj::Proj;
use serde::Deserialize;
use std::{collections::HashMap, fs::File, io::Read, path::Path};
use transit_model::{
    model::Collections,
    objects::{Coord, KeysValues, StopArea, StopPoint},
    Result,
};
use typed_index_collection::CollectionWithId;

#[derive(Debug, Deserialize)]
pub struct NaPTANStop {
    #[serde(rename = "ATCOCode")]
    atco_code: String,
    #[serde(rename = "NaptanCode")]
    naptan_code: String,
    #[serde(rename = "CommonName")]
    name: String,
    #[serde(rename = "Longitude")]
    longitude: f64,
    #[serde(rename = "Latitude")]
    latitude: f64,
    #[serde(rename = "Indicator")]
    indicator: String,
}

#[derive(Debug, Deserialize)]
pub struct NaPTANStopInArea {
    #[serde(rename = "AtcoCode")]
    atco_code: String,
    #[serde(rename = "StopAreaCode")]
    stop_area_code: String,
}

#[derive(Debug, Deserialize)]
pub struct NaPTANStopArea {
    #[serde(rename = "StopAreaCode")]
    stop_area_code: String,
    #[serde(rename = "Name")]
    name: String,
    #[serde(rename = "Easting")]
    easting: f64,
    #[serde(rename = "Northing")]
    northing: f64,
}

fn read_stop_areas<R>(reader: R) -> Result<CollectionWithId<StopArea>>
where
    R: Read,
{
    let mut reader = csv::ReaderBuilder::new()
        .delimiter(b',')
        .trim(csv::Trim::All)
        .from_reader(reader);
    let mut stop_areas = CollectionWithId::default();
    let from = "EPSG:27700";
    // FIXME: String 'EPSG:4326' is failing at runtime (string below is equivalent but works)
    let to = "+proj=longlat +datum=WGS84 +no_defs"; // See https://epsg.io/4326
    let converter = Proj::new_known_crs(from, to, None)
        .ok_or_else(|| format_err!("Proj cannot build a converter from '{}' to '{}'", from, to))?;
    for record in reader.deserialize() {
        let stop_area: NaPTANStopArea =
            record.with_context(|_| "Error parsing the CSV record into a StopArea")?;
        let point = Point::new(stop_area.easting, stop_area.northing);
        if let Ok(coord) = converter.convert(point).map(Coord::from) {
            stop_areas.push(StopArea {
                id: stop_area.stop_area_code.clone(),
                name: stop_area.name.clone(),
                visible: true,
                coord,
                ..Default::default()
            })?;
        } else {
            warn!(
                "Failed to convert point ({}, {}) from {} into WGS84",
                point.x(),
                point.y(),
                from,
            );
        }
    }
    Ok(stop_areas)
}

fn read_stops_in_area<R>(
    reader: R,
    stop_areas: &CollectionWithId<StopArea>,
) -> Result<HashMap<String, String>>
where
    R: Read,
{
    fn is_valid_stop_area(
        stop_in_area: &NaPTANStopInArea,
        stop_areas: &CollectionWithId<StopArea>,
    ) -> bool {
        stop_areas
            .get_idx(&stop_in_area.stop_area_code)
            .map(|_| true)
            .unwrap_or_else(|| {
                warn!("Failed to find Stop Area '{}'", stop_in_area.stop_area_code);
                false
            })
    }
    csv::ReaderBuilder::new()
        .delimiter(b',')
        .trim(csv::Trim::All)
        .from_reader(reader)
        .deserialize()
        .map(|record: csv::Result<NaPTANStopInArea>| {
            record.with_context(|_| "Error parsing the CSV record into a StopInArea")
        })
        .filter(|record| {
            match record {
                Ok(stop_in_area) => is_valid_stop_area(stop_in_area, stop_areas),
                // We want to keep record that are Err(_) so the `.collect()` below report errors
                Err(_) => true,
            }
        })
        .map(|record| {
            let stop_in_area = record?;
            Ok((stop_in_area.atco_code.clone(), stop_in_area.stop_area_code))
        })
        .collect()
}

// Create stop points and create missing stop areas for stop points without
// a corresponding stop area in NaPTAN dataset
fn read_stops<R>(
    reader: R,
    stops_in_area: &HashMap<String, String>,
) -> Result<(CollectionWithId<StopPoint>, CollectionWithId<StopArea>)>
where
    R: Read,
{
    let mut reader = csv::ReaderBuilder::new()
        .delimiter(b',')
        .trim(csv::Trim::All)
        .from_reader(reader);
    let mut stop_points = CollectionWithId::default();
    let mut stop_areas = CollectionWithId::default();
    for record in reader.deserialize() {
        let stop: NaPTANStop =
            record.with_context(|_| "Error parsing the CSV record into a Stop")?;
        let coord = Coord {
            lon: stop.longitude,
            lat: stop.latitude,
        };
        let mut codes = KeysValues::new();
        if !stop.naptan_code.is_empty() {
            codes.insert((String::from("NaptanCode"), stop.naptan_code.clone()));
        }
        let mut stop_point = StopPoint {
            id: stop.atco_code.clone(),
            name: stop.name.clone(),
            codes,
            visible: true,
            coord,
            stop_area_id: String::from("default_id"),
            platform_code: Some(stop.indicator.clone()),
            ..Default::default()
        };
        let stop_point = match stops_in_area.get(&stop.atco_code) {
            Some(stop_area_id) => StopPoint {
                stop_area_id: stop_area_id.clone(),
                ..stop_point
            },
            None => {
                let stop_area = StopArea::from(stop_point.clone());
                info!(
                    "Created StopArea {} for corresponding StopPoint {}",
                    stop_area.id, stop.atco_code
                );
                stop_point.stop_area_id = stop_area.id.clone();
                stop_areas.push(stop_area)?;
                stop_point
            }
        };
        stop_points.push(stop_point)?;
    }
    Ok((stop_points, stop_areas))
}

const STOP_AREAS_FILENAME: &str = "StopAreas.csv";
const STOPS_IN_AREA_FILENAME: &str = "StopsInArea.csv";
const STOPS_FILENAME: &str = "Stops.csv";

pub fn read<P: AsRef<Path>>(path: P, collections: &mut Collections) -> Result<()> {
    info!("reading NaPTAN file for {}", STOP_AREAS_FILENAME);
    let stop_areas_filepath = path.as_ref().join(STOP_AREAS_FILENAME);
    let reader = File::open(stop_areas_filepath)?;
    let stop_areas = read_stop_areas(reader)?;

    info!("reading NaPTAN file for {}", STOPS_IN_AREA_FILENAME);
    let stops_in_area_filepath = path.as_ref().join(STOPS_IN_AREA_FILENAME);
    let reader = File::open(stops_in_area_filepath)?;
    let stops_in_area = read_stops_in_area(reader, &stop_areas)?;

    info!("reading NaPTAN file for {}", STOPS_FILENAME);
    let stops_filepath = path.as_ref().join(STOPS_FILENAME);
    let reader = File::open(stops_filepath)?;
    let (stop_points, additional_stop_areas) = read_stops(reader, &stops_in_area)?;

    collections.stop_areas.try_merge(stop_areas)?;
    collections.stop_points.try_merge(stop_points)?;
    collections.stop_areas.try_merge(additional_stop_areas)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    mod read_stop_areas {
        use super::*;
        use pretty_assertions::assert_eq;

        #[test]
        fn parsing_works() {
            let csv_content = r#""StopAreaCode","Name","Easting","Northing"
"010G0001","Bristol Bus Station",358929,173523
"010G0002","Temple Meads",359657,172418"#;
            let stop_areas = read_stop_areas(csv_content.as_bytes()).unwrap();
            assert_eq!(2, stop_areas.len());
            let stop_area = stop_areas.get("010G0001").unwrap();
            assert_eq!("Bristol Bus Station", stop_area.name);
            let stop_area = stop_areas.get("010G0002").unwrap();
            assert_eq!("Temple Meads", stop_area.name);
        }

        #[test]
        #[should_panic]
        fn no_stop_area_code() {
            let csv_content = r#""Name","Easting","Northing"
"Temple Meads",359657,172418"#;
            read_stop_areas(csv_content.as_bytes()).unwrap();
        }

        #[test]
        #[should_panic]
        fn empty_stop_area_code() {
            let csv_content = r#""StopAreaCode","Name","Easting","Northing"
,"Bristol Bus Station",358929,173523
,"Temple Meads",359657,172418"#;
            read_stop_areas(csv_content.as_bytes()).unwrap();
        }

        #[test]
        #[should_panic]
        fn duplicate_id() {
            let csv_content = r#""StopAreaCode","Name","Easting","Northing"
"010G0001","Bristol Bus Station",358929,173523
"010G0001","Bristol Bus Station",358929,173523
"010G0002","Temple Meads",359657,172418"#;
            read_stop_areas(csv_content.as_bytes()).unwrap();
        }
    }

    mod read_stop_in_area {
        use super::*;
        use pretty_assertions::assert_eq;

        #[test]
        fn parsing_works() {
            let csv_content = r#""StopAreaCode","AtcoCode"
"010G0005","01000053220"
"910GBDMNSTR","0100BDMNSTR0""#;
            let mut stop_areas = CollectionWithId::default();
            stop_areas
                .push(StopArea {
                    id: String::from("010G0005"),
                    ..Default::default()
                })
                .unwrap();
            stop_areas
                .push(StopArea {
                    id: String::from("910GBDMNSTR"),
                    ..Default::default()
                })
                .unwrap();
            let stops_in_area = read_stops_in_area(csv_content.as_bytes(), &stop_areas).unwrap();
            assert_eq!(2, stops_in_area.len());
            let stop_area_code = stops_in_area.get("01000053220").unwrap();
            assert_eq!("010G0005", stop_area_code);
            let stop_area_code = stops_in_area.get("0100BDMNSTR0").unwrap();
            assert_eq!("910GBDMNSTR", stop_area_code);
        }

        #[test]
        fn missing_stop_area() {
            let csv_content = r#""StopAreaCode","AtcoCode"
"010G0005","01000053220"
"910GBDMNSTR","0100BDMNSTR0""#;
            let mut stop_areas = CollectionWithId::default();
            stop_areas
                .push(StopArea {
                    id: String::from("010G0005"),
                    ..Default::default()
                })
                .unwrap();
            let stops_in_area = read_stops_in_area(csv_content.as_bytes(), &stop_areas).unwrap();
            assert_eq!(1, stops_in_area.len());
            let stop_area_code = stops_in_area.get("01000053220").unwrap();
            assert_eq!("010G0005", stop_area_code);
        }

        #[test]
        #[should_panic]
        fn no_atco_code() {
            let csv_content = r#""StopAreaCode"
"010G0005"
"910GBDMNSTR""#;
            read_stops_in_area(csv_content.as_bytes(), &CollectionWithId::default()).unwrap();
        }
    }

    mod read_stops {
        use super::*;
        use pretty_assertions::assert_eq;

        #[test]
        fn parsing_works() {
            let csv_content = r#""ATCOCode","NaptanCode","CommonName","Indicator","Longitude","Latitude"
"0100053316","bstjpdm","Broad Walk Shops","Stop B",-2.5876178397,51.4558382170
"0100053264","bstmjdp","Alberton Road","NE-bound",-2.5407019785,51.4889912765"#;
            let mut stop_in_area = HashMap::new();
            stop_in_area.insert(String::from("0100053316"), String::from("stop-area-1"));
            stop_in_area.insert(String::from("0100053308"), String::from("stop-area-3"));
            let (stop_points, stop_areas) =
                read_stops(csv_content.as_bytes(), &stop_in_area).unwrap();

            let stop_point = stop_points.get("0100053316").unwrap();
            assert_eq!("Broad Walk Shops", stop_point.name);
            assert!(stop_point
                .codes
                .contains(&(String::from("NaptanCode"), String::from("bstjpdm"))));
            let stop_point = stop_points.get("0100053264").unwrap();
            assert_eq!("Alberton Road", stop_point.name);
            assert!(stop_point
                .codes
                .contains(&(String::from("NaptanCode"), String::from("bstmjdp"))));

            assert_eq!(1, stop_areas.len());
            let stop_area = stop_areas.get("Navitia:0100053264").unwrap();
            assert_eq!("Alberton Road", stop_area.name);
        }

        #[test]
        #[should_panic]
        fn no_atco_code() {
            let csv_content = r#""NaptanCode","CommonName","Indicator","Longitude","Latitude"
"Broad Walk Shops","bstjpdm","Stop B",-2.5876178397,51.4558382170
"bstmjdp","Alberton Road","NE-bound",-2.5407019785,51.4889912765"#;
            let stop_in_area = HashMap::new();
            read_stops(csv_content.as_bytes(), &stop_in_area).unwrap();
        }

        #[test]
        #[should_panic]
        fn duplicate_id() {
            let csv_content = r#""ATCOCode","NaptanCode","CommonName","Indicator","Longitude","Latitude"
"0100053316","bstjpdm","Broad Walk Shops","Stop B",-2.5876178397,51.4558382170
"0100053316","bstjpdm","Broad Walk Shops","Stop B",-2.5876178397,51.4558382170
"0100053264","bstmjdp","Alberton Road","NE-bound",-2.5407019785,51.4889912765"#;
            let mut stop_in_area = HashMap::new();
            stop_in_area.insert(String::from("0100053316"), String::from("stop-area-1"));
            stop_in_area.insert(String::from("0100053264"), String::from("stop-area-2"));
            read_stops(csv_content.as_bytes(), &stop_in_area).unwrap();
        }
    }
}
