// error types for fabrik specific tasks

use thiserror::Error;

#[derive(Debug, Error)]
pub enum FabrikLookupError {
    #[error("Could not find scenario '{0}'")]
    ScenarioNotFound(String),

    #[error("Could not find agency '{0}'")]
    AgencyNotFound(String),

    #[error("Couldn't read file")]
    Io(#[from] std::io::Error),

    #[error("Couldn't parse toml")]
    Toml(#[from] toml::de::Error),
}
