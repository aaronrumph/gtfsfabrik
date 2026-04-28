use crate::files::gtfs::{RequiredGtfsFile, format_missing_gtfs_files};

use thiserror::Error;

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
