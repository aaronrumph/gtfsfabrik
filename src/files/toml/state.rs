// Toml file to track the current state of the fabrik
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FabrikStateToml {
    pub current_scenario: String,

    // NOTE: for now, just basing it off how git does it, but think I need to come up with a
    // better solution
    #[serde(default)]
    pub current_commit: Option<String>,
    #[serde(default)]
    pub head_ref: Option<String>,
}
