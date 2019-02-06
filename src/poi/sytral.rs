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

use crate::{poi::Model, Result};
use csv;
use failure::bail;
use failure::format_err;
use failure::ResultExt;
use log::info;
use navitia_poi_model::objects::{
    Coord, Poi as NavitiaPoi, PoiType as NavitiaPoiType, Property as NavitiaPoiProperty,
};
use serde_derive::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use std::path::Path;
use std::result::Result as StdResult;

pub const MAIN_FILE: &str = "POI_TCL.csv";
pub const PR_FILE: &str = "parcs_relais.csv";
pub const PV_FILE: &str = "parcs_velos.csv";

fn de_from_comma_float<'de, D>(deserializer: D) -> StdResult<f64, D::Error>
where
    D: serde::Deserializer<'de>,
{
    use serde::Deserialize;
    let s = String::deserialize(deserializer)?;
    s.replace(",", ".")
        .parse::<f64>()
        .map_err(serde::de::Error::custom)
}

fn de_non_empty_string<'de, D>(deserializer: D) -> StdResult<String, D::Error>
where
    D: serde::Deserializer<'de>,
{
    use serde::Deserialize;
    let s = String::deserialize(deserializer)?;
    match s.is_empty() {
        true => Err(serde::de::Error::custom(format_err!(
            "empty string not allowed in deserialization"
        ))),
        false => Ok(s),
    }
}

macro_rules! ctx_from_path {
    ($path:expr) => {
        |_| format!("Error reading {:?}", $path)
    };
}

#[derive(Debug, Deserialize, Serialize, Clone)]
struct Poi {
    #[serde(rename = "cod_typ_poi", deserialize_with = "de_non_empty_string")]
    poi_type: String,
    #[serde(rename = "lib_typ_poi")]
    poi_type_label: String,
    #[serde(rename = "Idt")]
    id_vr: Option<String>,
    #[serde(rename = "idt_poi")]
    id_main: Option<String>,
    #[serde(rename = "lib_poi")]
    label_main: Option<String>,
    #[serde(rename = "Lib")]
    label_vr: Option<String>,
    #[serde(rename = "cmt_poi")]
    comment: Option<String>,
    #[serde(rename = "adr")]
    address: Option<String>,
    #[serde(rename = "cod_pst")]
    postal_code: Option<String>,
    #[serde(rename = "lib_cmn")]
    city_label: Option<String>,
    #[serde(rename = "coo_x_wgs84", deserialize_with = "de_from_comma_float")]
    coord_x: f64,
    #[serde(rename = "coo_y_wgs84", deserialize_with = "de_from_comma_float")]
    coord_y: f64,
    #[serde(rename = "Capacite")]
    capacity: Option<u64>,
    #[serde(rename = "Place_Handi")]
    disabled_capacity: Option<u64>,
    #[serde(rename = "Horaires")]
    opening: Option<String>,
    #[serde(rename = "P_surv")]
    supervised: Option<String>,
    #[serde(rename = "lib_typ_pvel")]
    description: Option<String>,
}

fn add_poi_with_properties(
    sytral_poi: &Poi,
    poi_id: String,
    poi_label: String,
    poi_type: String,
    properties: Vec<NavitiaPoiProperty>,
    pois: &mut Vec<NavitiaPoi>,
) {
    let visible = !vec!["GAB", "DEP", "BET"].contains(&sytral_poi.poi_type.as_str());
    pois.push(NavitiaPoi {
        id: format!("TCL:{}:{}", sytral_poi.poi_type, poi_id),
        name: poi_label,
        coord: Coord::new(sytral_poi.coord_x, sytral_poi.coord_y),
        poi_type_id: poi_type,
        properties,
        visible,
    });
}

