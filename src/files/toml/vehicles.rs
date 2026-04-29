use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FabrikVehicleToml {
    pub name: String,

    // TODO: model transit modes with types
    #[serde(default)]
    pub mode: Option<String>,

    // TODO: model capacity with types (sitting, standing etc)
    #[serde(default)]
    pub capacity: Option<u32>,

    // for recalculating travel times
    #[serde(default)]
    pub avg_acceleration_rate: Option<f64>,
    #[serde(default)]
    pub avg_deceleration_rate: Option<f64>,

    #[serde(default)]
    pub length: Option<f64>,

    #[serde(default)]
    pub description: Option<String>,
}
