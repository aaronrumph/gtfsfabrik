// This module has structs for the fabrik.toml file]
use crate::utils::geotypes::GeoScope;
use crate::utils::geotypes::Place;
use serde::Serialize;

#[derive(Debug, Serialize)]
struct FabrikToml {
    title: String,
    fabrik_info: GtfsFabrikInfo,
    config: FabrikConfig,
}

#[derive(Serialize, Debug)]
struct GtfsFabrikInfo {
    version: String,
}

#[derive(Debug, Serialize)]
struct FabrikConfig {
    pub place: Place,
    pub geoscope: GeoScope,
    pub is_multiagency: bool,
    pub use_git: bool,
    pub has_gtfs: bool,
    pub has_osm: bool,
}
