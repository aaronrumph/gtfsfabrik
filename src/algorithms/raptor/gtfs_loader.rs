// This module loads in GTFS data to use with RAPTOR

use crate::algorithms::raptor::transfers::calculate_naive_transfers;
use crate::algorithms::raptor::types::{
    IdMap, RaptorGtfsFeed, RaptorRoute, RaptorRouteID, RaptorRouteServingStop, RaptorStop, RaptorStopID,
    RaptorStopTime, RaptorTimetable, RaptorTransfer, RaptorTrip, RaptorTripID, RoutesServingStop,
};
use crate::gtfs::datetime::{gtfs_time_to_seconds, Seconds};
use crate::read_gtfs;
use crate::utils::errors::RaptorError;
use crate::utils::files::gtfs::{GtfsFiles, RouteColumns, StopColumns, StopTimesColumns, TripColumns};

use std::collections::HashMap;

/// Function to read in routes, stops, trips and stop times from a given GTFS feed into dataframe
pub fn load_gtfs(feed_dir: &str) -> Result<RaptorGtfsFeed, RaptorError> {
    // need to read in routes, trips, stops, and stop times
    let routes = read_gtfs!(feed_dir, GtfsFiles::Routes);
    let trips = read_gtfs!(feed_dir, GtfsFiles::Trips);
    let stops = read_gtfs!(feed_dir, GtfsFiles::Stops);
    let stop_times = read_gtfs!(feed_dir, GtfsFiles::StopTimes);

    let gtfs_feed = RaptorGtfsFeed {
        routes,
        trips,
        stops,
        stop_times,
    };

    Ok(gtfs_feed)
}

// This function maps GTFS ids to dense int ids to use for Raptor for stops, trips, and routes
pub fn map_ids(feed: &RaptorGtfsFeed) -> Result<IdMap, RaptorError> {
    // these maps translate between the gtfs ids and the int ids
    let mut routes: HashMap<String, RaptorRouteID> = HashMap::new();
    let mut trips: HashMap<String, RaptorTripID> = HashMap::new();
    let mut stops: HashMap<String, RaptorStopID> = HashMap::new();

    // TODO: optional optimization for case where ids are just ints

    // can use to_string() for columns (fun with enums)
    let stop_gtfs_ids = feed.stops.column(&StopColumns::StopID.to_string())?.str()?;
    let route_gtfs_ids = feed.routes.column(&RouteColumns::RouteID.to_string())?.str()?;
    let trip_gtfs_ids = feed.trips.column(&TripColumns::TripID.to_string())?.str()?;

    for (idx, gtfs_id) in stop_gtfs_ids.into_iter().enumerate() {
        if let Some(id) = gtfs_id {
            stops.insert(id.to_string(), RaptorStopID::new(idx));
        }
    }

    for (idx, gtfs_id) in route_gtfs_ids.into_iter().enumerate() {
        if let Some(id) = gtfs_id {
            routes.insert(id.to_string(), RaptorRouteID::new(idx));
        }
    }

    for (idx, gtfs_id) in trip_gtfs_ids.into_iter().enumerate() {
        if let Some(id) = gtfs_id {
            trips.insert(id.to_string(), RaptorTripID::new(idx));
        }
    }

    let id_map = IdMap { routes, trips, stops };

    Ok(id_map)
}

pub fn build_stops(feed: &RaptorGtfsFeed, id_map: &IdMap) -> Result<Vec<RaptorStop>, RaptorError> {
    // builds a continuous array of stops to be used in RAPTOR for marking
    let gtfs_stop_ids = feed.stops.column(&StopColumns::StopID.to_string())?.str()?;

    // NOTE: Will default to using gtfs_id for name if names not provided (GTFS spec says
    // they're optional)
    let stop_names = feed
        .stops
        .column(&StopColumns::Name.to_string())
        .ok()
        .and_then(|s| s.str().ok());

    let mut stops: Vec<Option<RaptorStop>> = vec![None; id_map.stops.len()];

    for (idx, id) in gtfs_stop_ids.into_iter().enumerate() {
        let Some(gtfs_id) = id else { continue };
        let raptor_id = id_map.stops[gtfs_id];
        let name = stop_names
            .as_ref()
            .and_then(|names| names.get(idx))
            .unwrap_or(gtfs_id)
            .to_string();
        stops[raptor_id.id] = Some(RaptorStop {
            stop_id: raptor_id,
            name,
        });
    }

    stops
        .into_iter()
        .enumerate()
        .map(|(idx, stop)| stop.ok_or(RaptorError::MissingStop(idx)))
        .collect()
}

