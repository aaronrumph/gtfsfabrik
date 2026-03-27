use rayon::prelude::*;
use serde::Serialize;

use crate::gtfs::datetime::Seconds;
use crate::utils::errors::RaptorError;

// TODOS:
// TODO: Change RouteID, StopId, RaptorTripID, etc, to be ints not Strings
// TODO: Create gtfs loader for RAPTOR
// TODO: Remove unnecessary cloning everywhere
// TODO: Fix spaghetti code nonsense in query function
// TODO: Change from HashMap<some_tuple> to Vec<Vec<..>> ?

// Types/structs needed for Raptor (reusing Stop,Route,RaptorTripID from main GTFS types)
const INFINITY: Seconds = Seconds::MAX;

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
pub struct RaptorStop {
    pub stop_id: RaptorStopID,
    pub name: String,
}

pub struct RaptorStopTime {
    // no need for reference to which stop because always accessed in order for raptor
    pub arrival: Seconds,
    pub departure: Seconds,
}

pub struct RaptorTrip {
    pub trip_id: RaptorTripID,
    pub stop_times: Vec<RaptorStopTime>,
}

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
struct RaptorState {
    // tau_prev[stop] is earilest arrival time at stop after k-1 rounds
    tau_prev: Vec<Seconds>,

    // tau_current[stop] is earliest arrival time at stop after k rounds (current round
    tau_current: Vec<Seconds>,

    // tau_best[stop] keeps track of the earliest arrival time in any round
    tau_best: Vec<Seconds>,

    // parent_leg[(round, stop)] returns the leg used to reach that stop. Used to build journey
    // diary at end
    parent_leg: Vec<Vec<Option<Leg>>>,

    // true at index if that stop_id has been marked this round
    marked: Vec<bool>,

    // array of all stops that have been marked this round
    marked_stops: Vec<RaptorStopID>,

    destination_stop: RaptorStopID,
}

impl RaptorState {
    // init a new RAPTOR query with origin stop and departure time
    fn new(
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
        }
    }

    // getters for tau_prev, etc.
    fn tau_prev(&self, stop: &RaptorStopID) -> Seconds {
        self.tau_prev[stop.id]
    }
    fn tau_current(&self, stop: &RaptorStopID) -> Seconds {
        self.tau_current[stop.id]
    }
    fn tau_best(&self, stop: &RaptorStopID) -> Seconds {
        self.tau_best[stop.id]
    }

    // to start a new round, current tau starts as prev tau
    fn start_round(&mut self) {
        self.tau_current.copy_from_slice(&self.tau_prev);
    }

    // updates earliest arrival time for round/best if better. Returns true if new time is better
    // than old times
    fn update(
        &mut self,
        round: usize,
        stop: RaptorStopID,
        arrival_time: Seconds,
        leg: Leg,
    ) -> bool {
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

    fn finish_round(&mut self) {
        self.tau_prev.copy_from_slice(&self.tau_current);
    }
}

// Raptor!
pub struct Raptor {
    pub timetable: RaptorTimetable,
    // TODO: initialize RaptorState here?
}

impl Raptor {
    pub fn new(timetable: RaptorTimetable) -> Self {
        Self { timetable }
    }

    fn reconstruct(
        &self,
        state: &RaptorState,
        origin: RaptorStopID,
        destination: RaptorStopID,
        depart_time: Seconds,
        max_transfers: usize,
    ) -> Result<Journey, RaptorError> {
        // find the best round so far
        let best_round = (1..=max_transfers)
            .into_par_iter()
            .filter_map(|round| {
                state.parent_leg[round][destination.id]
                    .as_ref()
                    .map(|leg| (round, leg.leg_end_time))
            })
            .min_by_key(|&(_, arrival)| arrival)
            .map(|(round, _)| round)
            .ok_or(RaptorError::DestinationUnreachable {
                origin,
                destination,
                departure_time: depart_time,
            })?;

        // traceback strat
        let mut legs: Vec<Leg> = Vec::new();
        let mut round = best_round;
        let mut current_stop = destination;

        for _ in 0..max_transfers {
            if current_stop == origin {
                break;
            }
            let Some(leg) = state.parent_leg[round][current_stop.id].clone() else {
                break;
            };

            current_stop = leg.origin_stop;
            let is_transit = leg.trip_id.is_some(); // get value out before moving with push
            legs.push(leg);

            if is_transit {
                round = round.saturating_sub(1); // to avoid round -= 1 when round = 0
            }
        }

        // need to reverse to go from origin to destianation rather than opposite
        legs.reverse();
        Ok(Journey::new(origin, destination, depart_time, legs))
    }

