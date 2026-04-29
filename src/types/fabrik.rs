use crate::{
    errors::fabrik::FabrikLookupError,
    files::toml::{agencies::FabrikAgencyToml, fabrik::FabrikToml, scenarios::FabrikScenarioToml},
};
use std::path::PathBuf;

/// The main representation of the Fabrik
pub struct Fabrik {
    // the root path of the fabrik
    pub root: PathBuf,
    // fabrik info read in from the fabrik.toml file
    pub manifest: FabrikToml,
}

impl Fabrik {
    // look up scenario in .fabrik/scenarios
    pub fn read_scenario(&self, scenario_name: &str) -> Result<FabrikScenarioToml, FabrikLookupError> {
        // TODO: Make "contains" methdo for FabrikToml
        if !self.manifest.scenarios.iter().any(|s| s == scenario_name) {
            return Err(FabrikLookupError::ScenarioNotFound(scenario_name.to_string()));
        }

        let scenario_file_name = format!("{scenario_name}.toml");
        let scenario_path = self.root.join(".fabrik").join("scenarios").join(scenario_file_name);

        // read in file, can error with io::error
        let file_text = std::fs::read_to_string(&scenario_path)?;
        let scenario: FabrikScenarioToml = toml::from_str(&file_text)?;

        Ok(scenario)
    }

    // look up agency in .fabrik/scenarios
    pub fn read_agency(&self, agency_name: &str) -> Result<FabrikAgencyToml, FabrikLookupError> {
        // TODO: Make "contains" methdo for FabrikToml
        if !self.manifest.scenarios.iter().any(|s| s == agency_name) {
            return Err(FabrikLookupError::AgencyNotFound(agency_name.to_string()));
        }

        let agency_file_name = format!("{agency_name}.toml");
        let agency_path = self.root.join(".fabrik").join("scenarios").join(agency_file_name);

        // read in file, can error with io::error
        let file_text = std::fs::read_to_string(&agency_path)?;
        let agency: FabrikAgencyToml = toml::from_str(&file_text)?;

        Ok(agency)
    }
}
