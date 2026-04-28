// This module allows for caching of a Raptor network so do not have slow loading times
// everytime a new query is called

use crate::errors::raptor::RaptorError;
use crate::files::gtfs::{RouteColumns, StopColumns, TripColumns};
use crate::{
    algorithms::raptor::types::{IdMap, RaptorGtfsFeed, RaptorTimetable},
    files::gtfs::StopTimesColumns,
};

use rkyv::{deserialize, rancor::Error};
use std::path::PathBuf;
use xxhash_rust::xxh64::Xxh64;

#[derive(Clone, Debug)]
pub struct RaptorCache {
    pub cache_dir: PathBuf,
}

impl RaptorCache {
    // default cache directory is $CACHE/gtfsfabrik/raptor
    fn default_cache_dir() -> Result<PathBuf, RaptorError> {
        match dirs::cache_dir() {
            Some(base) => Ok(base.join("gtfsfabrik").join("raptor")),
            None => {
                let error_msg = "Could not find your system's cache directory".to_string();
                Err(RaptorError::CacheError(error_msg))
            }
        }
    }

    // TODO: add docs
    pub fn new(cache_dir: PathBuf) -> Result<Self, RaptorError> {
        let valid_dir = Self::validate_dir(cache_dir)?;
        Ok(Self { cache_dir: valid_dir })
    }

    // TODO: add docs
    // TODO: change to impl default trait for RaptorCache
    pub fn default() -> Result<Self, RaptorError> {
        let dir = Self::default_cache_dir()?;
        Self::new(dir)
    }

    fn validate_dir(cache_dir: PathBuf) -> Result<PathBuf, RaptorError> {
        if !cache_dir.exists() {
            let error_msg = format!("No directory exists at cache path: '{}'", cache_dir.display());
            return Err(RaptorError::CacheError(error_msg));
        }
        Ok(cache_dir)
    }

    // produce a hash of the feed to use for cache validation based on GTFS data like agency name,
    // number of rows for files, and specific values from GTFS fiels. The resulting hash acts as
    // the filename!
    // very ugly but it works I think?
    fn hash_feed(&self, gtfs_feed: &RaptorGtfsFeed) -> Result<PathBuf, RaptorError> {
        // first need to build the input to hash
        let num_stops = gtfs_feed.stops.height();
        let num_stop_times = gtfs_feed.stop_times.height();
        let num_trips = gtfs_feed.trips.height();
        let num_routes = gtfs_feed.routes.height();

        // picking columns out of DFs
        let stop_ids = gtfs_feed.stops.column(&StopColumns::StopID.to_string())?.str()?;

        let stop_time_ids = gtfs_feed
            .stop_times
            .column(&StopTimesColumns::StopID.to_string())?
            .str()?;
        let trip_ids = gtfs_feed.trips.column(&TripColumns::TripID.to_string())?.str()?;
        let route_ids = gtfs_feed.routes.column(&RouteColumns::RouteID.to_string())?.str()?;

        // using the midway point
        let middle_stop = stop_ids.get(num_stops / 2).unwrap_or("");
        let middle_stop_time = stop_time_ids.get(num_stop_times / 2).unwrap_or("");
        let middle_trip = trip_ids.get(num_trips / 2).unwrap_or("");
        let middle_route = route_ids.get(num_routes / 2).unwrap_or("");

        let arrival_times = gtfs_feed
            .stop_times
            .column(&StopTimesColumns::ArrivalTime.to_string())?
            .str()?;
        let middle_arrival_time = arrival_times.get(num_stop_times / 2).unwrap_or("");
        let quartile_arrival_time = arrival_times.get(num_stop_times / 4).unwrap_or("");
        let three_quartile_arrival_time = arrival_times.get(3 * num_stop_times / 4).unwrap_or("");
        let one_third_arrival_time = arrival_times.get(num_stop_times / 3).unwrap_or("");

        // using Xxh64 as hashing algorithm because 64 bits should be enough to avoid collisions
        // and is reasonably quick
        let mut hasher = Xxh64::new(0);

        // idk the difference between to_le_bytes and as_bytes but compiler told me to do it
        hasher.update(&num_stops.to_le_bytes());
        hasher.update(&num_stop_times.to_le_bytes());
        hasher.update(&num_trips.to_le_bytes());
        hasher.update(&num_routes.to_le_bytes());

        hasher.update(middle_stop.as_bytes());
        hasher.update(middle_stop_time.as_bytes());
        hasher.update(middle_trip.as_bytes());
        hasher.update(middle_route.as_bytes());
        hasher.update(middle_arrival_time.as_bytes());
        hasher.update(quartile_arrival_time.as_bytes());
        hasher.update(three_quartile_arrival_time.as_bytes());
        hasher.update(one_third_arrival_time.as_bytes());

        let hash = hasher.digest();
        let cache_file_name = format!("{:016x}.bin", hash);
        let cache_file_path = self.cache_dir.join(cache_file_name);

        Ok(cache_file_path)
    }

    // using serde to serialize/deserialize timetable and id_map with rkyv
    pub fn load(&self, gtfs_feed: &RaptorGtfsFeed) -> Result<(RaptorTimetable, IdMap), RaptorError> {
        let cache_file_path = self.hash_feed(gtfs_feed)?;

        // check if file exists, and if not, err
        if !cache_file_path.exists() {
            let error_msg = format!(
                "Could not find cache at path '{}'",
                cache_file_path.to_string_lossy().to_string()
            );
            return Err(RaptorError::CacheError(error_msg));
        }

        let cache_bytes = match std::fs::read(&cache_file_path) {
            Ok(bytes) => bytes,
            Err(e) => {
                let error_msg = format!("failed to read cache: {}", e);
                return Err(RaptorError::CacheError(error_msg));
            }
        };

        // using rkyv safe api!
        let archived = match rkyv::access::<rkyv::Archived<(RaptorTimetable, IdMap)>, Error>(&cache_bytes) {
            Ok(a) => a,
            Err(e) => return Err(RaptorError::CacheError(format!("failed to validate cache: {}", e))),
        };

        let result = match deserialize::<(RaptorTimetable, IdMap), Error>(archived) {
            Ok(r) => r,
            Err(e) => return Err(RaptorError::CacheError(format!("failed to deserialize: {}", e))),
        };

        Ok(result)
    }

    // to save to cache file
    pub fn save(
        &self,
        gtfs_feed: &RaptorGtfsFeed,
        timetable: &RaptorTimetable,
        id_map: &IdMap,
    ) -> Result<(), RaptorError> {
        let cache_file_path = self.hash_feed(gtfs_feed)?;

        match std::fs::create_dir_all(&self.cache_dir) {
            Ok(_) => {}
            Err(e) => return Err(RaptorError::CacheError(format!("failed to create cache dir: {}", e))),
        }

        // Have to clone because very scary error messages when tried not to
        let bytes = match rkyv::to_bytes::<Error>(&(timetable.clone(), id_map.clone())) {
            Ok(b) => b,
            Err(e) => return Err(RaptorError::CacheError(format!("failed to serialize: {}", e))),
        };

        match std::fs::write(&cache_file_path, &bytes) {
            Ok(_) => {}
            Err(e) => return Err(RaptorError::CacheError(format!("failed to write cache: {}", e))),
        }

        Ok(())
    }
}
