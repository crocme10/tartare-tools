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

use std::collections::HashMap;
use transit_model::{
    model::Collections,
    objects::{Comment, CommentLinks, VehicleJourney},
    Result,
};
use typed_index_collection::{CollectionWithId, Id, Idx};

/// Merge the `Collections` parameter into the current `Collections` by consecutively merging
/// each collections representing the model.  Fails in case of id collision.
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
        mut physical_modes,
        mut stop_areas,
        mut stop_points,
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
    collections.contributors.try_merge(contributors)?;
    collections.datasets.try_merge(datasets)?;
    collections.networks.try_merge(networks)?;
    collections.commercial_modes.merge(commercial_modes);
    collections.lines.try_merge(lines)?;
    collections.routes.try_merge(routes)?;
    collections.frequencies.merge(frequencies);
    for physical_mode in physical_modes.take() {
        if collections.physical_modes.contains_id(&physical_mode.id) {
            // We already check the ID is present, unwrap is safe
            let mut existing_physical_mode = collections
                .physical_modes
                .get_mut(&physical_mode.id)
                .unwrap();
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
            collections.physical_modes.push(physical_mode).unwrap();
        }
    }

    collections.prices_v1.merge(prices_v1);
    collections.od_fares_v1.merge(od_fares_v1);
    collections.fares_v1.merge(fares_v1);
    collections.tickets.try_merge(tickets)?;
    collections.ticket_uses.try_merge(ticket_uses)?;
    collections.ticket_prices.merge(ticket_prices);
    collections
        .ticket_use_perimeters
        .merge(ticket_use_perimeters);
    collections
        .ticket_use_restrictions
        .merge(ticket_use_restrictions);
    collections.pathways.merge(pathways);
    collections.levels.merge(levels);
    collections.grid_calendars.try_merge(grid_calendars)?;
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

    // update comment idx of collection
    fn update_comment_idx<T: CommentLinks + Id<T>>(
        collection: &mut CollectionWithId<T>,
        c_idx_to_id: &HashMap<Idx<Comment>, String>,
        comments: &CollectionWithId<Comment>,
    ) {
        let mut objs = collection.take();
        for obj in &mut objs {
            *obj.comment_links_mut() = obj
                .comment_links()
                .iter()
                .filter_map(|c_idx| get_new_idx(*c_idx, c_idx_to_id, comments))
                .collect();
        }

        *collection = CollectionWithId::new(objs).unwrap();
    }

    let sp_idx_to_id = idx_to_id(&stop_points);
    let vj_idx_to_id = idx_to_id(&vehicle_journeys);
    let c_idx_to_id = idx_to_id(&comments);

    collections.comments.try_merge(comments)?;
    update_comment_idx(&mut stop_points, &c_idx_to_id, &collections.comments);
    update_comment_idx(&mut stop_areas, &c_idx_to_id, &collections.comments);

    collections.stop_points.try_merge(stop_points)?;
    collections.stop_areas.try_merge(stop_areas)?;

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

    fn update_vj_idx<'a, T: Clone>(
        map: &'a HashMap<(Idx<VehicleJourney>, u32), T>,
        vjs: &'a CollectionWithId<VehicleJourney>,
        vj_idx_to_id: &'a HashMap<Idx<VehicleJourney>, String>,
    ) -> impl Iterator<Item = ((Idx<VehicleJourney>, u32), T)> + 'a {
        map.iter()
            .filter_map(move |((old_vj_idx, sequence), value)| {
                get_new_idx(*old_vj_idx, vj_idx_to_id, vjs)
                    .map(|new_vj_idx| ((new_vj_idx, *sequence), value.clone()))
            })
    }

    // Update vehicle journey idx
    collections.stop_time_headsigns.extend(update_vj_idx(
        &stop_time_headsigns,
        &collections.vehicle_journeys,
        &vj_idx_to_id,
    ));

    collections.stop_time_ids.extend(update_vj_idx(
        &stop_time_ids,
        &collections.vehicle_journeys,
        &vj_idx_to_id,
    ));

    let mut new_stop_time_comments = HashMap::new();
    for ((old_vj_idx, sequence), value) in &stop_time_comments {
        let new_vj_idx =
            get_new_idx(*old_vj_idx, &vj_idx_to_id, &collections.vehicle_journeys).unwrap();
        let new_c_idx = get_new_idx(*value, &c_idx_to_id, &collections.comments).unwrap();
        new_stop_time_comments.insert((new_vj_idx, *sequence), new_c_idx);
    }
    collections
        .stop_time_comments
        .extend(new_stop_time_comments);
    collections.calendars.try_merge(calendars)?;
    collections.companies.try_merge(companies)?;
    collections.equipments.try_merge(equipments)?;
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
        use transit_model::{model::BUS_PHYSICAL_MODE, objects::PhysicalMode};

        #[test]
        fn physical_mode_co2_emission_max() {
            let physical_mode1 = PhysicalMode {
                id: String::from(BUS_PHYSICAL_MODE),
                name: String::from("Bus"),
                co2_emission: Some(21f32),
            };
            let physical_mode2 = PhysicalMode {
                id: String::from(BUS_PHYSICAL_MODE),
                name: String::from("Bus"),
                co2_emission: Some(42f32),
            };
            let mut collections = Collections::default();
            collections.physical_modes.push(physical_mode1).unwrap();
            let mut collections_to_merge = Collections::default();
            collections_to_merge
                .physical_modes
                .push(physical_mode2)
                .unwrap();
            let collections = try_merge_collections(collections, collections_to_merge).unwrap();
            let bus_mode = collections.physical_modes.get(BUS_PHYSICAL_MODE).unwrap();
            assert_relative_eq!(42f32, bus_mode.co2_emission.unwrap());
        }

        #[test]
        fn physical_mode_co2_emission_one_missing() {
            let physical_mode1 = PhysicalMode {
                id: String::from(BUS_PHYSICAL_MODE),
                name: String::from("Bus"),
                co2_emission: None,
            };
            let physical_mode2 = PhysicalMode {
                id: String::from(BUS_PHYSICAL_MODE),
                name: String::from("Bus"),
                co2_emission: Some(42f32),
            };
            let mut collections = Collections::default();
            collections.physical_modes.push(physical_mode1).unwrap();
            let mut collections_to_merge = Collections::default();
            collections_to_merge
                .physical_modes
                .push(physical_mode2)
                .unwrap();
            let collections = try_merge_collections(collections, collections_to_merge).unwrap();
            let bus_mode = collections.physical_modes.get(BUS_PHYSICAL_MODE).unwrap();
            assert_relative_eq!(42f32, bus_mode.co2_emission.unwrap());
        }
    }
}
