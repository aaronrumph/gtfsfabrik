use ::zip::ZipArchive;
use std::ffi::OsStr;
use std::fs::DirEntry;
use std::fs::File;
use std::path::Path;
use std::path::PathBuf;

use crate::utils::errors::GtfsError;

#[derive(Debug)]
pub enum GtfsInputType {
    ZipFile(PathBuf),
    UnzippedFolder(PathBuf),
    MultipleZips(PathBuf),
    MultipleFolders(PathBuf),
}

// TODO: refactor with a is_a_zipfile function

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

        let contents: Vec<DirEntry> = std::fs::read_dir(input_path)?.filter_map(|e| e.ok()).collect();

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
                if listing_path.is_file() && let Ok(file) = File::open(&listing_path) {
                        return ZipArchive::new(file).is_ok();
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
                    && (listing.path().join("stops.txt").exists() || listing.path().join("agency.txt").exists())
            });

            if has_gtfs_subfolders {
                return Ok(GtfsInputType::MultipleFolders(input_path.to_path_buf()));
            }
        }
    }

    // TODO: if not a gtfs folder/file give error that says so

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

pub fn format_missing_gtfs_files(files: &[RequiredGtfsFile]) -> String {
    files.iter().map(|f| f.to_string()).collect::<Vec<_>>().join(", ")
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

/// Check whether the inputted unzipped or zipped GTFS folder contains the minimum necessary files
pub fn has_required_gtfs_files(gtfs_zip_or_dir: &Path) -> Result<(), GtfsError> {
    // TODO: Add checks for correct columns in all files with functions for each (agency.txt,
    // etc.)

    // NOTE: goal is to do this without unzipping files because 1. potentially expensive and slow
    // and 2. hard to claw back any changes if unexpected panic and don't want to accidentally
    // create massive temp artifacts

    // case where is (hopefully) zipfile
    if gtfs_zip_or_dir.is_file() {
        let zip_file = File::open(gtfs_zip_or_dir)?;

        // NOTE: Basing this off by_index example from zip crate docs
        let mut archive =
            ZipArchive::new(zip_file).map_err(|_| GtfsError::NotAZip(gtfs_zip_or_dir.to_string_lossy().to_string()))?;

        let mut zip_filenames: Vec<String> = Vec::new();
        for i in 0..archive.len() {
            if let Ok(file) = archive.by_index(i) {
                zip_filenames.push(file.name().to_string());
            }
        }
        let missing_files: Vec<RequiredGtfsFile> = REQUIRED_GTFS_FILES
            .iter()
            .filter(|f| !zip_filenames.contains(&f.to_string()))
            .cloned()
            .collect();

        if missing_files.is_empty() {
            Ok(())
        } else {
            Err(GtfsError::InvalidGTFS(
                gtfs_zip_or_dir.to_string_lossy().to_string(),
                missing_files,
            ))
        }
    } else {
        // case where is normal folder
        let missing_files: Vec<RequiredGtfsFile> = REQUIRED_GTFS_FILES
            .iter()
            .filter(|file| !gtfs_zip_or_dir.join(file.to_string()).exists())
            .cloned()
            .collect();

        if missing_files.is_empty() {
            Ok(())
        } else {
            let readable_path = gtfs_zip_or_dir.to_string_lossy().to_string();
            let error_to_give = GtfsError::InvalidGTFS(readable_path, missing_files);
            Err(error_to_give)
        }
    }
}

/// Check that inputed gtfs argument is valid
pub fn validate_gtfs(gtfs_args: &Vec<String>) -> Result<Vec<GtfsInputType>, GtfsError> {
    let mut gtfs_types: Vec<GtfsInputType> = Vec::new();
    for path in gtfs_args {
        let input_type = det_gtfs_input_type(path)?;
        match &input_type {
            GtfsInputType::ZipFile(_path) | GtfsInputType::UnzippedFolder(_path) => {
                has_required_gtfs_files(_path)?;
            }
            GtfsInputType::MultipleZips(_path) | GtfsInputType::MultipleFolders(_path) => {
                for listing in std::fs::read_dir(_path)?.filter_map(|e| e.ok()) {
                    has_required_gtfs_files(&listing.path())?;
                }
            }
        }
        gtfs_types.push(input_type)
    }
    Ok(gtfs_types)
}

// ALL POSSIBLE GTFS FILES
#[derive(Debug, Clone)]
pub enum GtfsFiles {
    // required
    Agency,
    Stops, // only required if fixed route service! ADA/demand response is a whole can of worms
    Routes,
    Trips,
    StopTimes,
    Calendar,

    // optional / conditionally required
    CalendarDates,
    FareAtrributes,
    FareRules,
    TimeFrames,
    RiderCategories,
    FareMedia,
    FareProducts,
    FareLegRules,
    FareLegJoinRules,
    FareTransferRules,
    Areas,
    StopAreas,
    Networks,
    RouteNetworks,
    Shapes,
    Frequencies,
    Transfers,
    Pathways,
    Levels,
    LocationGroups,
    LocationGroupStops,
    Locations,
    BookingRules,
    Translations,
    FeedInfo,
    Attributions,
}

impl std::fmt::Display for GtfsFiles {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let filename = match self {
            GtfsFiles::Agency => "agency.txt",
            GtfsFiles::Stops => "stops.txt",
            GtfsFiles::Routes => "routes.txt",
            GtfsFiles::Trips => "trips.txt",
            GtfsFiles::StopTimes => "stop_times.txt",
            GtfsFiles::Calendar => "calendar.txt",
            GtfsFiles::CalendarDates => "calendar_dates.txt",
            GtfsFiles::FareAtrributes => "fare_atrributes.txt",
            GtfsFiles::FareRules => "fare_rules.txt",
            GtfsFiles::TimeFrames => "timeframes.txt",
            GtfsFiles::RiderCategories => "rider_categories.txt",
            GtfsFiles::FareMedia => "fare_media.txt",
            GtfsFiles::FareProducts => "fare_products.txt",
            GtfsFiles::FareLegRules => "fare_leg_rules.txt",
            GtfsFiles::FareLegJoinRules => "fare_leg_join_rules.txt",
            GtfsFiles::FareTransferRules => "fare_transfer_rules.txt",
            GtfsFiles::Areas => "areas.txt",
            GtfsFiles::StopAreas => "stop_areas.txt",
            GtfsFiles::Networks => "networks.txt",
            GtfsFiles::RouteNetworks => "route_networks.txt",
            GtfsFiles::Shapes => "shapes.txt",
            GtfsFiles::Frequencies => "frequencies.txt",
            GtfsFiles::Transfers => "transfers.txt",
            GtfsFiles::Pathways => "pathways.txt",
            GtfsFiles::Levels => "levels.txt",
            GtfsFiles::LocationGroups => "location_groups.txt",
            GtfsFiles::LocationGroupStops => "location_group_stops.txt",
            GtfsFiles::Locations => "locations.geojson",
            GtfsFiles::BookingRules => "booking_rules.txt",
            GtfsFiles::Translations => "translations.txt",
            GtfsFiles::FeedInfo => "feed_info.txt",
            GtfsFiles::Attributions => "attributions.txt",
        };
        write!(f, "{}", filename)
    }
}

