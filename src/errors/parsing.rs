use thiserror::Error;

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

// SECTION: TimeParsingError
#[derive(Debug, thiserror::Error)]
pub enum TimeParsingError {
    #[error("invalid time format '{0}', expected HH:mm:ss")]
    InvalidFormat(String),

    #[error("invalid {0} component '{1}'")]
    InvalidComponent(String, String),
}
