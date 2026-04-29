use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FabrikAgencyToml {
    pub name: String,
    pub gtfs_agency_id: String,

    // TODO: check actually fits agency.txt
    #[serde(default)]
    pub display_name: Option<String>,

    #[serde(default)]
    pub url: Option<String>,

    #[serde(default)]
    pub timezone: Option<String>,
}
