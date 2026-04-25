// types for RAPTOR
use polars::prelude::*;
use rayon::prelude::*;
use rkyv::{Archive, Deserialize, Serialize};
use std::collections::HashMap;

use crate::gtfs::datetime::Seconds;
use crate::utils::errors::RaptorError;
use crate::utils::files::gtfs::StopColumns;

// TODOS:
// TODO: Remove unnecessary cloning everywhere
// TODO: Fix spaghetti code nonsense in query function

// Types/structs needed for Raptor (reusing Stop,Route,RaptorTripID from main GTFS types)
pub const INFINITY: Seconds = Seconds::MAX;

// Route, stop, and trip ids specifically for raptor
#[derive(Debug, Archive, Serialize, Deserialize, Clone, Copy, PartialEq, Eq, Hash)]
pub struct RaptorRouteID {
    pub id: usize,
}

impl RaptorRouteID {
    pub fn new(id: usize) -> Self {
        Self { id }
    }
}

#[derive(Debug, Archive, Serialize, Deserialize, Clone, Copy, PartialEq, Eq, Hash)]
pub struct RaptorStopID {
    pub id: usize,
}

impl RaptorStopID {
    pub fn new(id: usize) -> Self {
        Self { id }
    }
}

#[derive(Debug, Archive, Serialize, Deserialize, Clone, Copy, PartialEq, Eq, Hash)]
pub struct RaptorTripID {
    pub id: usize,
}

impl RaptorTripID {
    pub fn new(id: usize) -> Self {
        Self { id }
    }
}

// giving all types/structs Raptor in name to avoid compiler yelling at me about same names
#[derive(Debug, Archive, Serialize, Deserialize, Clone, PartialEq, Eq, Hash)]
pub struct RaptorStop {
    pub stop_id: RaptorStopID,
    pub name: String,
}

#[derive(Debug, Archive, Serialize, Deserialize, Clone, Copy, PartialEq, Eq, Hash)]
pub struct RaptorStopTime {
    // no need for reference to which stop because always accessed in order for raptor
    pub arrival: Seconds,
    pub departure: Seconds,
}

#[derive(Debug, Archive, Serialize, Deserialize, Clone, PartialEq, Eq, Hash)]
pub struct RaptorTrip {
    pub trip_id: RaptorTripID,
    pub stop_times: Vec<RaptorStopTime>,
}

#[derive(Debug, Archive, Serialize, Deserialize, Clone, PartialEq, Eq, Hash)]
pub struct RaptorRoute {
    pub route_id: TimetableRouteID,
    pub stops: Vec<RaptorStopID>, // NOTE: Stops must be in sequential order along route
    pub trips: Vec<RaptorTrip>,   // NOTE: Trips must be in sequential order by depart time
}

#[derive(Debug, Archive, Serialize, Deserialize, Clone, Copy, PartialEq, Eq, Hash)]
pub struct RaptorTransfer {
    pub to_stop: RaptorStopID,
    pub walk_time: Seconds,
}

// NOTE: might want to change to CSR to avoid vector of vectors problems
#[derive(Debug, Archive, Serialize, Deserialize, Clone, Copy, PartialEq, Eq, Hash)]
pub struct RaptorRouteServingStop {
    pub route_id: TimetableRouteID,
    pub stop_sequence: usize,
}

// just a Vec<RouteServingStop> for a given stop, but makes it easier to read code
pub type RoutesServingStop = Vec<RaptorRouteServingStop>;
// same for transfers
pub type TransfersServingStop = Vec<RaptorTransfer>;

// The timetable is THE source for all info to implement the algorithm
// NOTE: !!!! vectors/arrays requires dense id range with no gaps so loader has to remap id's
#[derive(Debug, Archive, Serialize, Deserialize, Clone, PartialEq, Eq, Hash)]
pub struct RaptorTimetable {
    pub stops: Vec<RaptorStop>,
    pub routes: Vec<RaptorRoute>,
    pub transfers: Vec<TransfersServingStop>,
    pub routes_serving_stops: Vec<RoutesServingStop>,
}

impl RaptorTimetable {
    // TODO: binary search to make it faster?

