use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FabrikGtfsDirToml {
    // short name to be referenced elsewhere
    pub name: String,
    pub path: PathBuf,

    // foreign key: agency name
    #[serde(default)]
    pub agency: Option<String>,

    #[serde(default)]
    pub description: Option<String>,
}