// TODO: build_routes function to build routes and trips for raptor
// TODO: build_timetable function to build RaptorTimetable

// TODO: raptor_query function that wraps loader and simple raptor to give travel time
// TODO: itinerary function that wraps ... and returns full travel directions

// tuple with (sequence, stop_id, arrival_time, departure_time)
type StopTimesByTrip = HashMap<RaptorTripID, Vec<(usize, RaptorStopID, Seconds, Seconds)>>;

pub fn build_stop_times_by_trip(feed: &RaptorGtfsFeed, id_map: &IdMap) -> Result<StopTimesByTrip, RaptorError> {
    // most of the useful information for RAPTOR comes from stop times, so getting useful columns
    // out
    let stop_time_trip_ids = feed.stop_times.column(&StopTimesColumns::TripID.to_string())?.str()?;
    let stop_time_stop_ids = feed.stop_times.column(&StopTimesColumns::StopID.to_string())?.str()?;
    let stop_time_arrivals = feed
        .stop_times
        .column(&StopTimesColumns::ArrivalTime.to_string())?
        .str()?;
    let stop_time_departure = feed
        .stop_times
        .column(&StopTimesColumns::DepartureTime.to_string())?
        .str()?;
    let stop_time_sequences = feed
        .stop_times
        .column(&StopTimesColumns::StopSequence.to_string())?
        .str()?;

    // next making hashmap of stop times for each trip where key is trip_id and value is a
    // tuple with (sequence, stop_id, arrival_time, departure_time)
    let mut stop_times_by_trip_id: StopTimesByTrip = HashMap::new();

    // TODO: Use rayon on the building of the tuples and then move into a Hashmap after
    // NOTE: looks weird to use stop_time_trip_ids iterator to get all the stuff necessary to build the tuple, but
    // because they are all columns from same stop_times.txt, this is the same as going row by row,
    // but avoids weird polars type issues I encountered
    for idx in 0..stop_time_trip_ids.len() {
        // collecting trip_id, (sequence...) to build the hashmap above
        let gtfs_trip_id = match stop_time_trip_ids.get(idx) {
            Some(id) => id,
            None => {
                return Err(RaptorError::InvalidGtfs(format!(
                    "missing trip_id in stop_times.txt at row {}",
                    idx
                )))
            }
        };

        let gtfs_stop_id = match stop_time_stop_ids.get(idx) {
            Some(id) => id,
            None => {
                return Err(RaptorError::InvalidGtfs(format!(
                    "missing stop_id in stop_times.txt at row {}",
                    idx
                )))
            }
        };

        let trip_id = match id_map.trips.get(gtfs_trip_id) {
            Some(id) => *id,
            None => {
                return Err(RaptorError::InvalidGtfs(format!(
                    "trip_id '{}' in stop_times.txt not found in trips.txt",
                    gtfs_trip_id
                )))
            }
        };

        let stop_id = match id_map.stops.get(gtfs_stop_id) {
            Some(id) => *id,
            None => {
                return Err(RaptorError::InvalidGtfs(format!(
                    "stop_id '{}' in stop_times.txt not found in stops.txt",
                    gtfs_stop_id
                )))
            }
        };

        // TODO: !!! Assumes that arrival times and departure times are required, but GTFS Spec only says that
        // they're conditionally required. Need to add helper function to interpolate stop times when
        // missing either arrival or departure times!

        let arrival_time = match stop_time_arrivals.get(idx) {
            Some(time) => {
                gtfs_time_to_seconds(time).map_err(|e| RaptorError::StopTimeParsingError { row: idx, source: e })?
            }
            None => {
                let error_msg = format!("Missing arrival time in stop_times.txt for row {}", idx);
                return Err(RaptorError::InvalidGtfs(error_msg));
            }
        };

        let departure_time = match stop_time_departure.get(idx) {
            Some(time) => {
                gtfs_time_to_seconds(time).map_err(|e| RaptorError::StopTimeParsingError { row: idx, source: e })?
            }
            None => {
                let error_msg = format!("Missing arrival time in stop_times.txt for row {}", idx);
                return Err(RaptorError::InvalidGtfs(error_msg));
            }
        };

        let sequence: usize = match stop_time_sequences.get(idx) {
            Some(seq) => seq.parse::<usize>().map_err(|_| {
                RaptorError::InvalidGtfs(format!(
                    "invalid stop_sequence '{}' in stop_times.txt at row {}",
                    seq, idx
                ))
            })?,

            None => {
                return Err(RaptorError::InvalidGtfs(format!(
                    "missing stop_sequence in stop_times.txt at row {}",
                    idx
                )))
            }
        };

        // now build tuple
        let stop_time_entry = (sequence, stop_id, arrival_time, departure_time);

        // find the entry for this trip_id, if it doesn't already exist, then adding it
        let trip_stop_times = stop_times_by_trip_id.entry(trip_id).or_default();
        trip_stop_times.push(stop_time_entry);
    }
    // sort each trip's stop times by sequence — required for canonical sequence and alignment
    for times in stop_times_by_trip_id.values_mut() {
        times.sort_by_key(|stop_time| stop_time.0); // stop_time.0 is sequence
    }

    Ok(stop_times_by_trip_id)
}