    /// Finds the earliest trip for a given route id, for a given stop sequence idx for a given
    /// departure_time
    pub fn earliest_trip(
        &self,
        route_id: TimetableRouteID,
        stop_idx: usize, // stop_idx is which the order/sequence of stop in route
        earliest_departure: Seconds,
    ) -> Option<usize> {
        let route = self.routes.get(route_id.id)?;
        route
            .trips
            .iter()
            .position(|trip| trip.stop_times[stop_idx].departure >= earliest_departure)
    }

    // Takes gets the earliest arrival for a given route for a given trip for a given stop (as
    // given by stop sequence position)
    pub fn get_arrival_time(&self, route_id: TimetableRouteID, trip_idx: usize, stop_idx: usize) -> Seconds {
        self.routes[route_id.id].trips[trip_idx].stop_times[stop_idx].arrival
    }

    pub fn get_departure_time(&self, route_id: TimetableRouteID, trip_idx: usize, stop_idx: usize) -> Seconds {
        self.routes[route_id.id].trips[trip_idx].stop_times[stop_idx].departure
    }
}

// each leg of the Journey, so can easily provide directions if need be
#[derive(Debug, Archive, Serialize, Deserialize, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Leg {
    pub origin_stop: RaptorStopID,
    pub destination_stop: RaptorStopID,
    pub leg_start_time: Seconds,
    pub leg_end_time: Seconds,
    pub trip_id: Option<RaptorTripID>, // will be None if walking!
}

// Total Journey
#[derive(Debug, Archive, Serialize, Deserialize, Clone, PartialEq, Eq, Hash)]
pub struct Journey {
    pub origin: RaptorStopID,
    pub destination: RaptorStopID,
    pub departure_time: Seconds,
    pub legs: Vec<Leg>,
}

impl Journey {
    pub fn new(origin: RaptorStopID, destination: RaptorStopID, departure_time: Seconds, legs: Vec<Leg>) -> Self {
        Self {
            origin,
            destination,
            departure_time,
            legs,
        }
    }

    // gets the overall arrival time for the Journey (returns INFINITY if dest stop unreachable)
    pub fn arrival_time(&self) -> Seconds {
        self.legs.last().map(|last| last.leg_end_time).unwrap_or(INFINITY)
    }

    // returns the number of legs/transfers for the journery
    pub fn num_transfers(&self) -> usize {
        self.legs
            .iter()
            .filter(|leg| leg.trip_id.is_some())
            .count()
            .saturating_sub(1)
    }

    // returns the total time elapsed for the journey
    pub fn total_travel_time(&self) -> Result<Seconds, RaptorError> {
        let departure_time = self.departure_time;
        let arrival_time = self.arrival_time();

        // if couldn't reach destination, then arrival time will still be marked infinity
        if arrival_time == INFINITY {
            Err(RaptorError::DestinationUnreachable {
                origin: self.origin,
                destination: self.destination,
                departure_time: self.departure_time,
            })
        } else {
            let time_elapsed = arrival_time - departure_time;
            Ok(time_elapsed)
        }
    }
}

// current state tracker for algo. Using tau as per original RAPTOR paper
#[derive(Debug, Archive, Serialize, Deserialize, Clone, PartialEq, Eq, Hash)]
pub struct RaptorState {
    // tau_prev[stop] is earilest arrival time at stop after k-1 rounds
    pub tau_prev: Vec<Seconds>,

    // tau_current[stop] is earliest arrival time at stop after k rounds (current round
    pub tau_current: Vec<Seconds>,

    // tau_best[stop] keeps track of the earliest arrival time in any round
    pub tau_best: Vec<Seconds>,

    // parent_leg[(round, stop)] returns the leg used to reach that stop. Used to build journey
    // diary at end
    pub parent_leg: Vec<Vec<Option<Leg>>>,

    // true at index if that stop_id has been marked this round
    pub marked: Vec<bool>,

    // array of all stops that have been marked this round
    pub marked_stops: Vec<RaptorStopID>,
    pub destination_stop: RaptorStopID,
    pub departure_time: Seconds,

    pub round: usize,
}

