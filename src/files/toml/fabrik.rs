// This module has structs for the fabrik.toml file]
use crate::types::geotypes::{GeoScope, Place};
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct FabrikToml {
    // information about the version of gtfsfabrik for compatability
    pub fabrik_info: GtfsFabrikInfo,
    // the name of the fabrik (defaults to name of dir)
    pub fabrik_title: String,
    // config info like whether to use git, etc.
    pub config: FabrikConfig,

    // All of these are optional and are the NAMES of the things they refer to for look up
    // in their respective toml files
    #[serde(default)]
    pub scenarios: Vec<String>,

    #[serde(default)]
    pub lines: Vec<String>,

    #[serde(default)]
    pub stations: Vec<String>,

    #[serde(default)]
    pub agencies: Vec<String>,

    #[serde(default)]
    pub vehicles: Vec<String>,

    #[serde(default)]
    pub gtfs_dirs: Vec<String>,

    #[serde(default)]
    pub ridership_files: Vec<String>,

    #[serde(default)]
    pub osm_files: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct GtfsFabrikInfo {
    version: String,
    file_schema: u32,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct FabrikConfig {
    pub place: Place,
    pub geoscope: GeoScope,
    pub is_multiagency: bool,
    pub use_git: bool,
    pub has_gtfs: bool,
    pub has_osm: bool,
}
