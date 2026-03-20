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
            GtfsError::IoError(path) => write!(f, "Some unknown/unexpected IO error occured with '{}' Maybe try checking file permissions or submitting a bug report", path),
            GtfsError::Other(path) => write!(f, "Some unknown/unexpected error occured using the provided path: '{}' . Feel free to submit a bug report", path)
        }
    }
}

// so that can handle io errors from std::fs::...
impl From<std::io::Error> for GtfsError {
    fn from(e: std::io::Error) -> Self {
        GtfsError::IoError(e.to_string())
    }
}