impl RaptorState {
    // init a new RAPTOR query with origin stop and departure time
    pub fn new(
        origin: RaptorStopID,
        destination: RaptorStopID,
        departure_time: Seconds,
        num_stops: usize,
        max_transfers: usize,
        round: usize,
    ) -> Self {
        // initializing the tau arrays with INFINITY FOR all stops
        let mut tau_prev = vec![INFINITY; num_stops];
        let tau_current = vec![INFINITY; num_stops];
        let mut tau_best = vec![INFINITY; num_stops];

        // set first stop best arrival time to the departure time
        tau_prev[origin.id] = departure_time;
        tau_best[origin.id] = departure_time;

        let mut marked = vec![false; num_stops];
        marked[origin.id] = true;

        Self {
            tau_prev,
            tau_current,
            tau_best,
            parent_leg: vec![vec![None; num_stops]; max_transfers + 1],
            marked,
            marked_stops: vec![origin],
            destination_stop: destination,
            departure_time,
            round: 0,
        }
    }

    // getters for tau_prev, etc.
    pub fn tau_prev(&self, stop: &RaptorStopID) -> Seconds {
        self.tau_prev[stop.id]
    }
    pub fn tau_current(&self, stop: &RaptorStopID) -> Seconds {
        self.tau_current[stop.id]
    }
    pub fn tau_best(&self, stop: &RaptorStopID) -> Seconds {
        self.tau_best[stop.id]
    }

    // to start a new round, current tau starts as prev tau
    pub fn start_round(&mut self) {
        self.tau_current.copy_from_slice(&self.tau_prev);
    }

    // updates earliest arrival time for round/best if better. Returns true if new time is better
    // than old times
    pub fn update(&mut self, round: usize, stop: RaptorStopID, arrival_time: Seconds, leg: Leg) -> bool {
        // short circuit to prevent boarding before leaving
        if arrival_time < self.departure_time {
            return false;
        }
        let dest_best = self.tau_best[self.destination_stop.id];
        if arrival_time < self.tau_current[stop.id] && arrival_time < self.tau_best[stop.id] && arrival_time < dest_best
        {
            self.tau_current[stop.id] = arrival_time;
            self.tau_best[stop.id] = arrival_time;
            self.parent_leg[round][stop.id] = Some(leg);
            if !self.marked[stop.id] {
                self.marked[stop.id] = true;
                self.marked_stops.push(stop);
            }
            true
        } else {
            false
        }
    }

    pub fn finish_round(&mut self) {
        self.tau_prev.copy_from_slice(&self.tau_current);
    }
}

// SECTION: RaptorQueryResult

#[derive(Clone, Debug, Archive, Serialize, Deserialize)]
pub struct RaptorQueryResult {
    pub earliest_arrival_time: Seconds,
    pub travel_time: Seconds,
    pub diary: Option<Journey>,
}

// SECTION: RaptorGtfsFeed

#[derive(Clone)]
pub struct RaptorGtfsFeed {
    pub routes: DataFrame,
    pub trips: DataFrame,
    pub stops: DataFrame,
    pub stop_times: DataFrame,
}

impl RaptorGtfsFeed {
    pub fn get_stop_locations(&self, id_map: &IdMap) -> Result<Vec<(RaptorStopID, f64, f64)>, RaptorError> {
        let num_stops = id_map.stops.len();

        let stop_ids_col = self.stops.column(&StopColumns::StopID.to_string())?.str()?;
        let stop_lats_col = self.stops.column(&StopColumns::Latitude.to_string())?.str()?;
        let stop_lons_col = self.stops.column(&StopColumns::Longitude.to_string())?.str()?;

        // collect (raptor_stop_id, lat, long
        let mut stop_coords: Vec<(RaptorStopID, f64, f64)> = vec![];
        // same thing about going through rows
        for idx in 0..num_stops {
            let gtfs_stop_id = match stop_ids_col.get(idx) {
                Some(id) => id,
                None => {
                    return Err(RaptorError::InvalidGtfs(format!(
                        "missing stop_id in stops.txt at row {}",
                        idx
                    )));
                }
            };
            let stop_id = match id_map.stops.get(gtfs_stop_id) {
                Some(id) => *id,
                None => continue,
            };
            let latitude = match stop_lats_col.get(idx) {
                Some(s) => match s.parse::<f64>() {
                    Ok(lat) => lat,
                    Err(e) => {
                        let lat_error = RaptorError::StopLocationParsingError {
                            lat_or_lon: String::from("lat"),
                            row: idx,
                            source: e,
                        };
                        return Err(lat_error);
                    }
                },
                None => {
                    let error_msg = format!("missing stop_lat in stops.txt at row {}", idx);
                    return Err(RaptorError::InvalidGtfs(error_msg));
                }
            };

            let longitude = match stop_lons_col.get(idx) {
                Some(s) => match s.parse::<f64>() {
                    Ok(lon) => lon,
                    Err(e) => {
                        let lon_error = RaptorError::StopLocationParsingError {
                            lat_or_lon: String::from("long"),
                            row: idx,
                            source: e,
                        };
                        return Err(lon_error);
                    }
                },
                None => {
                    let error_msg = format!("missing stop lon in stops.txt at row {}", idx);
                    return Err(RaptorError::InvalidGtfs(error_msg));
                }
            };
            stop_coords.push((stop_id, latitude, longitude));
        }
        Ok(stop_coords)
    }
}

