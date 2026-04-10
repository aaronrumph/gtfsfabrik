use rayon::prelude::*;

use crate::algorithms::raptor::types::{
    Journey, Leg, RaptorRouteID, RaptorState, RaptorStopID, RaptorTimetable, INFINITY,
};
use crate::gtfs::datetime::Seconds;
use crate::utils::errors::RaptorError;

// Raptor!
#[derive(Debug, Clone)]
pub struct SimpleRaptor {
    pub timetable: RaptorTimetable,
    // TODO: initialize RaptorState here?
}

impl SimpleRaptor {
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

        // short circuit if same origin and destination
        if origin == destination {
            return Ok(Journey::new(origin, destination, depart_time, vec![]));
        }

        // collect all stops from the timetable
        let mut state = RaptorState::new(
            origin,
            destination,
            depart_time,
            timetable.stops.len(),
            max_transfers,
            0,
        );

        // relax transfers from origin before round 1
        for transfer in &timetable.transfers[origin.id] {
            let arr = depart_time + transfer.walk_time;
            let leg = Leg {
                origin_stop: origin,
                destination_stop: transfer.to_stop,
                leg_start_time: depart_time,
                leg_end_time: arr,
                trip_id: None,
            };
            state.update(1, transfer.to_stop, arr, leg);
        }

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
                    timetable.routes_serving_stops[stop.id]
                        .iter()
                        .map(move |route| (route.route_id, stop, route.stop_sequence))
                })
                .collect();

            // clear marked stops for this round
            for &stop in &state.marked_stops {
                state.marked[stop.id] = false;
            }
            state.marked_stops.clear();

            // this (RaptorStopID, usize) tuple gives the stop id and it's ordering in the route
            let mut route_queue: Vec<Option<(RaptorStopID, usize)>> = vec![None; timetable.routes.len()];
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
                        if let Some(trip_idx) = timetable.earliest_trip(route_id, stop_idx, prev_arr) {
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
