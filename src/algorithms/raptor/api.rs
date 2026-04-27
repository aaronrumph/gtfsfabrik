// This module contains some wrapper functions/structs to provide an API to call RAPTOR with
use crate::{
    algorithms::raptor::{
        cache::RaptorCache,
        gtfs_loader::{build_timetable, load_gtfs, map_ids},
        simple_raptor::RaptorHandler,
        types::{
            IdMap, Invertible, Journey, RaptorGtfsFeed, RaptorQueryResult, RaptorStopID, RaptorTimetable, RaptorTripID,
            ReverseIdMap,
        },
    },
    gtfs::datetime::{Seconds, seconds_to_gtfs_time},
    utils::errors::RaptorError,
};

pub struct Raptor {
    gtfs_dir: String,
    cache: RaptorCache,
    timetable: RaptorTimetable,
    feed: RaptorGtfsFeed,
    id_map: IdMap,
    reverse_id_map: ReverseIdMap,
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

            // TODO: get rid of clone here!!! need to impl cache for ReverseIdMap
            let reverse_id_map = id_map.clone().invert();

            // write timetable and id_map to cache
            match cache.save(&feed, &timetable, &id_map) {
                Ok(_) => {}
                Err(e) => eprintln!("Warning: failed to save cache: {}", e),
            }

