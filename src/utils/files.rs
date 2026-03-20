use std::fs::DirEntry;
use std::fs::File;
use std::path::Path;
use std::path::PathBuf;
use zip::ZipArchive;

use crate::utils::errors::GtfsError;

#[derive(Debug)]
pub enum GtfsInputType {
    ZipFile(PathBuf),
    UnzippedFolder(PathBuf),
    MultipleZips(PathBuf),
    MultipleFolders(PathBuf),
}

// TODO: Clean up logic and readability of this function
pub fn det_gtfs_input_type(path: &str) -> Result<GtfsInputType, GtfsError> {
    let input_path = Path::new(path);

    // exactly four paths to check 1. path not found, 2. path is zip, 3. path is dir, path is
    // something else?? (can this even happen??)
    if !input_path.exists() {
        return Err(GtfsError::NotFound(path.to_string()));
    }

    // assume it's a zipfile
    if input_path.is_file() {
        let zip_file = File::open(input_path)?;

        // if zip::ZipArchive::new doesn't work, it should be because the file is not a
        // valid zip, so can use to test if is a zip or not
        return match ZipArchive::new(zip_file) {
            Ok(_) => Ok(GtfsInputType::ZipFile(input_path.to_path_buf())),
            Err(_) => Err(GtfsError::NotAZip(path.to_string())),
        };
    } else if input_path.is_dir() {
        // couple cases for is a dir: 1. single agency unzipped folder,
        // 2. multiagency folder containing unzipped folders,
        // 3. multiagency folder containing multiple zips,

        let contents: Vec<DirEntry> = std::fs::read_dir(input_path)?
            .filter_map(|e| e.ok())
            .collect();

        // TODO: find way to refine assumption below and at

        // assume that if it has stops.txt or agency.txt then it's a GTFS folder
        let is_single_agency_folder = contents
            .iter()
            .any(|e| e.file_name() == "stops.txt" || e.file_name() == "agency.txt");

        if is_single_agency_folder {
            return Ok(GtfsInputType::UnzippedFolder(input_path.to_path_buf()));
        } else {
            // does not have stops.txt or agency.txt at top level

            // check whether directorie has zips with same try to unzip method as before
            let has_zips = contents.iter().any(|listing| {
                let listing_path = listing.path();
                if listing_path.is_file() {
                    if let Ok(file) = File::open(&listing_path) {
                        return ZipArchive::new(file).is_ok();
                    }
                }
                false
            });

            if has_zips {
                return Ok(GtfsInputType::MultipleZips(input_path.to_path_buf()));
            }

            // otherwise check if dir has subdirectories that are gtfs folders with same assumption
            // as before
            let has_gtfs_subfolders = contents.iter().any(|listing| {
                listing.path().is_dir()
                    && (listing.path().join("stops.txt").exists()
                        || listing.path().join("agency.txt").exists())
            });

            if has_gtfs_subfolders {
                return Ok(GtfsInputType::MultipleFolders(input_path.to_path_buf()));
            }
        }
    }

    Err(GtfsError::Other(path.to_string()))
}

// CHECKING GTFS VALIDITY
// enums/display method for required files

#[derive(Debug, Clone)] // NOTE: IF CHANGE REMEMBER CHANGE CONST INSTANCE BELOW
pub enum RequiredGtfsFile {
    Agency,
    Stops,
    Routes,
    Trips,
    StopTimes,
    Calendar,
}

// so that can print which files missing in error message!
impl std::fmt::Display for RequiredGtfsFile {
    fn fmt(&self, format: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            RequiredGtfsFile::Agency => write!(format, "agency.txt"),
            RequiredGtfsFile::Stops => write!(format, "stops.txt"),
            RequiredGtfsFile::Routes => write!(format, "routes.txt"),
            RequiredGtfsFile::Trips => write!(format, "trips.txt"),
            RequiredGtfsFile::StopTimes => write!(format, "stop_times.txt"),
            RequiredGtfsFile::Calendar => write!(format, "calendar.txt"),
        }
    }
}

pub fn format_missing_gtfs_files(files: &Vec<RequiredGtfsFile>) -> String {
    files
        .iter()
        .map(|f| f.to_string())
        .collect::<Vec<_>>()
        .join(", ")
}

// using to check off which files which are required are present (cross-off)
const REQUIRED_GTFS_FILES: &[RequiredGtfsFile] = &[
    RequiredGtfsFile::Agency,
    RequiredGtfsFile::Stops,
    RequiredGtfsFile::Routes,
    RequiredGtfsFile::Trips,
    RequiredGtfsFile::StopTimes,
    RequiredGtfsFile::Calendar,
];

/// Check whether the inputted unzipped(!!!) GTFS folder contains the minimum necessary files
pub fn has_required_gtfs_files(unzipped_folder_path: &Path) -> Result<(), GtfsError> {
    let missing_files: Vec<RequiredGtfsFile> = REQUIRED_GTFS_FILES
        .iter()
        .filter(|file| !unzipped_folder_path.join(file.to_string()).exists())
        .cloned()
        .collect();

    match missing_files.is_empty() {
        true => Ok(()),
        false => {
            let readable_path = unzipped_folder_path.to_string_lossy().to_string();
            let error_to_give = GtfsError::InvalidGTFS(readable_path, missing_files);
            Err(error_to_give)
        }
    }
}