/// Builds raptor usable routes for a GTFSFeed. Needs an IdMap piped into it aswell to convert
/// gtfs_ids to usable RAPTOR ids.
/// # Returns:
/// Tuple (raptor_routes, routes_serving_stops) for use in RaptorTimetable
pub fn build_routes(
    feed: &RaptorGtfsFeed,
    id_map: &IdMap,
) -> Result<(Vec<RaptorRoute>, Vec<RoutesServingStop>), RaptorError> {
    // returns result, if Ok(...) then returns tuple of 0: routes and 1:
    // routes_serving_stops for timetable building

    // NOTE: assumes that if arrival time for one route is earlier than another, than its departure
    // time is too

    // need to map all trips to the route they follow
    let trip_ids_col = feed.trips.column(&TripColumns::TripID.to_string())?.str()?;
    let route_ids_col = feed.trips.column(&TripColumns::RouteID.to_string())?.str()?;
    let mut trip_route_map: HashMap<RaptorTripID, RaptorRouteID> = HashMap::new();
    for idx in 0..trip_ids_col.len() {
        // need to check whether all trips have ids for raptor to work
        let trip_id = match trip_ids_col.get(idx) {
            Some(id) => id,
            None => {
                let error_msg = format!("Missing trip_id in trips.txt for row {}", idx);
                return Err(RaptorError::InvalidGtfs(error_msg));
            }
        };

        let route_id = match route_ids_col.get(idx) {
            Some(id) => id,
            None => {
                let error_msg = format!("Missing route_id in trips.txt for row {}", idx);
                return Err(RaptorError::InvalidGtfs(error_msg));
            }
        };

        let raptor_trip_id = match id_map.trips.get(trip_id) {
            Some(id) => *id,
            None => {
                return Err(RaptorError::InvalidGtfs(format!(
                    "Missing trip_id in trips.txt for row {}",
                    idx
                )))
            }
        };
        let raptor_route_id = match id_map.routes.get(route_id) {
            Some(id) => *id,
            None => {
                return Err(RaptorError::InvalidGtfs(format!(
                    "Missing route_id in trips.txt for row {}",
                    idx
                )))
            }
        };

        trip_route_map.insert(raptor_trip_id, raptor_route_id);
    }

    // next building routes using stop_times and trip info
    let stop_times_by_trip_id = build_stop_times_by_trip(feed, id_map)?;

    // grouping trips by the route they serve to build routes
    let mut trips_by_route: HashMap<RaptorRouteID, Vec<RaptorTripID>> = HashMap::new();
    for (trip_id, route_id) in &trip_route_map {
        // check if route_id already in hashmap, if not add it
        let trip_list = trips_by_route.entry(*route_id).or_default();
        // then add the trip_id to the vector
        trip_list.push(*trip_id);
    }

    // now can actually build the RaptorRoute obj for each route
    let num_stops = id_map.stops.len();
    let mut routes: Vec<RaptorRoute> = Vec::new();
    let mut routes_serving_stops: Vec<RoutesServingStop> = vec![Vec::new(); num_stops];
    let mut route_idx = 0usize;

    for (_gtfs_route_id, trip_ids_this_route) in &trips_by_route {
        // all trips on the same route have same stop seq, so using first trip to get stop order
        let first_trip_id = match trip_ids_this_route.first() {
            Some(trip) => trip,
            None => continue,
        };
        let stop_sequence = match stop_times_by_trip_id.get(first_trip_id) {
            Some(trip) => trip,
            None => continue,
        };

        // building the sequenced list of stops this route serves
        let mut route_stops: Vec<RaptorStopID> = Vec::new();
        for (_sequence, stop_id, _arrival, _departure) in stop_sequence {
            route_stops.push(*stop_id);
        }
        if route_stops.is_empty() {
            // ignore this route if no stops (can be unused route)
            continue;
        }

        // index for quickly finding a stop's position in this route
        let mut stop_positions: HashMap<RaptorStopID, usize> = HashMap::new();
        for (position, stop_id) in route_stops.iter().enumerate() {
            stop_positions.insert(*stop_id, position);
        }

        // now need to build a RaptorTrip obj for each trip for this route
        let mut raptor_trips_for_this_route: Vec<RaptorTrip> = Vec::new();
        for trip_id in trip_ids_this_route {
            let times_for_trip = match stop_times_by_trip_id.get(trip_id) {
                Some(trip) => trip,
                None => {
                    let error_msg = format!(
                        "trip_id '{:?}' in trips.txt has no stop times in stop_times.txt",
                        trip_id
                    );
                    return Err(RaptorError::InvalidGtfs(error_msg));
                }
            };

            // order stop times for this trip by arrival times
            let mut ordered_times_for_trip: Vec<RaptorStopTime> = vec![
                RaptorStopTime {
                    arrival: 0,
                    departure: 0
                };
                route_stops.len()
            ];

            // simple sorting by iterating over position
            for (_position, stop_id, arrival_time, departure_time) in times_for_trip {
                match stop_positions.get(stop_id) {
                    Some(position) => {
                        ordered_times_for_trip[*position] = RaptorStopTime {
                            arrival: *arrival_time,
                            departure: *departure_time,
                        };
                    }

                    // erroring on no stop times
                    // BUG: Assumes that arrival/departure times are required even though only
                    // conditionally required
                    None => {
                        let error_msg = format!(
                            "stop '{:?}' in trip '{:?}' stop_times not found in route stop sequence",
                            stop_id, trip_id
                        );
                        return Err(RaptorError::InvalidGtfs(error_msg));
                    }
                }
            }

            raptor_trips_for_this_route.push(RaptorTrip {
                trip_id: *trip_id,
                stop_times: ordered_times_for_trip,
            });
        }

        // if no trips defined for this route, then can skip to next route
        if raptor_trips_for_this_route.is_empty() {
            continue;
        }

        // now need to sort trips by departure time for each route
        // NOTE: NEED TO BE SORTED because my raptor implemenation assumes you can iterate over
        // vector to find next trip for that route
        raptor_trips_for_this_route.sort_by_key(|trip| trip.stop_times[0].departure);

        // adding this route to routes_serving_stops
        let route_id = RaptorRouteID { id: route_idx };
        for (position, stop_id) in route_stops.iter().enumerate() {
            let route_serving_this_stop = RaptorRouteServingStop {
                route_id,
                stop_sequence: position,
            };
            routes_serving_stops[stop_id.id].push(route_serving_this_stop);
        }

        // add this route to routes vec
        let this_route = RaptorRoute {
            route_id,
            stops: route_stops,
            trips: raptor_trips_for_this_route,
        };
        routes.push(this_route);
        route_idx += 1;
    }
    Ok((routes, routes_serving_stops))
}

// NOTE: change so that can accept choice between naive and OSM based
pub fn build_timetable(feed: &RaptorGtfsFeed, id_map: &IdMap) -> Result<RaptorTimetable, RaptorError> {
    let stops = build_stops(feed, id_map)?;
    let (routes, routes_serving_stops) = build_routes(feed, id_map)?;
    let transfers = calculate_naive_transfers(feed, id_map)?;

    Ok(RaptorTimetable {
        stops,
        routes,
        transfers,
        routes_serving_stops,
    })
}