fn get_poi_id_without_collision(
    poi_id: &Option<String>,
    poi_ids: &mut HashSet<String>,
    file: &str,
) -> Result<String> {
    let poi_id = match poi_id.as_ref() {
        Some(val) => val.clone(),
        None => bail!("poi with undefined id found in file {}", file),
    };
    if !poi_ids.insert(poi_id.clone().into()) {
        bail!(
            "poi with id {:?} found at least twice in file {:?}",
            poi_id,
            file
        );
    }
    Ok(poi_id)
}

fn extract_from_main_file<P: AsRef<Path>>(
    dir_path: &P,
    pois: &mut Vec<NavitiaPoi>,
    poi_types: &mut HashMap<String, NavitiaPoiType>,
) -> Result<()> {
    info!("extract pois from file {}", MAIN_FILE);
    let main_file_path = dir_path.as_ref().join(MAIN_FILE);
    let mut poi_ids = HashSet::new();
    let mut rdr = csv::ReaderBuilder::new()
        .delimiter(b';')
        .from_path(&main_file_path)?;
    for sytral_poi in rdr.deserialize() {
        let sytral_poi: Poi = sytral_poi.with_context(ctx_from_path!(main_file_path))?;
        let poi_id = get_poi_id_without_collision(&sytral_poi.id_main, &mut poi_ids, MAIN_FILE)?;
        let mut properties = vec![];
        if let Some(desc) = &sytral_poi.comment {
            properties.push(NavitiaPoiProperty {
                key: "description".to_string(),
                value: desc.to_string(),
            });
        }
        if let Some(address) = &sytral_poi.address {
            properties.push(NavitiaPoiProperty {
                key: "addr:full".to_string(),
                value: address.to_string(),
            });
        }
        if let Some(postal_code) = &sytral_poi.postal_code {
            properties.push(NavitiaPoiProperty {
                key: "addr:postcode".to_string(),
                value: postal_code.to_string(),
            });
        }
        if let Some(city_label) = &sytral_poi.city_label {
            properties.push(NavitiaPoiProperty {
                key: "addr:city".to_string(),
                value: city_label.to_string(),
            });
        }
        let poi_type = format!("TCL:{}", sytral_poi.poi_type);
        poi_types.entry(poi_type.clone()).or_insert(NavitiaPoiType {
            id: poi_type.clone(),
            name: sytral_poi.poi_type_label.clone(),
        });
        add_poi_with_properties(
            &sytral_poi,
            poi_id,
            sytral_poi.label_main.clone().unwrap(),
            poi_type,
            properties,
            pois,
        );
    }
    Ok(())
}

fn extract_from_parcs_relais<P: AsRef<Path>>(
    dir_path: P,
    pois: &mut Vec<NavitiaPoi>,
    poi_types: &mut HashMap<String, NavitiaPoiType>,
) -> Result<()> {
    info!("extract pois from file {}", PR_FILE);
    let parcs_relais_file_path = dir_path.as_ref().join(PR_FILE);
    let mut poi_ids = HashSet::new();
    let mut rdr = csv::ReaderBuilder::new()
        .delimiter(b';')
        .from_path(&parcs_relais_file_path)?;
    for sytral_poi in rdr.deserialize() {
        let sytral_poi: Poi = sytral_poi.with_context(ctx_from_path!(parcs_relais_file_path))?;
        let poi_id = get_poi_id_without_collision(&sytral_poi.id_vr, &mut poi_ids, PR_FILE)?;
        let mut properties = vec![];
        if let Some(capacity) = &sytral_poi.capacity {
            properties.push(NavitiaPoiProperty {
                key: "capacity".to_string(),
                value: format!("{}", capacity),
            });
        }
        if let Some(disabled_capacity) = &sytral_poi.disabled_capacity {
            let mut disabled = "no".to_string();
            if *disabled_capacity > 0 {
                disabled = format!("{}", disabled_capacity);
            }
            properties.push(NavitiaPoiProperty {
                key: "capacity:disabled".to_string(),
                value: disabled,
            });
        }
        if let Some(opening) = &sytral_poi.opening {
            properties.push(NavitiaPoiProperty {
                key: "opening".to_string(),
                value: opening.to_string(),
            });
        }
        if let Some(supervised_raw) = &sytral_poi.supervised {
            let mut supervised = "no".to_string();
            if supervised_raw == "O" {
                supervised = "yes".to_string();
            }
            properties.push(NavitiaPoiProperty {
                key: "supervised".to_string(),
                value: supervised,
            });
        }
        properties.push(NavitiaPoiProperty {
            key: "operator".to_string(),
            value: "SYTRAL".to_string(),
        });
        properties.push(NavitiaPoiProperty {
            key: "network".to_string(),
            value: "TCL".to_string(),
        });
        properties.push(NavitiaPoiProperty {
            key: "ref".to_string(),
            value: poi_id.clone(),
        });
        let poi_type = "amenity:parking".to_string();
        poi_types.entry(poi_type.clone()).or_insert(NavitiaPoiType {
            id: poi_type.clone(),
            name: sytral_poi.poi_type_label.clone(),
        });
        add_poi_with_properties(
            &sytral_poi,
            poi_id,
            sytral_poi.label_vr.clone().unwrap(),
            poi_type,
            properties,
            pois,
        );
    }
    Ok(())
}

