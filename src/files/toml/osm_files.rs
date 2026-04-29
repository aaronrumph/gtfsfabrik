use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FabrikOsmFileToml {
    // the name for the osm file TODO: Come up with default
    pub name: String,
    pub path: PathBuf,

    #[serde(default)]
    pub description: Option<String>,
}
