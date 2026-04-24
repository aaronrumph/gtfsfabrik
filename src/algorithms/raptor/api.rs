// This module contains some wrapper functions/structs to provide an API to call RAPTOR with
use crate::{
    algorithms::raptor::{
        cache::RaptorCache,
        gtfs_loader::{build_timetable, load_gtfs, map_ids},
        new_raptor::RaptorHandler,
        simple_raptor::SimpleRaptor,
        types::{IdMap, Journey, RaptorQueryResult, RaptorTimetable},
    },
    gtfs::datetime::Seconds,
    utils::errors::RaptorError,
};

pub struct Raptor {
    gtfs_dir: String,
    cache: RaptorCache,
    timetable: RaptorTimetable,
    id_map: IdMap,
    raptor_handler: Option<RaptorHandler>, // this way can reuse simple raptor obj
}

impl Raptor {
    // private build function that others can call out to
    fn build(gtfs_dir: &str, cache: RaptorCache, use_cache: bool) -> Result<Self, RaptorError> {
        let feed = load_gtfs(gtfs_dir)?;

        if !use_cache {
            // build from scratch
            let id_map = map_ids(&feed)?;
            let timetable = build_timetable(&feed, &id_map)?;

            // write timetable and id_map to cache
            match cache.save(&feed, &timetable, &id_map) {
                Ok(_) => {}
                Err(e) => eprintln!("Warning: failed to save cache: {}", e),
            }

            return Ok(Self {
                gtfs_dir: gtfs_dir.to_string(),
                cache,
                timetable,
                id_map,
                raptor_handler: None, // None until first (simple) query, then initializes
            });
        }

        // otherwise try to read in from cache
        match cache.load(&feed) {
            Ok((cached_timetable, cached_id_map)) => Ok(Self {
                gtfs_dir: gtfs_dir.to_string(),
                cache,
                timetable: cached_timetable,
                id_map: cached_id_map,
                raptor_handler: None,
            }),
            // TODO: Better error handling here for cache failures
            Err(_) => {
                // Reading from cache failed so build from scratch
                let id_map = map_ids(&feed)?;
                let timetable = build_timetable(&feed, &id_map)?;
                cache.save(&feed, &timetable, &id_map)?;

                Ok(Self {
                    gtfs_dir: gtfs_dir.to_string(),
                    cache,
                    timetable,
                    id_map,
                    raptor_handler: None,
                })
            }
        }
    }

    // TODO: Add docs!
    pub fn new(gtfs_dir: &str) -> Result<Self, RaptorError> {
        let cache = RaptorCache::default()?;
        Self::build(gtfs_dir, cache, true)
    }

    // TODO: Add gtfs name support instead of just ids!

    /// Simple travel time query for RAPTOR. Takes origin_stop and destination_stop as
    /// gtfs_ids and returns just the travel time between them
    pub fn travel_time(
        &mut self,
        origin_stop: &str,      // gtfs id
        destination_stop: &str, // gtfs id
        depart_time: Seconds,
        max_transfers: usize,
    ) -> Result<Seconds, RaptorError> {
        // error handling for origin and destination first to make sure valid gtfs ids
        let origin = *self
            .id_map
            .stops
            .get(origin_stop)
            .ok_or_else(|| RaptorError::InvalidGtfs(format!("Unknown stop {}", origin_stop)))?;

        let destination = *self
            .id_map
            .stops
            .get(destination_stop)
            .ok_or_else(|| RaptorError::InvalidGtfs(format!("Unknown stop {}", destination_stop)))?;

        // check if there's already a RaptorHandler obj, if not add it
        if self.raptor_handler.is_none() {
            self.raptor_handler = Some(RaptorHandler::new(self.timetable.clone()))
        }

        let unwrapped_handler = self
            .raptor_handler
            .as_mut()
            .expect("There should be a raptor handler by this point");

        let query_result = unwrapped_handler.query(origin, destination, depart_time, Some(max_transfers))?;

        Ok(query_result.travel_time)
    }

    // TODO: write trip diary function
    pub fn trip_details(
        &mut self,
        origin_stop: &str,      // gtfs id
        destination_stop: &str, // gtfs id
        depart_time: Seconds,
        max_transfers: usize,
    ) -> Result<Journey, RaptorError> {
        // error handling for origin and destination first to make sure valid gtfs ids
        let origin = *self
            .id_map
            .stops
            .get(origin_stop)
            .ok_or_else(|| RaptorError::InvalidGtfs(format!("Unknown stop {}", origin_stop)))?;

        let destination = *self
            .id_map
            .stops
            .get(destination_stop)
            .ok_or_else(|| RaptorError::InvalidGtfs(format!("Unknown stop {}", destination_stop)))?;

        // check if there's already a RaptorHandler obj, if not add it
        if self.raptor_handler.is_none() {
            self.raptor_handler = Some(RaptorHandler::new(self.timetable.clone()))
        }

        let unwrapped_handler = self
            .raptor_handler
            .as_mut()
            .expect("There should be a raptor handler by this point");

        let query_result: RaptorQueryResult =
            unwrapped_handler.query(origin, destination, depart_time, Some(max_transfers))?;

        Ok(query_result.diary.expect("Expected a valid trip diary"))
    }
}
