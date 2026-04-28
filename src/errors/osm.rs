use thiserror::Error;

// SECTION: OSM ERRORS

#[derive(Debug, Error)]
pub enum OSMError {
    #[error("No file was found at the path you provided: {0}")]
    FileNotFound(String),

    #[error("The file you inputted was not an OSM PBF file! {0}")]
    NotAPbfFile(String),

    #[error("The path you gave is a directory, not a file! {0}")]
    NotAFile(String),

    #[error("Unknown error with path {0}")]
    UnknownError(String),
}
