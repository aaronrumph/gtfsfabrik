use crate::{
    algorithms::raptor::types::RaptorStopID, errors::parsing::TimeParsingError, gtfs::datetime::Seconds,
    gtfs::datetime::seconds_to_gtfs_time,
};

use thiserror::Error;

// SECTION: RAPTOR ERRORS

#[derive(Debug, Error)]
pub enum RaptorError {
    #[error("Destination stop is not reachable from origin station")]
    DestinationUnreachable {
        origin: RaptorStopID,
        destination: RaptorStopID,
        departure_time: Seconds,
    },

    // TODO: Replace earliest_departure time with non-seconds (actual time)
    #[error(
        "No trips found for route {} after {}",
        route_id,
        seconds_to_gtfs_time(*earliest_departure)
    )]
    NoTrips {
        route_id: String,
        earliest_departure: Seconds,
    },

    #[error("Journey has no legs")]
    EmptyJourney,

    #[error("Route {route_id} not found in timetable")]
    RouteNotFound { route_id: String },

    // TODO: same as above
    #[error("Departure time {0} is invalid")]
    InvalidDepartureTime(Seconds),

    #[error("Polars error: {0}")]
    PolarsError(#[from] polars::prelude::PolarsError),

    #[error("Missing stop at index {0} during timetable construction")]
    MissingStop(usize),

    #[error("Invalid GTFS data: {0}")]
    InvalidGtfs(String),

    #[error("Time parsing error in stop_times.txt at row {row}: \n {source}")]
    StopTimeParsingError {
        row: usize,
        #[source]
        source: TimeParsingError,
    },

    #[error("Failed to parse stop location in stops.txt file at row '{row}' \n {source}")]
    StopLocationParsingError {
        lat_or_lon: String,
        row: usize,
        #[source]
        source: std::num::ParseFloatError,
    },

    #[error("Cache error when trying to build raptor: {0}")]
    CacheError(String),

    #[error("Some unknown error occured {0}")]
    UnknownError(String),
}
