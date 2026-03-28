// This module loads in GTFS data to use with RAPTOR
use crate::algorithms::raptor::utils::{
    RaptorRoute, RaptorRouteID, RaptorStop, RaptorStopID, RaptorTimetable, RaptorTrip, RaptorTripID,
};
use crate::read_gtfs;
use crate::utils::errors::RaptorError;
use crate::utils::files::gtfs::{GtfsFiles, RouteColumns, StopColumns, TripColumns};

use polars::prelude::*;
use std::collections::HashMap;

pub struct RaptorGtfsFeed {
    pub routes: DataFrame,
    pub trips: DataFrame,
    pub stops: DataFrame,
    pub stop_times: DataFrame,
}

pub struct IdMap {
    pub routes: HashMap<String, RaptorRouteID>,
    pub trips: HashMap<String, RaptorTripID>,
    pub stops: HashMap<String, RaptorStopID>,
}

pub struct ReverseIdMap {
    pub routes: HashMap<RaptorRouteID, String>,
    pub trips: HashMap<RaptorTripID, String>,
    pub stops: HashMap<RaptorStopID, String>,
}

// NOTE: because the ids have to be unique as per the GTFS spec, the id map is invertible, so
// providing easy way to convert between inverted and normal
pub trait Invertible {
    type Inverted;
    fn invert(&self) -> Self::Inverted;
}

impl Invertible for IdMap {
    type Inverted = ReverseIdMap;
    fn invert(&self) -> ReverseIdMap {
        ReverseIdMap {
            stops: self.stops.iter().map(|(key, value)| (*value, key.clone())).collect(),
            routes: self.routes.iter().map(|(key, value)| (*value, key.clone())).collect(),
            trips: self.trips.iter().map(|(key, value)| (*value, key.clone())).collect(),
        }
    }
}

impl Invertible for ReverseIdMap {
    type Inverted = IdMap;
    fn invert(&self) -> IdMap {
        IdMap {
            stops: self.stops.iter().map(|(key, value)| (value.clone(), *key)).collect(),
            routes: self.routes.iter().map(|(key, value)| (value.clone(), *key)).collect(),
            trips: self.trips.iter().map(|(key, value)| (value.clone(), *key)).collect(),
        }
    }
}

/// Function to read in routes, stops, trips and stop times from a given GTFS feed into dataframe
pub fn load_gtfs(feed_dir: &str) -> Result<RaptorGtfsFeed, PolarsError> {
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
    let route_gtfs_ids = feed.stops.column(&RouteColumns::RouteID.to_string())?.str()?;
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
