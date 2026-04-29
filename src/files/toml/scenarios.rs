use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FabrikScenarioToml {
    pub name: String,

    #[serde(default)]
    pub description: Option<String>,

    #[serde(default)]
    pub agencies: Vec<String>,

    #[serde(default)]
    pub gtfs_dirs: Vec<PathBuf>,

    #[serde(default)]
    pub osm_files: Vec<PathBuf>,

    #[serde(default)]
    pub lines: Vec<String>,

    #[serde(default)]
    pub stations: Vec<String>,

    #[serde(default)]
    pub vehicles: Vec<String>,
}
