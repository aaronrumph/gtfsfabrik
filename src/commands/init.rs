use ::std::path::{Path, PathBuf};
use std::fs;
use std::io::{self, Write};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum InitError {
    #[error("IO error: {0}")]
    IOError(#[from] io::Error),

    #[error("Geographic scope given was invalid, must be ('place', 'county', 'msa', or 'csa')")]
    InvalidGeoScope,

    #[error("There is already something at the desired path!")]
    PathNotEmpty,
}

pub struct InitOptions {
    pub path: String,
    pub gtfs: Option<String>,
    pub osm: Option<String>,
    pub place: Option<String>,
    pub geoscope: Option<String>,
    pub ridership: Option<String>,
    pub usegit: bool,
}

pub fn init_fabrik(init_options: InitOptions) -> Result<(), InitError> {
    // input validation to make sure flags given can be used
    if let Some(gs) = &init_options.geoscope {
        match gs.as_str() {
            "place" | "county" | "msa" | "csa" => {}
            _ => return Err(InitError::InvalidGeoScope),
        }
    }

    // create the fabrik
    let fabrik_base_path = PathBuf::from(&init_options.path);
    if !fabrik_base_path.exists() {
        match fs::create_dir(&fabrik_base_path) {
            Ok(_) => {}
            Err(e) if e.kind() == io::ErrorKind::NotFound => {
                // at least some directories along the way are missing
                println!("The given path {0} is missing intermediate directories along the way. Would you like to create them? [y/n]: ", init_options.path);

                let mut inputted_decision = String::new();
                io::stdin().read_line(&mut inputted_decision)?;

                if inputted_decision.trim().to_lowercase() == "y" {
                    fs::create_dir_all(&fabrik_base_path)?;
                } else {
                    return Ok(());
                }
            }
            Err(e) => return Err(e.into()),
        }
    }

    // setting up log file locations
    let logs_dir = fabrik_base_path.join(".logs");
    fs::create_dir(&logs_dir);

    let log_files_needed = ["info", "debug", "warnings", "errors", "command_history"];
    for file in &log_files_needed {
        // don't need to check whether file exists before creating it because will
        // by definition be in a new dir
        let file_name = logs_dir.join(format!("{}.log", file));
        fs::File::create(file_name);
    }

    let hidden_fabrik_dir = fabrik_base_path.join(".fabrik");
    fs::create_dir(&hidden_fabrik_dir);

    let fabrik_subdirectories = ["data", "elements", "products", "analysis", "scenarios"];
    for subdir in fabrik_subdirectories {
        fs::create_dir(hidden_fabrik_dir.join(subdir))?;
    }

    // TODO: Add templates for .state.toml and fabrik.toml files and create them with init
    // TODO: set up git proper .gitignore

    // NOTE: Here for now so that compiler/linter doesn't get mad about not returning result

    Ok(())
}
