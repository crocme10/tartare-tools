use crate::unify::Unify;
use std::collections::HashMap;
use transit_model::{model::Collections, objects::PhysicalMode, Result};
use typed_index_collection::{CollectionWithId, Id, Idx};

fn merge_physical_modes(
    physical_modes: &mut CollectionWithId<PhysicalMode>,
    mut extend: CollectionWithId<PhysicalMode>,
) {
    for physical_mode in extend.take() {
        if physical_modes.contains_id(&physical_mode.id) {
            // We already check the ID is present, unwrap is safe
            let mut existing_physical_mode = physical_modes.get_mut(&physical_mode.id).unwrap();
            existing_physical_mode.co2_emission = match (
                existing_physical_mode.co2_emission,
                physical_mode.co2_emission,
            ) {
                (Some(e), Some(n)) => Some(e.max(n)),
                (Some(e), None) => Some(e),
                (None, Some(n)) => Some(n),
                (None, None) => None,
            }
        } else {
            // We already check the ID is not present, unwrap is safe
            physical_modes.push(physical_mode).unwrap();
        }
    }
}

/// Merge the `Collections` parameter into the current `Collections` by consecutively merging
/// each collections representing the model. Here are the merging rules:
/// - Ignore duplicate identifiers for:
///   * companies
///   * lines
///   * modes (physical and commercial)
///   * networks
///   * operators
///   * routes
///   * stops (stop point, stop area, zone, entrance/exit, generic node, and boarding area)
pub fn try_merge_collections(
    mut collections: Collections,
    extend: Collections,
) -> Result<Collections> {
    let Collections {
        contributors,
        datasets,
        networks,
        commercial_modes,
        lines,
        routes,
        mut vehicle_journeys,
        frequencies,
        physical_modes,
        stop_areas,
        stop_points,
        stop_locations,
        calendars,
        companies,
        comments,
        equipments,
        transfers,
        trip_properties,
        geometries,
        admin_stations,
        stop_time_headsigns,
        stop_time_ids,
        stop_time_comments,
        prices_v1,
        od_fares_v1,
        fares_v1,
        tickets,
        ticket_uses,
        ticket_prices,
        ticket_use_perimeters,
        ticket_use_restrictions,
        pathways,
        levels,
        grid_calendars,
        grid_exception_dates,
        grid_periods,
        grid_rel_calendar_line,
        ..
    } = extend;
    collections.contributors.merge(contributors);
    collections.datasets.merge(datasets);
    collections.companies.merge(companies);
    collections.networks.unify(networks);
    collections.commercial_modes.merge(commercial_modes);
    collections.lines.unify(lines);
    collections.routes.unify(routes);
    merge_physical_modes(&mut collections.physical_modes, physical_modes);
    collections.frequencies.merge(frequencies);

    collections.prices_v1.merge(prices_v1);
    collections.od_fares_v1.merge(od_fares_v1);
    collections.fares_v1.merge(fares_v1);
    collections.tickets.merge(tickets);
    collections.ticket_uses.merge(ticket_uses);
    collections.ticket_prices.merge(ticket_prices);
    collections
        .ticket_use_perimeters
        .merge(ticket_use_perimeters);
    collections
        .ticket_use_restrictions
        .merge(ticket_use_restrictions);
    collections.pathways.merge(pathways);
    collections.levels.merge(levels);
    collections.grid_calendars.merge(grid_calendars);
    collections.grid_exception_dates.merge(grid_exception_dates);
    collections.grid_periods.merge(grid_periods);
    collections
        .grid_rel_calendar_line
        .merge(grid_rel_calendar_line);

    fn get_new_idx<T>(
        old_idx: Idx<T>,
        old_idx_to_id: &HashMap<Idx<T>, String>,
        merged_collection: &CollectionWithId<T>,
    ) -> Option<Idx<T>> {
        old_idx_to_id
            .get(&old_idx)
            .and_then(|id| merged_collection.get_idx(id))
    }
    fn idx_to_id<T: Id<T>>(collection: &CollectionWithId<T>) -> HashMap<Idx<T>, String> {
        collection
            .iter()
            .map(|(idx, obj)| (idx, obj.id().into()))
            .collect()
    }

    let sp_idx_to_id = idx_to_id(&stop_points);

    collections.comments.try_merge(comments)?;

    collections.stop_points.unify(stop_points);
    collections.stop_areas.unify(stop_areas);
    collections.stop_locations.unify(stop_locations);

    // Update stop point idx in new stop times
    let mut vjs = vehicle_journeys.take();
    for vj in &mut vjs {
        for st in &mut vj.stop_times.iter_mut() {
            if let Some(new_idx) =
                get_new_idx(st.stop_point_idx, &sp_idx_to_id, &collections.stop_points)
            {
                st.stop_point_idx = new_idx;
            }
        }
    }
    vehicle_journeys = CollectionWithId::new(vjs)?;
    collections.vehicle_journeys.try_merge(vehicle_journeys)?;
    collections.stop_time_headsigns.extend(stop_time_headsigns);
    collections.stop_time_ids.extend(stop_time_ids);
    collections.stop_time_comments.extend(stop_time_comments);
    collections.calendars.try_merge(calendars)?;
    collections.equipments.merge(equipments);
    collections.transfers.merge(transfers);
    collections.trip_properties.try_merge(trip_properties)?;
    collections.geometries.try_merge(geometries)?;
    collections.admin_stations.merge(admin_stations);
    Ok(collections)
}

#[cfg(test)]
mod tests {
    use super::*;
    use approx::assert_relative_eq;

    mod physical_mode {
        use super::*;

        #[test]
        fn physical_mode_co2_emission_max() {
            let mut physical_modes = CollectionWithId::from(PhysicalMode {
                id: String::from("Bus"),
                name: String::from("Bus"),
                co2_emission: Some(21f32),
            });
            let extend = CollectionWithId::from(PhysicalMode {
                id: String::from("Bus"),
                name: String::from("Bus"),
                co2_emission: Some(42f32),
            });
            merge_physical_modes(&mut physical_modes, extend);
            let bus_mode = physical_modes.get("Bus").unwrap();
            assert_relative_eq!(42f32, bus_mode.co2_emission.unwrap());
        }

        #[test]
        fn physical_mode_co2_emission_one_missing() {
            let mut physical_modes = CollectionWithId::from(PhysicalMode {
                id: String::from("Bus"),
                name: String::from("Bus"),
                co2_emission: None,
            });
            let extend = CollectionWithId::from(PhysicalMode {
                id: String::from("Bus"),
                name: String::from("Bus"),
                co2_emission: Some(42f32),
            });
            merge_physical_modes(&mut physical_modes, extend);
            let bus_mode = physical_modes.get("Bus").unwrap();
            assert_relative_eq!(42f32, bus_mode.co2_emission.unwrap());
        }
    }
}