            return Ok(Self {
                gtfs_dir: gtfs_dir.to_string(),
                cache,
                timetable,
                feed,
                id_map,
                reverse_id_map,
                raptor_handler: None, // None until first (simple) query, then initializes
            });
        }

        // otherwise try to read in from cache
        match cache.load(&feed) {
            Ok((cached_timetable, cached_id_map)) => {
                // TODO: fix reverse id_map!!
                let reverse_id_map = cached_id_map.clone().invert();

                Ok(Self {
                    gtfs_dir: gtfs_dir.to_string(),
                    cache,
                    timetable: cached_timetable,
                    feed,
                    id_map: cached_id_map,
                    reverse_id_map,
                    raptor_handler: None,
                })
            }
            // TODO: Better error handling here for cache failures
            Err(_) => {
                // Reading from cache failed so build from scratch
                let id_map = map_ids(&feed)?;
                let timetable = build_timetable(&feed, &id_map)?;
                cache.save(&feed, &timetable, &id_map)?;

                // TODO: fix reverse id map!!
                let reverse_id_map = id_map.clone().invert();

                Ok(Self {
                    gtfs_dir: gtfs_dir.to_string(),
                    cache,
                    timetable,
                    feed,
                    id_map,
                    reverse_id_map,
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
    pub fn trip_diary(
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

        // TODO: get rid of expect here
        Ok(query_result.diary.expect("Expected a valid trip diary"))
    }

    pub fn trip_diary_readable(
        &mut self,
        origin_stop: &str,      // gtfs id
        destination_stop: &str, // gtfs id
        depart_time: Seconds,
        max_transfers: usize,
    ) -> Result<String, RaptorError> {
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

        // results of query
        let total_travel_time = query_result.travel_time;
        // TODO: get rid of expect here
        let trip_diary = query_result.diary.expect("Expected a valid trip diary");

        // using helper function
        let human_readable_diary = self.make_human_readable(&trip_diary, total_travel_time)?;
        Ok(human_readable_diary)
    }

    // helper function to make trip diary human readable
    fn make_human_readable(&self, trip_diary: &Journey, travel_time: Seconds) -> Result<String, RaptorError> {
        // TODO: move arrival_time into Journey struct!

        // need to translate: Raptor ID-> GTFS ID -> GTFS name
        let origin_gtfs_name = self.raptor_stop_id_to_name(&trip_diary.origin)?;
        let destination_gtfs_name = self.raptor_stop_id_to_name(&trip_diary.destination)?;

        // human readable depature time and arrival/travel times
        let departure_time = seconds_to_gtfs_time(trip_diary.departure_time);
        let total_travel_time = seconds_to_gtfs_time(travel_time);

        let arrival_time = trip_diary.departure_time + travel_time;
        let arrival_time = seconds_to_gtfs_time(arrival_time);

        // this first part is trip overview ex: "'Monroe (Blue)' to 'Monroe (Red)' at 08:00:00"
        let mut human_readable_journey = format!(
            "{} to {}: \nDEPART AT: {} | ARRIVE AT: {} | TOTAL TRAVEL TIME: {}\n",
            origin_gtfs_name, destination_gtfs_name, departure_time, arrival_time, total_travel_time
        );

        // now go through legs and for each translate to readable
        if trip_diary.legs.is_empty() {
            return Err(RaptorError::EmptyJourney);
        }
        for leg in &trip_diary.legs {
            // same translation as above
            let leg_origin_name = self.raptor_stop_id_to_name(&leg.origin_stop)?;
            let leg_destination_name = self.raptor_stop_id_to_name(&leg.destination_stop)?;

            // human readable times
            let start_time = seconds_to_gtfs_time(leg.leg_start_time);
            let end_time = seconds_to_gtfs_time(leg.leg_end_time);

            let travel_time_for_leg = leg.leg_end_time - leg.leg_start_time;
            let travel_time_for_leg = seconds_to_gtfs_time(travel_time_for_leg);

            // trip_id is None if walking trip
            let trip_mode_info = match leg.trip_id {
                Some(raptor_trip_id) => {
                    // translate RAPTOR trip id -> GTFS Trip ID -> Name
                    self.raptor_trip_id_to_name(&raptor_trip_id)?
                }
                None => String::from("Walk"),
            };

            let human_readable_leg = format!(
                "{leg_origin_name} => {leg_destination_name} [{trip_mode_info}] | {start_time}-{end_time} | ({travel_time_for_leg} travel time)\n"
            );

            human_readable_journey.push_str(&human_readable_leg);
        }

        Ok(human_readable_journey)
    }

    // helper function to look up corresponding name for id
    fn raptor_stop_id_to_name(&self, raptor_id: &RaptorStopID) -> Result<String, RaptorError> {
        // need to translate: Raptor ID-> GTFS ID -> GTFS name

        // FIX: better strategy for name look ups because this is insanely slow
        let gtfs_id = match self.reverse_id_map.stops.get(raptor_id) {
            Some(id) => id,
            None => {
                let error_msg = String::from("Something bad happened when making human readable");
                return Err(RaptorError::UnknownError(error_msg));
            }
        };

        // BUG: assumes stop_name is required field and panics when can't find

        // just need to look up in dataframe
        let gtfs_stop_ids = self.feed.stops.column("stop_id")?.str()?;
        let gtfs_stop_names = self.feed.stops.column("stop_name")?.str()?;

        let stop_row_idx = gtfs_stop_ids
            .into_iter()
            .position(|id| id == Some(gtfs_id))
            .ok_or_else(|| RaptorError::InvalidGtfs(format!("Unknown stop_id {gtfs_id}")))?;

        gtfs_stop_names
            .get(stop_row_idx)
            .map(|s| s.to_string())
            .ok_or_else(|| RaptorError::InvalidGtfs(format!("Missing stop_name for stop_id {gtfs_id}")))
    }

    // helper function to turn internal trip id to pretty name to print
    fn raptor_trip_id_to_name(&self, raptor_id: &RaptorTripID) -> Result<String, RaptorError> {
        // FIX: again super super slow to constantly look up like this!

        // need to translate:
        // RaptorTripID -> GTFS Trip ID -> GTFS Route ID -> (pretty) Route name
        let gtfs_id = match self.reverse_id_map.trips.get(raptor_id) {
            Some(id) => id,
            None => {
                let error_msg = String::from("Something bad happened when making human readable");
                return Err(RaptorError::UnknownError(error_msg));
            }
        };

        // data columns from feed
        let gtfs_trip_ids = self.feed.trips.column("trip_id")?.str()?;
        let gtfs_route_ids = self.feed.trips.column("route_id")?.str()?;

        // FIX: This whole function is just a complete mess

        // from routes.txt
        // FIX: !!!!!!!
        let gtfs_routes_route_id = self.feed.routes.column("route_id")?.str()?;
        let gtfs_route_short_names = self.feed.routes.column("route_short_name")?.str()?;
        let gtfs_route_long_names = self.feed.routes.column("route_long_name")?.str()?;

        // find trip in feed
        let trip_row_idx = gtfs_trip_ids
            .into_iter()
            .position(|id| id == Some(gtfs_id))
            .ok_or_else(|| RaptorError::InvalidGtfs(format!("Unknown trip id {gtfs_id}")))?;

        // which GTFS route id does this trip have?
        let trip_gtfs_route_id = gtfs_route_ids
            .get(trip_row_idx)
            .map(|s| s.to_string())
            .ok_or_else(|| RaptorError::InvalidGtfs(format!("Missing route_id for trip_id {gtfs_id}")))?;

        // now find row in routes.txt for that route id
        let route_row_idx = gtfs_routes_route_id
            .into_iter()
            .position(|id| id == Some(&trip_gtfs_route_id))
            .ok_or_else(|| RaptorError::InvalidGtfs(format!("Unknown trip id {gtfs_id}")))?;

        // look up names AT LEAST ONE of which MUST be there!
        let route_short_name = gtfs_route_short_names.get(route_row_idx).map(|s| s.to_string());
        let route_long_name = gtfs_route_long_names.get(route_row_idx).map(|s| s.to_string());

        // making a pretty name depending on whether had short/long name
        let pretty_route_name = match (route_short_name, route_long_name) {
            (Some(short_name), Some(long_name)) => format!("{short_name}: {long_name}"),
            (Some(short_name), None) => format!("{short_name}"),
            (None, Some(long_name)) => format!("{long_name}"),

            (None, None) => {
                // should never happen!
                let error_msg =
                    format!("Route id: '{trip_gtfs_route_id}' is missing either a long name or short name!");
                return Err(RaptorError::InvalidGtfs(error_msg));
            }
        };

        // if made it this far then this success!
        Ok(pretty_route_name)
    }
}
