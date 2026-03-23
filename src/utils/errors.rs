// Module containing all the errors that can arise

use crate::utils::files::format_missing_gtfs_files;
use crate::utils::files::RequiredGtfsFile;

use std::io;
use thiserror::Error;

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

#[derive(Debug, Error)]
pub enum GeocodingError {
    #[error("No input provided'{0}'")]
    NoInput(String),

    #[error("Geocoding failed. Could be because the place you specified is misspelled, is not usable, or is too vague: {0}")]
    GeocodingFailed(String),
}

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
            GtfsError::NotAZip(path) => write!(
                f,
                "The path you provided was a file, but not a zip file!: '{}'",
                path
            ),
            GtfsError::InvalidGTFS(path, missing) => write!(
                f,
                "The GTFS feed at '{}' is missing the following REQUIRED files: {}",
                path,
                format_missing_gtfs_files(missing)
            ),
            GtfsError::IoError(path) => write!(f, "An IO error occured with '{}' Maybe try checking file permissions or submitting a bug report", path),
            GtfsError::Other(path) => write!(f, "The provided path is not a GTFS folder or zipfile!: '{}' . Submit a bug report if you believe this is incorrect", path)
        }
    }
}

// so that can handle io errors from std::fs::...
impl From<std::io::Error> for GtfsError {
    fn from(e: std::io::Error) -> Self {
        GtfsError::IoError(e.to_string())
    }
}

// OSM SECTION ---------

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
