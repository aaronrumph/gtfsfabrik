// MCRaptor implemenation (multi-criteria)
use rayon::prelude::*;

use crate::algorithms::raptor::gtfs_loader::{build_timetable, load_gtfs, map_ids};
use crate::algorithms::raptor::types::{
    INFINITY, IdMap, Journey, Leg, RaptorGtfsFeed, RaptorRouteID, RaptorState, RaptorStop, RaptorStopID,
    RaptorTimetable,
};
use crate::gtfs::datetime::Seconds;
use crate::utils::errors::RaptorError;

// Raptor!
pub struct Raptor {
    pub feed: String,
    raptor_feed: RaptorGtfsFeed,
    timetable: RaptorTimetable,
    id_map: IdMap,
    state: Option<RaptorState>,
}

impl Raptor {
    pub fn new(feed: &str) -> Result<Self, RaptorError> {
        // internally build all the structs needed based solely on input gtfs_dir
        let raptor_feed = load_gtfs(feed)?;
        let id_map = map_ids(&raptor_feed)?;
        let timetable = build_timetable(&raptor_feed, &id_map)?;
        let state = None; // start off with no state, will be initialized in query
        let feed = feed.to_string();
        Ok(Self {
            feed,
            raptor_feed,
            timetable,
            id_map,
            state,
        })
    }

    /// RAPTOR query. Returns a Vector containing all non-dominated journeys.
    /// # Parameters:
    /// `departure_time`: (Seconds) The number of Seconds past midnight on the day desired
    /// `origin`: (&str) The GTFS stop id of the stop to use as the origin
    /// `destination`: (&str) The GTFS stop_id of the stop to use as the destination
    pub fn query(&self, departure_time: Seconds, origin: &str, destination: &str) -> Result<Vec<Journey>, RaptorError> {
        let origin_id = self.id_map.gtfs_id_to_raptor_id(origin)?;
        let destination_id = self.id_map.gtfs_id_to_raptor_id(destination)?;

        // TODO: Raptor algo implementaiton

        Err(RaptorError::EmptyJourney)
    }
}