// SECTION: IdMaps

#[derive(Debug, Archive, Serialize, Deserialize, Clone, PartialEq, Eq)]
pub struct IdMap {
    pub routes: HashMap<String, RaptorRouteID>,
    pub trips: HashMap<String, RaptorTripID>,
    pub stops: HashMap<String, RaptorStopID>,
}

pub struct ReverseIdMap {
    pub routes: HashMap<RaptorRouteID, String>,
    pub trips: HashMap<RaptorTripID, String>,
    pub stops: HashMap<RaptorStopID, String>,
}

// NOTE: because the ids have to be unique as per the GTFS spec, the id map is invertible, so
// providing easy way to convert between inverted and normal
pub trait Invertible {
    type Inverted;
    fn invert(&self) -> Self::Inverted;
}

impl Invertible for IdMap {
    type Inverted = ReverseIdMap;
    fn invert(&self) -> ReverseIdMap {
        ReverseIdMap {
            stops: self.stops.iter().map(|(key, value)| (*value, key.clone())).collect(),
            routes: self.routes.iter().map(|(key, value)| (*value, key.clone())).collect(),
            trips: self.trips.iter().map(|(key, value)| (*value, key.clone())).collect(),
        }
    }
}

impl Invertible for ReverseIdMap {
    type Inverted = IdMap;
    fn invert(&self) -> IdMap {
        IdMap {
            stops: self.stops.iter().map(|(key, value)| (value.clone(), *key)).collect(),
            routes: self.routes.iter().map(|(key, value)| (value.clone(), *key)).collect(),
            trips: self.trips.iter().map(|(key, value)| (value.clone(), *key)).collect(),
        }
    }
}

impl IdMap {
    /// Function to take a gtfs_id as a &str and map it to the corresponding internal raptor id
    pub fn gtfs_id_to_raptor_id(&self, gtfs_id: &str) -> Result<RaptorStopID, RaptorError> {
        match self.stops.get(gtfs_id) {
            Some(raptor_id) => Ok(*raptor_id),
            None => {
                let error_msg = format!("Could not find GTFS id: {}", gtfs_id);
                Err(RaptorError::InvalidGtfs(error_msg))
            }
        }
    }
}

impl ReverseIdMap {
    pub fn raptor_id_to_gtfs_id(&self, raptor_id: RaptorStopID) -> Result<String, RaptorError> {
        match self.stops.get(&raptor_id) {
            Some(gtfs_id) => Ok(gtfs_id.clone()),
            None => {
                let readable_raptor_id = raptor_id.id.to_string();
                let error_msg = format!("Could not find Raptor id: {}", readable_raptor_id);
                Err(RaptorError::InvalidGtfs(error_msg))
            }
        }
    }
}

// SECTION: TimeTableRouteID

#[derive(Debug, Archive, Serialize, Deserialize, Clone, Copy, PartialEq, Eq, Hash)]
pub struct TimetableRouteID {
    pub id: usize,
}

impl TimetableRouteID {
    pub fn new(id: usize) -> Self {
        Self { id }
    }
}
