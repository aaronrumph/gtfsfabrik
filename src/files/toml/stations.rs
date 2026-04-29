use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FabrikStationToml {
    // snappy name of station
    pub name: String,

    // TODO: Make fit with GTFS!!
    #[serde(default)]
    pub gtfs_stop_id: Vec<String>,

    #[serde(default)]
    pub description: Option<String>,

    #[serde(default)]
    pub lat: Option<f64>,

    #[serde(default)]
    pub lon: Option<f64>,
}
