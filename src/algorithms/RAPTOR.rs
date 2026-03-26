use std::collections::HashMap;

use crate::gtfs::datetime::Seconds;
use crate::gtfs::types::route::RouteID;
use crate::gtfs::types::stop::StopID;
use crate::gtfs::types::trip::TripID;
use crate::utils::errors::RaptorError;

// Types/structs needed for Raptor (reusing Stop,Route,TripID from main GTFS types)
const INFINITY: Seconds = Seconds::MAX;

// giving all types/structs Raptor in name to avoid compiler yelling at me about same names
pub struct RaptorStop {
    pub stop_id: StopID,
    pub name: String,
}

pub struct RaptorStopTime {
    // no need for reference to which stop because always accessed in order for raptor
    pub arrival: Seconds,
    pub departure: Seconds,
}

pub struct RaptorTrip {
    pub trip_id: TripID,
    pub stop_times: Vec<RaptorStopTime>,
}

pub struct RaptorRoute {
    pub route_id: RouteID,
    pub stops: Vec<StopID>,
    pub trips: Vec<RaptorTrip>,
}

pub struct RaptorTransfer {
    pub from_stop: StopID,
    pub to_stop: StopID,
    pub walk_time: Seconds,
}

// The timetable is THE source for all info to implement the algorithm
pub struct RaptorTimetable {
    pub stops: HashMap<StopID, RaptorStop>,
    pub routes: HashMap<RouteID, RaptorRoute>,
    pub transfers: HashMap<StopID, Vec<RaptorTransfer>>,
    pub routes_serving_stop: HashMap<StopID, Vec<(RouteID, usize)>>,
}

impl RaptorTimetable {
    pub fn earliest_trip(
        &self,
        route_id: &RouteID,
        stop_idx: usize, // stop_idx is which the order/sequence of stop in route
        earliest_departure: Seconds,
    ) -> Option<usize> {
        let route = self.routes.get(route_id)?;
        route
            .trips
            .iter()
            .position(|trip| trip.stop_times[stop_idx].departure >= earliest_departure)
    }

    // Takes gets the earliest arrival for a given route for a given trip for a given stop (as
    // given by stop sequence position)
    pub fn get_arrival_time(
        &self,
        route_id: &RouteID,
        trip_idx: usize,
        stop_idx: usize,
    ) -> Seconds {
        self.routes[route_id].trips[trip_idx].stop_times[stop_idx].arrival
    }

    pub fn get_departure_time(
        &self,
        route_id: &RouteID,
        trip_idx: usize,
        stop_idx: usize,
    ) -> Seconds {
        self.routes[route_id].trips[trip_idx].stop_times[stop_idx].departure
    }
}

// each leg of the Journey, so can easily provide directions if need be
#[derive(Debug, Clone)]
pub struct Leg {
    pub origin_stop: StopID,
    pub destination_stop: StopID,
    pub leg_start_time: Seconds,
    pub leg_end_time: Seconds,
    pub trip_id: Option<TripID>, // will be None if walking!
}

// Total Journey
#[derive(Debug, Clone)]
pub struct Journey {
    pub origin: StopID,
    pub destination: StopID,
    pub departure_time: Seconds,
    pub legs: Vec<Leg>,
}

impl Journey {
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
    pub fn total_travel_time(&self) -> Result<usize, RaptorError> {
        let departure_time = self
            .legs
            .first()
            .map(|first| first.leg_start_time)
            .ok_or(RaptorError::EmptyJourney)?;
        let arrival_time = self.arrival_time();

        // origin and destination stop for nice error
        if arrival_time == INFINITY {
            Err(RaptorError::DestinationUnreachable {
                origin: self.origin.clone(),
                destination: self.destination.clone(),
                departure_time: self.departure_time.clone(),
            })
        } else {
            let time_elapsed = arrival_time - departure_time;
            Ok(time_elapsed)
        }
    }
}
