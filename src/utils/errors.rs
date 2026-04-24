// Module containing all the errors that can arise
use crate::algorithms::raptor::types::RaptorStopID;
use crate::gtfs::datetime::Seconds;
use crate::utils::files::gtfs::RequiredGtfsFile;
use crate::utils::files::gtfs::format_missing_gtfs_files;

use std::io;
use thiserror::Error;

// TODO: Split this module into sane submodules

// SECTION: INIT ERRORS
#[derive(Debug, Error)]
pub enum InitError {
    #[error("IO error: {0}")]
    IOError(#[from] io::Error),

    #[error("Geographic scope given was invalid, must be ('place', 'county', 'msa', or 'csa')")]
    InvalidGeoScope,

    #[error("There is already something at the desired path!")]
    PathNotEmpty,

    #[error("GTFS error: {0}")]
    GTFSError(#[from] GtfsError),

    #[error("OSM Error!: {0}")]
    OSMError(#[from] OSMErorr),
}

// SECTION: GEOCODING ERRORS
#[derive(Debug, Error)]
pub enum GeocodingError {
    #[error("No input provided'{0}'")]
    NoInput(String),

    #[error(
        "Geocoding failed. Could be because the place you specified is misspelled, is not usable, or is too vague: {0}"
    )]
    GeocodingFailed(String),
}

// SECTION: GTFS ERRORS
#[derive(Debug, Error)]
pub enum GtfsError {
    // NOTE: error message going in impl Display because InvalidGTFS requires string and
    // vec
    NotFound(String),
    NotAZip(String),
    InvalidGTFS(String, Vec<RequiredGtfsFile>),
    IoError(String),
    Other(String),
}

// for giving good error messages for bad gtfs because important!
impl std::fmt::Display for GtfsError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            GtfsError::NotFound(path) => write!(f, "Path not found for path: '{}'", path),
            GtfsError::NotAZip(path) => write!(f, "The path you provided was a file, but not a zip file!: '{}'", path),
            GtfsError::InvalidGTFS(path, missing) => write!(
                f,
                "The GTFS feed at '{}' is missing the following REQUIRED files: {}",
                path,
                format_missing_gtfs_files(missing)
            ),
            GtfsError::IoError(path) => write!(
                f,
                "An IO error occured with '{}' Maybe try checking file permissions or submitting a bug report",
                path
            ),
            GtfsError::Other(path) => write!(
                f,
                "The provided path is not a GTFS folder or zipfile!: '{}' . Submit a bug report if you believe this is incorrect",
                path
            ),
        }
    }
}

// so that can handle io errors from std::fs::...
impl From<std::io::Error> for GtfsError {
    fn from(e: std::io::Error) -> Self {
        GtfsError::IoError(e.to_string())
    }
}

#[derive(Debug, thiserror::Error)]
pub enum TimeParsingError {
    #[error("invalid time format '{0}', expected HH:mm:ss")]
    InvalidFormat(String),

    #[error("invalid {0} component '{1}'")]
    InvalidComponent(String, String),
}

// SECTION: OSM ERRORS
#[derive(Debug, Error)]
pub enum OSMErorr {
    #[error("No file was found at the path you provided: {0}")]
    FileNotFound(String),

    #[error("The file you inputted was not an OSM PBF file! {0}")]
    NotAPbfFile(String),

    #[error("The path you gave is a directory, not a file! {0}")]
    NotAFile(String),

    #[error("Unknown error with path {0}")]
    UnknownError(String),
}

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
    #[error("No trips found for route {route_id} after {earliest_departure}s")]
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
