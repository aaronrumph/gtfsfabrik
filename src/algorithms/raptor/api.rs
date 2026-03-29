// This module contains some wrapper functions/structs to provide an API to call RAPTOR with

use std::path::PathBuf;

use crate::{
    algorithms::raptor::{
        cache::RaptorCache,
        gtfs_loader::{build_timetable, load_gtfs, map_ids},
        types::{IdMap, RaptorTimetable},
    },
    utils::errors::RaptorError,
};

pub struct Raptor {
    gtfs_dir: String,
    cache: RaptorCache,
    timetable: RaptorTimetable,
    id_map: IdMap,
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
            });
        }

        // otherwise try to read in from cache
        match cache.load(&feed) {
            Ok((cached_timetable, cached_id_map)) => Ok(Self {
                gtfs_dir: gtfs_dir.to_string(),
                cache,
                timetable: cached_timetable,
                id_map: cached_id_map,
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
                })
            }
        }
    }

    // TODO: Add docs!
    pub fn new(gtfs_dir: &str) -> Result<Self, RaptorError> {
        let cache = RaptorCache::default()?;
        Self::build(gtfs_dir, cache, true)
    }
}
