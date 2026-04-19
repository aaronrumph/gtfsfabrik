use std::cmp::Ordering;

// THIS IS A NEW GROUND UP ATTEMPT AT RAPTOR BECAUSE PREVIOUS ATTEMPT WAS MORE CONVALUTED
use rayon::prelude::*;

use crate::algorithms::raptor::types::{
    INFINITY, Journey, Leg, RaptorQueryResult, RaptorRouteID, RaptorRouteServingStop, RaptorState, RaptorStopID, RaptorTimetable, RaptorTransfer
};
use crate::gtfs::datetime::Seconds;
use crate::utils::errors::RaptorError;

// RAPTOR obj/class
#[derive(Debug, Clone)]
pub struct RaptorHandler {
    // NOTE: moving state out of seperate class into here
    pub timetable: RaptorTimetable,

    num_stops: usize, // helpful to have on hand

    // tau_prev[stop] is earilest arrival time at stop after k-1 rounds
    tau_prev: Vec<Seconds>,
    // tau_current[stop] is earliest arrival time at stop after k rounds (current round
    tau_current: Vec<Seconds>,
    // tau_best[stop] keeps track of the earliest arrival time in any round
    tau_best: Vec<Seconds>,

    // true at index if that stop_id has been marked this round
    marked_prev: Vec<bool>,
    marked_current: Vec<bool>,

    // array of all stops that have been marked this round
    marked_stops_prev: Vec<RaptorStopID>,
    marked_stops_current: Vec<RaptorStopID>,

    // parent_leg[round][stop] returns the leg used to reach that stop in a given round for journey
    parent_leg: Vec<Vec<Option<Leg>>>,
}

impl RaptorHandler {
    // pub so that can keep state of RAPTOR private
    pub fn new(timetable: RaptorTimetable) -> Self {
        // thin wrapper around initialize so can call new from elsewhere
        RaptorHandler::initialize(timetable)
    }

    // the purpose of seperating out initialize is to be able to set up RaptorHandler for a given
    // timetable once, and clone if need be (will bench to see if faster)
    // is cleaner seperation though because keeps query params in query fn
    fn initialize(timetable: RaptorTimetable) -> Self {
        // initializing
        let num_stops = timetable.stops.len();

        // initializing the tau tables
        let tau_prev = vec![INFINITY; num_stops];
        let tau_current = vec![INFINITY; num_stops];
        let tau_best = vec![INFINITY; num_stops];

        // no marked stops to start
        let marked_prev = vec![false; num_stops];
        let marked_current = vec![false; num_stops];

        let parent_leg = vec![];

        Self {
            timetable,
            tau_prev,
            tau_current,
            tau_best,
            marked_prev,
            marked_current,
            marked_stops_prev: vec![],
            marked_stops_current: vec![],
            parent_leg,
            num_stops,
        }
    }

