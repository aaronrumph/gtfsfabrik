use serde::Serialize;

use crate::gtfs::datetime::Seconds;
use crate::utils::errors::RaptorError;

// TODOS:
// TODO: Create gtfs loader for RAPTOR
// TODO: Remove unnecessary cloning everywhere
// TODO: Fix spaghetti code nonsense in query function
// TODO: Change from HashMap<some_tuple> to Vec<Vec<..>> ?
// BUG: Should short circuit and return 0 if routing from station to same station

// Types/structs needed for Raptor (reusing Stop,Route,RaptorTripID from main GTFS types)
pub const INFINITY: Seconds = Seconds::MAX;

// Route, stop, and trip ids specifically for raptor
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct RaptorRouteID {
    pub id: usize,
}

impl RaptorRouteID {
    pub fn new(id: usize) -> Self {
        Self { id }
    }
}

#[derive(Debug, Serialize, Copy, PartialEq, Eq, Hash, Clone)]
pub struct RaptorStopID {
    pub id: usize,
}

impl RaptorStopID {
    pub fn new(id: usize) -> Self {
        Self { id }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct RaptorTripID {
    pub id: usize,
}

impl RaptorTripID {
    pub fn new(id: usize) -> Self {
        Self { id }
    }
}

// giving all types/structs Raptor in name to avoid compiler yelling at me about same names
#[derive(Debug, Clone)]
pub struct RaptorStop {
    pub stop_id: RaptorStopID,
    pub name: String,
}

#[derive(Debug, Clone, Copy)]
pub struct RaptorStopTime {
    // no need for reference to which stop because always accessed in order for raptor
    pub arrival: Seconds,
    pub departure: Seconds,
}

#[derive(Debug, Clone)]
pub struct RaptorTrip {
    pub trip_id: RaptorTripID,
    pub stop_times: Vec<RaptorStopTime>,
}

#[derive(Clone, Debug)]
pub struct RaptorRoute {
    pub route_id: RaptorRouteID,
    pub stops: Vec<RaptorStopID>,
    pub trips: Vec<RaptorTrip>,
}

#[derive(Clone)]
pub struct RaptorTransfer {
    pub to_stop: RaptorStopID,
    pub walk_time: Seconds,
}

// The timetable is THE source for all info to implement the algorithm
// NOTE: !!!! vectors/arrays requires dense id range with no gaps so loader has to remap id's
pub struct RaptorTimetable {
    pub stops: Vec<RaptorStop>,
    pub routes: Vec<RaptorRoute>,
    pub transfers: Vec<Vec<RaptorTransfer>>,
    pub routes_serving_stop: Vec<Vec<(RaptorRouteID, usize)>>,
}

impl RaptorTimetable {
    // TODO: binary search to make it faster?
    pub fn earliest_trip(
        &self,
        route_id: RaptorRouteID,
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
    pub fn get_arrival_time(
        &self,
        route_id: RaptorRouteID,
        trip_idx: usize,
        stop_idx: usize,
    ) -> Seconds {
        self.routes[route_id.id].trips[trip_idx].stop_times[stop_idx].arrival
    }

    pub fn get_departure_time(
        &self,
        route_id: RaptorRouteID,
        trip_idx: usize,
        stop_idx: usize,
    ) -> Seconds {
        self.routes[route_id.id].trips[trip_idx].stop_times[stop_idx].departure
    }
}

// each leg of the Journey, so can easily provide directions if need be
#[derive(Debug, Clone)]
pub struct Leg {
    pub origin_stop: RaptorStopID,
    pub destination_stop: RaptorStopID,
    pub leg_start_time: Seconds,
    pub leg_end_time: Seconds,
    pub trip_id: Option<RaptorTripID>, // will be None if walking!
}

// Total Journey
#[derive(Debug, Clone)]
pub struct Journey {
    pub origin: RaptorStopID,
    pub destination: RaptorStopID,
    pub departure_time: Seconds,
    pub legs: Vec<Leg>,
}

impl Journey {
    pub fn new(
        origin: RaptorStopID,
        destination: RaptorStopID,
        departure_time: Seconds,
        legs: Vec<Leg>,
    ) -> Self {
        Self {
            origin,
            destination,
            departure_time,
            legs,
        }
    }

    // gets the overall arrival time for the Journey (returns INFINITY if dest stop unreachable)
    pub fn arrival_time(&self) -> Seconds {
        self.legs
            .last()
            .map(|last| last.leg_end_time)
            .unwrap_or(INFINITY)
    }

    // returns the number of legs/transfers for the journery
    pub fn num_transfers(&self) -> usize {
        self.legs
            .iter()
            .filter(|leg| leg.trip_id.is_some())
            .count()
            .saturating_sub(1)
    }

    // TODO:: FIX THIS FUNCTION!
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
}

impl RaptorState {
    // init a new RAPTOR query with origin stop and departure time
    pub fn new(
        origin: RaptorStopID,
        destination: RaptorStopID,
        departure_time: Seconds,
        num_stops: usize,
        max_transfers: usize,
    ) -> Self {
        // initializing the tau arrays with INFINITY FOR all stops
        let mut tau_prev = vec![INFINITY; num_stops];
        let mut tau_current = vec![INFINITY; num_stops];
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
    pub fn update(
        &mut self,
        round: usize,
        stop: RaptorStopID,
        arrival_time: Seconds,
        leg: Leg,
    ) -> bool {
        // short circuit to prevent boarding before leaving
        if arrival_time < self.departure_time {
            return false;
        }
        let dest_best = self.tau_best[self.destination_stop.id];
        if arrival_time < self.tau_current[stop.id]
            && arrival_time < self.tau_best[stop.id]
            && arrival_time < dest_best
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