// Required headers/columns for each csv !!

// per gtfs spec, some files' precense is required/optional/conditionally required
// NOTE: Conditionally Required checks? Not currently doing because labor, but might be worth it?
#[derive(Debug, Clone, PartialEq)]
pub enum ColumnPresence {
    Required,
    ConditionallyRequired,
    ConditionallyForbidden,
    Optional,
}

// FIRST MACRO!  using so that don't have to manually write both enum and display impl for all gtfs
// files

macro_rules! gtfs_columns {
    ($file_name:ident { $($column:ident => $col_in_file_name:literal, $required:ident),* $(,)? }) => {
        #[derive(Debug, Clone)]
        pub enum $file_name {
            $($column),*
        }

        impl std::fmt::Display for $file_name {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                let name = match self {
                    $($file_name::$column => $col_in_file_name),*
                };
                write!(f, "{}", name)
            }
        }

        impl $file_name {
            pub fn presence(&self) -> ColumnPresence {
                match self {
                    $($file_name::$column => ColumnPresence::$required),*
                }
            }

            pub fn get_required() -> Vec<Self> {
                vec![$($file_name::$column),*].into_iter().filter(|c| c.presence() == ColumnPresence::Required).collect()
            }

            pub fn get_conditionally_required() -> Vec<Self> {
                vec![$($file_name::$column),*].into_iter().filter(|c| c.presence() == ColumnPresence::ConditionallyRequired).collect()
            }

            pub fn get_conditionally_forbidden() -> Vec<Self> {
                vec![$($file_name::$column),*].into_iter().filter(|c| c.presence() == ColumnPresence::ConditionallyForbidden).collect()
            }
            pub fn get_optional() -> Vec<Self> {
                vec![$($file_name::$column),*].into_iter().filter(|c| c.presence() == ColumnPresence::Optional).collect()
            }
        }
    };
}

// The following gtfs_columns! macro invocations make enums that already have 1. Display (for
// actual plain text names) and 2. presence information already implemented

gtfs_columns!(AgencyColumns {
    ID => "agency_id", ConditionallyRequired,
    Name => "agency_name", Required,
    URL => "agency_url", Required,
    Timezone => "agency_timezone", Required,
    Lang => "agency_lang", Optional,
    Phone => "agency_phone", Optional,
    FareUrl => "agency_fare_url", Optional,
    Email => "agency_email", Optional,
    CEMVSupport => "cemv_support", Optional,
});