    pub fn query(
        &self,
        origin: RaptorStopID,
        destination: RaptorStopID,
        depart_time: Seconds,
        max_transfers: usize,
    ) -> Result<Journey, RaptorError> {
        let timetable = &self.timetable;

        // collect all stops from the timetable
        let mut state = RaptorState::new(
            origin,
            destination,
            depart_time,
            timetable.stops.len(),
            max_transfers,
        );

        // rounds correlate to adding one new transfer (one new round is one additional transfer)
        // so this is main loop for algorithm
        for round in 1..=max_transfers {
            state.start_round();

            // for first part of round collect all route, boarding_stop, and stop_idx tuple from
            // marked stops (using rayon for parallel) then add into queue using only earliest
            // boarding stop per route
            let route_entries: Vec<(RaptorRouteID, RaptorStopID, usize)> = state
                .marked_stops
                .par_iter()
                .flat_map_iter(|&stop| {
                    timetable.routes_serving_stop[stop.id]
                        .iter()
                        .map(move |&(route_id, stop_idx)| (route_id, stop, stop_idx))
                })
                .collect();

            // clear marked stops for this round
            for &stop in &state.marked_stops {
                state.marked[stop.id] = false;
            }
            state.marked_stops.clear();

            // this (RaptorStopID, usize) tuple gives the stop id and it's ordering in the route
            let mut route_queue: Vec<Option<(RaptorStopID, usize)>> =
                vec![None; timetable.routes.len()];
            for (route_id, stop, stop_idx) in route_entries {
                match &route_queue[route_id.id] {
                    Some((_, queued_idx)) if stop_idx >= *queued_idx => {}
                    _ => route_queue[route_id.id] = Some((stop, stop_idx)),
                }
            }

            // second part of round, traverse each route and mark stops
            for (route_idx, entry) in route_queue.iter().enumerate() {
                let Some((_, boarding_idx)) = entry else { continue };
                let route_id = RaptorRouteID { id: route_idx };
                let route = &timetable.routes[route_idx];

                let mut current_trip: Option<usize> = None;
                let mut boarded_at: Option<(RaptorStopID, usize)> = None; // same tuple as before

                // have to dereference because can't use usize as iterator?? idk compiler told me
                // to do it
                for stop_idx in *boarding_idx..route.stops.len() {
                    let stop = &route.stops[stop_idx];

                    let prev_arr = state.tau_prev(stop);
                    // TODO: compiler/clippy says collapsible if statement

                    // only choose to board if new arrival time at that stop is better than
                    // prvioues
                    if prev_arr < INFINITY {
                        if let Some(trip_idx) =
                            timetable.earliest_trip(route_id, stop_idx, prev_arr)
                        {
                            let is_earlier = current_trip.map_or(true, |current| {
                                timetable.get_departure_time(route_id, trip_idx, stop_idx)
                                    < timetable.get_departure_time(route_id, current, stop_idx)
                            });
                            if is_earlier {
                                current_trip = Some(trip_idx);
                                boarded_at = Some((*stop, stop_idx));
                            }
                        }
                    }

                    // no trip boarded yet
                    let (Some(trip_idx), Some((board_stop, board_idx))) = (current_trip, boarded_at) else {
                        continue;
                    };

                    let arrival_time = timetable.get_arrival_time(route_id, trip_idx, stop_idx);

                    // TODO: solve cloning overuse

                    // construct the leg from current trip info
                    let leg = Leg {
                        origin_stop: board_stop,
                        destination_stop: *stop,
                        leg_start_time: timetable.get_departure_time(route_id, trip_idx, board_idx),
                        leg_end_time: arrival_time,
                        trip_id: Some(route.trips[trip_idx].trip_id),
                    };

                    // need to deref to use update
                    state.update(round, *stop, arrival_time, leg);
                }
            }
            // third part of algo: relax transfers (see if makes sense to walk to transfer)
            let improved_stops = state.marked_stops.clone();

            for stop in improved_stops {
                // for each stop, check if arr time with transfer would be better
                let transfers = &timetable.transfers[stop.id];
                if transfers.is_empty() {
                    continue;
                };

                let current_arrival = state.tau_current(&stop);
                for transfer in transfers {
                    let final_arrival = current_arrival + transfer.walk_time;
                    let leg = Leg {
                        origin_stop: stop,
                        destination_stop: transfer.to_stop,
                        leg_start_time: current_arrival,
                        leg_end_time: final_arrival,
                        trip_id: None, // none since walking
                    };

                    state.update(round, transfer.to_stop, final_arrival, leg);
                }
            }

            if state.marked_stops.is_empty() {
                break;
            }

            state.finish_round();
        }
        self.reconstruct(&state, origin, destination, depart_time, max_transfers)
    }
}
