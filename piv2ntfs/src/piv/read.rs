use failure::format_err;
use log::info;
use serde::Deserialize;
use std::{fs::File, path::Path};
use transit_model::{model::Collections, Result};
use walkdir::WalkDir;

#[derive(Debug, Deserialize)]
struct VehicleDescription {}

fn manage_vehicle_content(
    _vehicle_description: VehicleDescription,
    _collections: &mut Collections,
) -> Result<()> {
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
                serde_json::from_reader::<_, Vec<VehicleDescription>>(file)
                    .map_err(|e| format_err!("{}", e))
            })?;
        for vehicle_description in daily_transportation_plan {
            manage_vehicle_content(vehicle_description, collections)?;
        }
    }
    Ok(())
}