gtfs_columns!(StopColumns {
    StopID => "stop_id", Required,
    Code => "stop_code", Optional,
    Name => "stop_name", ConditionallyRequired,
    TTSName => "tts_stop_name", Optional,
    Description => "stop_desc", Optional,
    Latitude => "stop_lat", ConditionallyRequired,
    Longitude => "stop_lon", ConditionallyRequired,
    ZoneID => "zone_id", Optional,
    URL => "stop_url", Optional,
    LocationType => "location_type", Optional,
    ParentStation => "parent_station", ConditionallyRequired,
    Timezone => "stop_timezone", Optional,
    WheelchairBoarding => "wheelchair_boarding", Optional,
    LevelID => "level_id", Optional,
    PlatformCode => "platform_code", Optional,
    Access => "stop_access", ConditionallyForbidden,
});

gtfs_columns!(RouteColumns {
    RouteID => "route_id", Required,
    AgencyID => "agency_id", ConditionallyRequired,
    ShortName => "route_short_name", ConditionallyRequired,
    LongName => "route_long_name", ConditionallyRequired,
    Description => "route_desc", Optional,
    Type => "route_type", Required,
    URL => "route_url", Optional,
    Color => "route_color", Optional,
    TextColor => "route_text_color", Optional,
    SortOrder => "route_sort_order", Optional,
    ContinuousPickup => "continuous_pickup", ConditionallyForbidden,
    ContinuousDropOff => "continuous_drop_off", ConditionallyForbidden,
    NetworkID => "network_id", ConditionallyForbidden,
    CEMVSupport => "cemv_support", Optional,
});

gtfs_columns!(TripColumns {
    RouteID => "route_id", Required,
    ServiceID => "service_id", Required,
    TripID => "trip_id", Required,
    HeadSign => "trip_headsign", Optional,
    ShortName => "trip_short_name", Optional,
    DirectionID => "direction_id", Optional,
    BlockID => "block_id", Optional,
    ShapeID => "shape_id", ConditionallyRequired,
    WheelchairAccessible => "wheelchair_accessible", Optional,
    BikesAllowed => "bikes_allowed", Optional,
    CarsAllowed => "cars_allowed", Optional,
});

gtfs_columns!(StopTimesColumns {
    TripID => "trip_id", Required,
    ArrivalTime => "arrival_time", ConditionallyRequired,
    DepartureTime => "departure_time", ConditionallyRequired,
    StopID => "stop_id", ConditionallyRequired,
    LocationGroupID => "location_group_id", ConditionallyForbidden,
    LocationID => "location_id", ConditionallyForbidden,
    StopSequence => "stop_sequence", Required,
    StopHeadsign => "stop_headsign", Optional,
    StartPickupDropOffWindow => "start_pickup_drop_off_window", ConditionallyRequired,
    EndPickupDropOffWindow => "end_pickup_drop_off_window", ConditionallyRequired,
    PickupType => "pickup_type", ConditionallyForbidden,
    DropOffType => "drop_off_type", ConditionallyForbidden,
    ContinuousPickup => "continuous_pickup", ConditionallyForbidden,
    ContinuousDropOff => "continuous_drop_off", ConditionallyForbidden,
    ShapeDistTraveled => "shape_dist_traveled", Optional,
    Timepoint => "timepoint", Optional,
    PickupBookingRuleId => "pickup_booking_rule_id", Optional,
    DropOffBookingRuleId => "drop_off_booking_rule_id", Optional,
});

gtfs_columns!(CalendarTimes {
    ServiceID => "service_id", Required,
    Monday => "monday", Required,
    Tuesday => "tuesday", Required,
    Wednesday => "wednesday", Required,
    Thursday => "thursday", Required,
    Friday => "friday", Required,
    Saturday => "saturday", Required,
    Sunday => "sunday", Required,
    StartDate => "start_date", Required,
    EndDate => "end_date", Required,
});

gtfs_columns!(CalendarDates {
    ServiceID => "service_id", Required,
    Date => "date", Required,
    ExceptionType => "exception_type", Required,
});

// TODO: add column checks for required files
// TODO: Add columns for all files?

// macro to read in gtfs file to Dataframe with read_gtfs!(GtfsFiles::...) !!!!
#[macro_export]
macro_rules! read_gtfs {
    ($path:expr, GtfsFiles::$variant:ident) => {{
        use polars::prelude::*;
        let file = $crate::utils::files::gtfs::GtfsFiles::$variant;
        let path = std::path::Path::new($path).join(file.to_string());
        CsvReadOptions::default()
            .with_has_header(true)
            .with_infer_schema_length(Some(0)) // need to read all cols as strings to avoid errors
            .try_into_reader_with_file_path(Some(path))?
            .finish()?
    }};
}
