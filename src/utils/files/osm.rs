// internal modules
use crate::utils::errors::OSMErorr;

// external
use std::ffi::OsStr;
use std::path::Path;

// NOTE: Write PBF verification function using header???
pub fn validate_osm(osm_arg: &str) -> Result<(), OSMErorr> {
    // doing lazy check for extension becaused 1. no good crate that will check for headers etc and
    // 2. it's good enough for Osmium
    let input_path = osm_arg.to_string();
    let file_path = Path::new(osm_arg);
    if !file_path.exists() {
        Err(OSMErorr::FileNotFound(input_path))
    } else if file_path.is_dir() {
        Err(OSMErorr::NotAFile(input_path))
    } else if let Some(file_ext) = file_path.extension().and_then(OsStr::to_str) {
        if file_ext == "pbf" {
            Ok(())
        } else {
            Err(OSMErorr::NotAPbfFile(input_path))
        }
    } else {
        Err(OSMErorr::UnknownError(input_path))
    }
}