    pub fn query(
        &mut self,
        origin: RaptorStopID,
        destination: RaptorStopID,
        departure_time: Seconds,
        max_transfers: Option<usize>,
    ) -> Result<RaptorQueryResult, RaptorError> {
        // TODO: Need to make state clear function to make sure can reuse obj

        // we start with 0, where round 0 is 0 transfers (so only initial stop and
        // walkable neighbors)
        let mut query_round = 0;

        // If no max_transfers, default big num
        let max_transfers = match max_transfers {
            Some(num) => num,
            None => 10  // not absurdly large, but large enough that should never happen
        };
        let max_boardings = max_transfers + 1; // useful to avoid off by one errors

        // initializing leg tracker (+ 1 because of just walk round)
        self.parent_leg = vec![vec![None; self.num_stops]; max_boardings + 1];


        // mark and update origin
        self.marked_current[origin.id] = true;
        self.marked_stops_current.push(origin);
        
        self.tau_current[origin.id] = departure_time;
        self.tau_best[origin.id] = departure_time;

        // for each nearby stop mark earliest arrival time as depart + transfer time
        for transfer in self.timetable.transfers[origin.id].iter() {
            let transfer_dest_id = transfer.to_stop;
            let arrival_time = departure_time + transfer.walk_time;

            self.tau_current[transfer_dest_id.id] = arrival_time;
            self.tau_best[transfer_dest_id.id] = arrival_time;

            // adding 0th round leg info so that can see initial walk to stop
            self.parent_leg[0][transfer_dest_id.id] = Some(Leg { 
                origin_stop: origin, 
                destination_stop: transfer_dest_id, 
                leg_start_time: departure_time, 
                leg_end_time: arrival_time, 
                trip_id: None});

            // mark in both lists for initial round
            self.marked_stops_current.push(transfer_dest_id);
            self.marked_current[transfer_dest_id.id] = true;
        }
        // now properly initialized, can begin round by round algo. 


        // first, helper function that gets state to proper place for start of next round (returns
        // new round if allowed)
        fn increment_round(starting_round: usize, max_boardings: usize, raptor_handler: &mut RaptorHandler) -> Option<usize> {
            // check for max transfers limit
            if starting_round >= max_boardings { // SAFETY: should never be greater but jic
                return None;
            } else {

                let num_stops = raptor_handler.num_stops;

                // move current into prev, clear current, and clear marked
                // QUESTION: way to get out of cloning here? Memswap? 
                raptor_handler.tau_prev = raptor_handler.tau_current.clone();
                raptor_handler.tau_current = vec![INFINITY; num_stops];

                // QUESTION: same thing here, way to avoid cloning?
                raptor_handler.marked_prev = raptor_handler.marked_current.clone();
                raptor_handler.marked_current = vec![false; num_stops];

                raptor_handler.marked_stops_prev = raptor_handler.marked_stops_current.clone();
                raptor_handler.marked_stops_current = vec![];

                // FIX: memswapping should be more efficient because no deep cloning required and
                // same initialization of vectors involved

                Some(starting_round + 1)
            }
        }

        // helper function to return the best route so far so can short circuit easily
        fn best_so_far(origin: RaptorStopID, destination: RaptorStopID, departure_time: Seconds, raptor_handler: &RaptorHandler) -> Result<RaptorQueryResult, RaptorError> {

            // easy part; check tau_best for arrival time
            let earliest_arrival_time = match raptor_handler.tau_best[destination.id] {
                INFINITY => return Err(RaptorError::DestinationUnreachable { origin, destination, departure_time }),
                some_real_time => some_real_time,
            }

            let travel_time = earliest_arrival_time - departure_time;

            // TODO: Diary construction!!!

            Ok(RaptorQueryResult { earliest_arrival_time, travel_time, diary: None })
        }

        // now can start going round by round, starting with round 1
        let mut query_round = match increment_round(0, max_boardings, self) {
            // returns None if need to short circuit
            Some(round) => round,
            None => return best_so_far(origin, destination, departure_time, self),
        };

        while query_round <= max_boardings {

            // first building queue
            // because dense int ids for routes too, can fully reserve up front!
            // BENCH: better to reserve like this in practice?
            let mut route_queue: Vec<Option<usize>> = vec![None; self.timetable.routes.len()];


            // BENCH: cf. par_iter() vs iter
            // go through all stops that were marked in previous round
            for stop_id in self.marked_stops_prev.iter() {
                // go through each route that serves stop and add to earliest boarding opp to queue
                for route_serving_stop in self.timetable.routes_serving_stops[stop_id.id].iter() {
                    let route_id = route_serving_stop.route_id;
                    let stop_sequence = route_serving_stop.stop_sequence;

                    // have to avoid adding route to queue twice
                    match route_queue[route_id.id] {
                        // if route already in queue && got on earlier or same point, leave be
                        Some(existing_sequence) if existing_sequence <= stop_sequence => {}
                        // else, put in queue/replace with earlier boarding
                        _ => route_queue[route_id.id] = Some(stop_sequence),
                    }
                }
            }


            // BENCH: cf. par_iter
            // now need to mark stops/scan along the routes
            for (route_idx, maybe_start_sequence) in route_queue.iter().enumerate() {
                let start_position = match maybe_start_sequence {
                    Some(sequence) => *sequence,
                    None => continue,
                };

                // checking whether better trip possible from past round (FIFO baby)
                let route_id = RaptorRouteID::new(route_idx);
                let route = &self.timetable.routes[route_idx];
                let mut current_trip_idx: Option<usize> = None;
                let mut boarded_at_stop_sequence: Option<usize> = None;

                // go forward along route ot next stop
                for stop_sequence in start_position..route.stops.len() {
                    let stop_id = route.stops[stop_sequence];

                    // improve stop arrival with currently boarded trip if possible
                    if let Some(trip_idx) = current_trip_idx {
                        let arrival_time =
                            self.timetable.get_arrival_time(route_id, trip_idx, stop_sequence);

                        if arrival_time < self.tau_best[stop_id.id] {
                            self.tau_current[stop_id.id] = arrival_time;
                            self.tau_best[stop_id.id] = arrival_time;

                            // need to mark down leg info once updated best arrival time for stop
                            if let Some(board_seq) = boarded_at_stop_sequence {
                                // makes it so that if switched boarding stop for trip...
                                self.parent_leg[query_round][stop_id.id] = Some(Leg {
                                    origin_stop: route.stops[board_seq],
                                    destination_stop: stop_id,
                                    leg_start_time: self.timetable.get_departure_time(route_id, trip_idx, board_seq),
                                    leg_end_time: arrival_time,
                                    trip_id: Some(route.trips[trip_idx].trip_id),
                                });
                            }

                            // check only here cuz want to avoid using hashset and don't want
                            // duplicates
                            if !self.marked_current[stop_id.id] {
                                self.marked_current[stop_id.id] = true;
                                self.marked_stops_current.push(stop_id);
                            }
                        }
                    }

                    // TODO: FIX NESTING HERE!!!

                    // check if prev arrival time would have let board earlier trip
                    let prev_arrival_time = self.tau_prev[stop_id.id];
                    if prev_arrival_time != INFINITY {
                        if let Some(candidate_trip_idx) = self.timetable.earliest_trip(route_id, stop_sequence, prev_arrival_time) {
                            match current_trip_idx {
                                Some(existing_trip_idx) if existing_trip_idx <= candidate_trip_idx => {}
                                _ => {
                                    current_trip_idx = Some(candidate_trip_idx);
                                    boarded_at_stop_sequence = Some(stop_sequence);
                                }
                            }
                        }                    
                    }
                }
            }

            // relaxing transfers/footpaths/walking
            let newly_marked_stops = self.marked_stops_current.clone();
            // BENCH: par_iter vs iter
            // go through all possible transfers from stop to see if can improve EAT
            for stop_id in newly_marked_stops.iter() {
                for transfer in self.timetable.transfers[stop_id.id].iter() {
                    let transfer_dest_id = transfer.to_stop;
                    let arrival_time = self.tau_current[stop_id.id] + transfer.walk_time;


                    // same as initialization round // TODO: make into reusable function??
                    if arrival_time < self.tau_current[transfer_dest_id.id] {
                        self.tau_current[transfer_dest_id.id] = arrival_time;
                        self.tau_best[transfer_dest_id.id] =
                            self.tau_best[transfer_dest_id.id].min(arrival_time);

                        self.parent_leg[query_round][transfer_dest_id.id] = Some(Leg {
                            origin_stop: *stop_id,
                            destination_stop: transfer_dest_id,
                            leg_start_time: self.tau_current[stop_id.id],
                            leg_end_time: arrival_time,
                            trip_id: None, // cuz walking
                        });

                        // again avoiding duplication
                        if !self.marked_current[transfer_dest_id.id] {
                            self.marked_current[transfer_dest_id.id] = true;
                            self.marked_stops_current.push(transfer_dest_id);
                        }
                    }
                }
            }

            // stop the whole thing if no improvement anywhere for this entire round
            if self.marked_stops_current.is_empty() {
                break;
            }

            // otherwise go to next round
            query_round = match increment_round(query_round, max_boardings, self) {
                Some(round) => round,
                None => break,
            };
        }

        best_so_far(origin, destination, departure_time, self)
    }
}