fn extract_from_parcs_velos<P: AsRef<Path>>(
    dir_path: P,
    pois: &mut Vec<NavitiaPoi>,
    poi_types: &mut HashMap<String, NavitiaPoiType>,
) -> Result<()> {
    info!("extract pois from file {}", PV_FILE);
    let parcs_velos_file_path = dir_path.as_ref().join(PV_FILE);
    let mut poi_ids = HashSet::new();
    let mut rdr = csv::ReaderBuilder::new()
        .delimiter(b';')
        .from_path(&parcs_velos_file_path)?;
    for sytral_poi in rdr.deserialize() {
        let sytral_poi: Poi = sytral_poi.with_context(ctx_from_path!(parcs_velos_file_path))?;
        let poi_id = get_poi_id_without_collision(&sytral_poi.id_vr, &mut poi_ids, PV_FILE)?;
        let mut properties = vec![];
        if let Some(capacity) = &sytral_poi.capacity {
            properties.push(NavitiaPoiProperty {
                key: "capacity".to_string(),
                value: format!("{}", capacity),
            });
        }
        if let Some(desc) = &sytral_poi.description {
            properties.push(NavitiaPoiProperty {
                key: "description".to_string(),
                value: desc.to_string(),
            });
        }
        let poi_type = "amenity:bicycle_parking".to_string();
        poi_types.entry(poi_type.clone()).or_insert(NavitiaPoiType {
            id: poi_type.clone(),
            name: sytral_poi.poi_type_label.clone(),
        });
        add_poi_with_properties(
            &sytral_poi,
            poi_id,
            sytral_poi.label_vr.clone().unwrap(),
            poi_type,
            properties,
            pois,
        );
    }
    Ok(())
}

pub fn extract_pois<P: AsRef<Path>>(sytral_path: P) -> Result<Model> {
    info!("Extracting pois from sytral");
    let mut pois: Vec<NavitiaPoi> = vec![];
    let mut poi_types: HashMap<String, NavitiaPoiType> = HashMap::new();
    for required_file in vec![MAIN_FILE, PR_FILE, PV_FILE] {
        if !sytral_path.as_ref().join(required_file).exists() {
            bail!("missing file {}", required_file)
        }
    }
    extract_from_main_file(&sytral_path, &mut pois, &mut poi_types)?;
    extract_from_parcs_relais(&sytral_path, &mut pois, &mut poi_types)?;
    extract_from_parcs_velos(&sytral_path, &mut pois, &mut poi_types)?;
    Ok(Model {
        pois,
        poi_types: poi_types.into_iter().map(|(_, p)| p).collect(),
    })
}
