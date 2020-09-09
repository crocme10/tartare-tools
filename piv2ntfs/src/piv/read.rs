use chrono::{
    naive::{MAX_DATE, MIN_DATE},
    DateTime, FixedOffset, NaiveDate, Timelike,
};
use failure::format_err;
use log::{info, Level as LogLevel};
use serde::Deserialize;
use serde_json::Value;
use skip_error::skip_error_and_log;
use std::{cmp, collections::BTreeSet, convert::TryFrom, fs::File, path::Path};
use transit_model::{
    model::Collections,
    objects::{Route as NtfsRoute, *},
    validity_period, Result,
};
use typed_index_collection::{CollectionWithId, Error::*, Id, Idx};
use walkdir::WalkDir;

/// Deserialize string datetime (Y-m-dTH:M:Sz) to NaiveDateTime
fn de_from_datetime_string<'de, D>(deserializer: D) -> Result<DateTime<FixedOffset>, D::Error>
where
    D: serde::Deserializer<'de>,
{
    let s = String::deserialize(deserializer)?;
    DateTime::parse_from_rfc3339(&s).map_err(serde::de::Error::custom)
}

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
    name: Option<String>,
    #[serde(rename = "codeLigne")]
    code: Option<String>,
}

impl Into<Line> for Ligne {
    fn into(self) -> Line {
        Line {
            id: self.id,
            name: self.name.unwrap_or_default(),
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
            name: self.name.unwrap_or_default(),
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

#[derive(Clone, Debug, Deserialize, PartialEq, Eq)]
struct Horaire {
    #[serde(rename = "dateHeure", deserialize_with = "de_from_datetime_string")]
    date_heure: DateTime<FixedOffset>,
    evenement: Option<Evenement>,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Eq)]
struct Emplacement {
    code: String,
    libelle: Option<String>,
}

impl Into<StopPoint> for Emplacement {
    fn into(self) -> StopPoint {
        StopPoint {
            id: self.code.clone(),
            name: self.libelle.unwrap_or_default(),
            stop_area_id: self.code,
            ..Default::default()
        }
    }
}

impl Into<StopArea> for Emplacement {
    fn into(self) -> StopArea {
        StopArea {
            id: self.code,
            name: self.libelle.unwrap_or_default(),
            ..Default::default()
        }
    }
}

#[derive(Clone, Debug, Deserialize, PartialEq, Eq)]
struct Desserte {
    arrivee: Option<Horaire>,
    depart: Option<Horaire>,
    emplacement: Emplacement,
    rang: u32,
    #[serde(rename = "indicateurMonteeInterdite", default)]
    pickup: bool,
    #[serde(rename = "indicateurDescenteInterdite", default)]
    drop_off: bool,
}

impl cmp::PartialOrd for Desserte {
    fn partial_cmp(&self, other: &Self) -> Option<cmp::Ordering> {
        self.rang.partial_cmp(&other.rang)
    }
}

impl cmp::Ord for Desserte {
    fn cmp(&self, other: &Self) -> cmp::Ordering {
        self.rang.cmp(&other.rang)
    }
}

#[derive(Clone, Debug, Deserialize)]
struct ListeArretsDesserte {
    #[serde(rename = "arret")]
    arrets: BTreeSet<Desserte>,
}

#[derive(Clone, Debug, Deserialize)]
struct ModeTransport {
    #[serde(rename = "codeMode")]
    code_mode: String,
    #[serde(rename = "libelleMode")]
    libelle_mode: String,
    #[serde(rename = "codeSousMode")]
    code_sous_mode: String,
    #[serde(rename = "libelleSousMode")]
    libelle_sous_mode: String,
    #[serde(rename = "typeMode")]
    type_mode: String,
}

impl Into<PhysicalMode> for ModeTransport {
    fn into(self) -> PhysicalMode {
        match (
            self.code_mode.to_lowercase().as_str(),
            self.code_sous_mode.to_lowercase().as_str(),
            self.type_mode.to_lowercase().as_str(),
        ) {
            ("bus", _, "routier") => PhysicalMode {
                id: "Bus".to_string(),
                name: "Bus".to_string(),
                ..Default::default()
            },
            ("coach", _, "routier") => PhysicalMode {
                id: "Coach".to_string(),
                name: "Autocar".to_string(),
                ..Default::default()
            },
            ("rail", "local", "ferre") => PhysicalMode {
                id: "LocalTrain".to_string(),
                name: "Train".to_string(),
                ..Default::default()
            },
            ("rail", "railshuttle", "ferre") => PhysicalMode {
                id: "RailShuttle".to_string(),
                name: "Navette ferrée".to_string(),
                ..Default::default()
            },
            ("rail", "regionalrail", "ferre") => PhysicalMode {
                id: "Train".to_string(),
                name: "TER / Intercités".to_string(),
                ..Default::default()
            },
            ("rail", "suburbanrailway", "ferre") => PhysicalMode {
                id: "RapidTransit".to_string(),
                name: "RER / Transilien".to_string(),
                ..Default::default()
            },
            ("rail", "tramtrain", "ferre") => PhysicalMode {
                id: "Tram".to_string(),
                name: "Tramway".to_string(),
                ..Default::default()
            },
            ("tram", "tramtrain", "ferre") => PhysicalMode {
                id: "Tram".to_string(),
                name: "Tramway".to_string(),
                ..Default::default()
            },
            _ => PhysicalMode {
                id: "LongDistanceTrain".to_string(),
                name: "Train grande vitesse".to_string(),
                ..Default::default()
            },
        }
    }
}

#[derive(Clone, Debug, Deserialize)]
struct Operateur {
    #[serde(rename = "codeOperateur")]
    code_operateur: String,
    #[serde(rename = "libelleOperateur")]
    libelle_operateur: String,
}

impl Into<Company> for Operateur {
    fn into(self) -> Company {
        Company {
            id: self.code_operateur,
            name: self.libelle_operateur,
            ..Default::default()
        }
    }
}

#[derive(Debug, Deserialize, PartialEq)]
enum PlanTransportSource {
    OPE,
    PTP,
    PTA,
}

#[derive(Copy, Clone, Debug, Deserialize, PartialEq, Eq)]
enum EvenementType {
    #[serde(rename = "NORMAL")]
    Normal,
    #[serde(rename = "CREATION")]
    Creation,
    #[serde(rename = "MODIFICATION_DESSERTE_AJOUTEE")]
    ModificationDesserteAjoutee,
    #[serde(rename = "MODIFICATION_DESSERTE_SUPPRIMEE")]
    MoficationDesserteSupprimee,
    #[serde(rename = "MODIFICATION_PROLONGATION")]
    ModificationProlongation,
    #[serde(rename = "MODIFICATION_LIMITATION")]
    ModificationLimitation,
    #[serde(rename = "MODIFICATION_DETOURNEMENT")]
    ModificationDetournement,
    #[serde(rename = "SUPPRESSION")]
    Suppression,
    #[serde(rename = "RETARD")]
    Retard,
    #[serde(rename = "RETARD_OBSERVE")]
    RetardObserve,
    #[serde(rename = "RETARD_PROJETE")]
    RetardProjete,
    #[serde(rename = "SUPPRESSION_TOTALE")]
    SuppressionTotale,
    #[serde(rename = "SUPPRESSION_PARTIELLE")]
    SuppressionPartielle,
    #[serde(rename = "SUPPRESSION_DETOURNEMENT")]
    SuppressionDetournement,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Eq)]
struct Evenement {
    #[serde(rename = "type")]
    evenement_type: EvenementType,
    #[serde(rename = "codeCategorie")]
    code_categorie: String,
    #[serde(rename = "libelleCategorie")]
    libelle_categorie: String,
}

#[derive(Debug, Deserialize)]
struct VehicleDescription {
    marque: Marque,
    parcours: Parcours,
    #[serde(rename = "modeTransport")]
    mode_transport: ModeTransport,
    #[serde(rename = "listeArretsDesserte")]
    liste_arrets_desserte: ListeArretsDesserte,
    numero: String,
    operateur: Operateur,
    #[serde(rename = "dateCirculation")]
    date_circulation: String,
    #[serde(rename = "planTransportSource")]
    plan_transport_source: PlanTransportSource,
    #[serde(rename = "evenement")]
    evenements: Option<Vec<Evenement>>,
    #[serde(rename = "codeCirculation")]
    code_circulation: String,
}

impl VehicleDescription {
    fn departure_time(&self) -> Option<DateTime<FixedOffset>> {
        self.liste_arrets_desserte
            .arrets
            .iter()
            .next()
            .and_then(|arret| arret.depart.as_ref())
            .map(|horaire| horaire.date_heure)
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

fn get_or_create_object_idx<T, E: Clone + Into<T>>(
    element: &E,
    collection: &mut CollectionWithId<T>,
) -> Idx<T>
where
    T: Id<T>,
{
    collection
        .push(element.clone().into())
        .unwrap_or_else(|error| match error {
            // Can `.unwrap()` since we know the identifier exists.
            IdentifierAlreadyExists(id) => collection.get_idx(&id).unwrap(),
        })
}

fn get_or_create_service_id(
    circulation_date: NaiveDate,
    calendars: &mut CollectionWithId<Calendar>,
) -> String {
    let circulation_id = circulation_date.format("%Y%m%d").to_string();
    if !calendars.contains_id(&circulation_id) {
        let mut calendar = Calendar::new(circulation_id.clone());
        calendar.dates.insert(circulation_date);
        let _ = calendars.push(calendar);
    }
    circulation_id
}

fn set_validity_period(circulation_date: NaiveDate, validity_period: &mut ValidityPeriod) {
    validity_period.start_date = cmp::min(validity_period.start_date, circulation_date);
    validity_period.end_date = cmp::max(validity_period.end_date, circulation_date);
}

fn get_or_create_stop_point_stop_area_idx(
    emplacement_description: &Emplacement,
    physical_mode_id: &str,
    collections: &mut Collections,
) -> Idx<StopPoint> {
    let stop_area_idx =
        get_or_create_object_idx(emplacement_description, &mut collections.stop_areas);

    let mut stop_point: StopPoint = emplacement_description.clone().into();
    stop_point.id = format!("{}:{}", stop_point.id, physical_mode_id);

    let stop_point_idx = match collections.stop_points.push(stop_point) {
        Ok(idx) => idx,
        Err(IdentifierAlreadyExists(id)) => collections.stop_points.get_idx(&id).unwrap(),
    };

    if let Some(name) = &emplacement_description.libelle {
        let mut stop_area = collections.stop_areas.index_mut(stop_area_idx);
        if stop_area.name.is_empty() {
            stop_area.name = name.clone();
        }
        let mut stop_point = collections.stop_points.index_mut(stop_point_idx);
        if stop_point.name.is_empty() {
            stop_point.name = name.clone();
        }
    }
    stop_point_idx
}

fn fill_stop_times(
    vehicle_journey: &mut VehicleJourney,
    vehicle_description: &VehicleDescription,
    collections: &mut Collections,
) -> Result<()> {
    // Midnight of the current day.
    let base_time = vehicle_description
        .departure_time()
        .and_then(|dt| dt.with_hour(0))
        .and_then(|dt| dt.with_minute(0))
        .and_then(|dt| dt.with_second(0))
        .ok_or_else(|| {
            format_err!(
                "Failed to calculate circulation date for vehicle journey '{}'.",
                vehicle_journey.id
            )
        })?;
    let time_since_base =
        |horaire: &Option<Horaire>| -> Result<Option<Time>, std::num::TryFromIntError> {
            horaire
                .as_ref()
                .map(|horaire| {
                    u32::try_from(
                        horaire
                            .date_heure
                            .signed_duration_since(base_time)
                            .num_seconds(),
                    )
                    .map(|elapsed_seconds| Time::new(0, 0, elapsed_seconds))
                })
                .transpose()
        };
    for desserte in &vehicle_description.liste_arrets_desserte.arrets {
        let opt_departure_time = time_since_base(&desserte.depart)?;
        let opt_arrival_time = time_since_base(&desserte.arrivee)?;
        let (departure_time, arrival_time, pickup_type, drop_off_type) =
            match (opt_departure_time, opt_arrival_time) {
                (Some(departure_time), Some(arrival_time)) => (
                    departure_time,
                    arrival_time,
                    desserte.pickup.into(),
                    desserte.drop_off.into(),
                ),
                (Some(departure_time), None) => {
                    (departure_time, departure_time, desserte.pickup.into(), 1)
                }
                (None, Some(arrival_time)) => {
                    (arrival_time, arrival_time, 1, desserte.drop_off.into())
                }
                _ => continue,
            };

        let stop_point_idx = get_or_create_stop_point_stop_area_idx(
            &desserte.emplacement,
            &vehicle_journey.physical_mode_id,
            collections,
        );
        vehicle_journey.stop_times.push(StopTime {
            stop_point_idx,
            departure_time,
            arrival_time,
            sequence: desserte.rang,
            boarding_duration: 0,
            alighting_duration: 0,
            pickup_type,
            drop_off_type,
            datetime_estimated: false,
            local_zone_id: None,
            precision: Some(StopTimePrecision::Exact),
        });
    }
    Ok(())
}

fn manage_vehicle_content(
    vehicle_description: &VehicleDescription,
    collections: &mut Collections,
    validity_period: &mut ValidityPeriod,
) -> Result<()> {
    let physical_mode_idx = get_or_create_object_idx(
        &vehicle_description.mode_transport,
        &mut collections.physical_modes,
    );
    let company_idx =
        get_or_create_object_idx(&vehicle_description.operateur, &mut collections.companies);

    let company_id = collections.companies[company_idx].id.clone();
    let physical_mode_id = collections.physical_modes[physical_mode_idx].id.clone();
    let id = format!(
        "{}:{}:{}:{}",
        &vehicle_description.date_circulation,
        vehicle_description.numero,
        company_id,
        physical_mode_id,
    );

    let vj_date = vehicle_description
        .departure_time()
        .map(|dt| dt.date().naive_local())
        .ok_or_else(|| format_err!("Failed to get a service date for vehicle journey '{}'.", id))?;

    let network_idx =
        get_or_create_object_idx(&vehicle_description.marque, &mut collections.networks);
    let commercial_mode_idx = get_or_create_object_idx(
        &vehicle_description.marque,
        &mut collections.commercial_modes,
    );
    get_or_create_line_id(
        &vehicle_description.parcours.route.ligne,
        &mut collections.lines,
        collections.networks[network_idx].id.clone(),
        collections.commercial_modes[commercial_mode_idx].id.clone(),
    );
    let route_idx = get_or_create_object_idx(
        &vehicle_description.parcours.route.ligne,
        &mut collections.routes,
    );

    let service_id = get_or_create_service_id(vj_date, &mut collections.calendars);
    let dataset = collections.datasets.values().next().unwrap();

    let mut vehicle_journey = VehicleJourney {
        id,
        route_id: collections.routes[route_idx].id.clone(),
        physical_mode_id,
        company_id,
        service_id,
        dataset_id: dataset.id.clone(),
        ..Default::default()
    };

    vehicle_journey.codes.insert((
        "source".to_string(),
        vehicle_description.code_circulation.to_string(),
    ));
    fill_stop_times(&mut vehicle_journey, &vehicle_description, collections)?;
    collections.vehicle_journeys.push(vehicle_journey)?;
    set_validity_period(vj_date, validity_period);

    Ok(())
}

fn update_validity_period(
    datasets: &mut CollectionWithId<Dataset>,
    validity_period: &ValidityPeriod,
) -> Result<CollectionWithId<Dataset>> {
    let mut datasets = datasets.take();
    for dataset in &mut datasets {
        validity_period::set_dataset_validity_period(dataset, &validity_period);
    }
    CollectionWithId::new(datasets).map_err(|e| format_err!("{}", e))
}

pub fn read_daily_transportation_plan(
    daily_folder: &Path,
    collections: &mut Collections,
) -> Result<()> {
    let mut validity_period = ValidityPeriod {
        start_date: MAX_DATE,
        end_date: MIN_DATE,
    };
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
                    "Unvalid vehicle journey {}: {}",
                    vehicle_content["numero"],
                    e
                )),
                LogLevel::Warn
            );
            if vehicle_description.plan_transport_source == PlanTransportSource::PTA {
                skip_error_and_log!(
                    manage_vehicle_content(&vehicle_description, collections, &mut validity_period),
                    LogLevel::Warn
                );
            }
        }
    }
    collections.datasets = update_validity_period(&mut collections.datasets, &validity_period)?;
    Ok(())
}
