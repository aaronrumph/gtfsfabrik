use std::io;
use thiserror::Error;

use crate::errors::{gtfs::GtfsError, osm::OSMError};

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
    OSMError(#[from] OSMError),

    #[error("No path provided!")]
    NoPathProvided,
}

// SECTION: InteractiveError

#[derive(Debug, Error)]
pub enum InteractiveError {
    #[error("Readline error: {0}")]
    ReadlineError(#[from] rustyline::error::ReadlineError),

    #[error("Failed to parse command line: {0}")]
    ParseError(#[from] shell_words::ParseError),
}

// SECTION: FabrikCommandError

#[derive(Debug, Error)]
pub enum FabrikCommandError {
    // top level error handling!
    #[error(transparent)]
    InitError(#[from] InitError),

    #[error(transparent)]
    InteractiveError(#[from] InteractiveError),

    #[error("That command has not been implemented yet, sorry!")]
    CommandNotImplemented,
}
