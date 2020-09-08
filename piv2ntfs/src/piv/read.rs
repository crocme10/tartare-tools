use failure::format_err;
use log::{info, Level as LogLevel};
use serde::Deserialize;
use serde_json::Value;
use skip_error::skip_error_and_log;
use std::{fs::File, path::Path};
use transit_model::{
    model::Collections,
    objects::{CommercialMode, Line, Network, Route as NtfsRoute},
    Result,
};
use typed_index_collection::{CollectionWithId, Error::*};
use walkdir::WalkDir;

#[derive(Clone, Debug, Deserialize)]
struct Marque {
    code: String,
    libelle: String,
}

impl Into<Network> for Marque {
    fn into(self) -> Network {
        Network {
            id: self.code,
            name: self.libelle,
            ..Default::default()
        }
    }
}

impl Into<CommercialMode> for Marque {
    fn into(self) -> CommercialMode {
        CommercialMode {
            id: self.code,
            name: self.libelle,
        }
    }
}

#[derive(Clone, Debug, Deserialize)]
struct Ligne {
    #[serde(rename = "idLigne")]
    id: String,
    #[serde(rename = "libelleLigne")]
    name: String,
    #[serde(rename = "codeLigne")]
    code: Option<String>,
}

impl Into<Line> for Ligne {
    fn into(self) -> Line {
        Line {
            id: self.id,
            name: self.name,
            code: self.code,
            ..Default::default()
        }
    }
}

impl Into<NtfsRoute> for Ligne {
    fn into(self) -> NtfsRoute {
        NtfsRoute {
            id: self.id.clone(),
            line_id: self.id,
            name: self.name,
            ..Default::default()
        }
    }
}

#[derive(Clone, Debug, Deserialize)]
struct Route {
    ligne: Ligne,
}

#[derive(Clone, Debug, Deserialize)]
struct Parcours {
    route: Route,
}

#[derive(Debug, Deserialize)]
struct VehicleDescription {
    marque: Marque,
    parcours: Parcours,
}

fn get_or_create_network_id(
    network_description: &Marque,
    networks: &mut CollectionWithId<Network>,
) -> String {
    match networks.push(network_description.clone().into()) {
        Ok(idx) => networks[idx].id.clone(),
        Err(IdentifierAlreadyExists(id)) => id,
    }
}

fn get_or_create_commercial_mode_id(
    commercial_mode_description: &Marque,
    commercial_modes: &mut CollectionWithId<CommercialMode>,
) -> String {
    match commercial_modes.push(commercial_mode_description.clone().into()) {
        Ok(idx) => commercial_modes[idx].id.clone(),
        Err(IdentifierAlreadyExists(id)) => id,
    }
}

fn get_or_create_line_id(
    line_description: &Ligne,
    lines: &mut CollectionWithId<Line>,
    network_id: String,
    commercial_mode_id: String,
) -> String {
    let mut line: Line = line_description.clone().into();
    line.network_id = network_id;
    line.commercial_mode_id = commercial_mode_id;
    match lines.push(line) {
        Ok(idx) => lines[idx].id.clone(),
        Err(IdentifierAlreadyExists(id)) => id,
    }
}

fn get_or_create_route_id(
    line_description: &Ligne,
    routes: &mut CollectionWithId<NtfsRoute>,
) -> String {
    match routes.push(line_description.clone().into()) {
        Ok(idx) => routes[idx].id.clone(),
        Err(IdentifierAlreadyExists(id)) => id,
    }
}

fn manage_vehicle_content(
    vehicle_description: VehicleDescription,
    collections: &mut Collections,
) -> Result<()> {
    let network_id =
        get_or_create_network_id(&vehicle_description.marque, &mut collections.networks);
    let commercial_mode_id = get_or_create_commercial_mode_id(
        &vehicle_description.marque,
        &mut collections.commercial_modes,
    );
    get_or_create_line_id(
        &vehicle_description.parcours.route.ligne,
        &mut collections.lines,
        network_id,
        commercial_mode_id,
    );
    let _route_id = get_or_create_route_id(
        &vehicle_description.parcours.route.ligne,
        &mut collections.routes,
    );
    Ok(())
}

pub fn read_daily_transportation_plan(
    daily_folder: &Path,
    collections: &mut Collections,
) -> Result<()> {
    for file in WalkDir::new(daily_folder)
        .into_iter()
        .filter_map(std::result::Result::ok)
        .filter(|dir_entry| dir_entry.file_type().is_file())
    {
        let file_path = file.path();
        info!("Reading {:?}", file_path.file_name().unwrap_or_default());
        let daily_transportation_plan = File::open(file_path)
            .map_err(|e| format_err!("{}", e))
            .and_then(|file| {
                serde_json::from_reader::<_, Vec<Value>>(file).map_err(|e| format_err!("{}", e))
            })?;
        for vehicle_content in daily_transportation_plan {
            let vehicle_description: VehicleDescription = skip_error_and_log!(
                serde_json::from_value(vehicle_content.clone()).map_err(|e| format_err!(
                    "unvalid train {}: {}",
                    vehicle_content["numero"],
                    e
                )),
                LogLevel::Warn
            );
            manage_vehicle_content(vehicle_description, collections)?;
        }
    }
    Ok(())
}
