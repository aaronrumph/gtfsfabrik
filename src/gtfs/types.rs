// This module contains the types for GTFS data (agency, stop, etc.)
use geo::Coord;
use serde::Serialize;

#[derive(Debug, Serialize)]
pub struct Agency {
    // TODO: agency struct implementation
}

#[derive(Debug, Serialize)]
pub struct Stop {
    // TODO: stop struct implementation
    pub stop_id: String,
    pub stop_code: Option<String>,
    pub stop_name: Option<String>,
    pub tts_stop_name: Option<String>,
    pub stop_desc: Option<String>,
    pub stop_lat: Option<f64>,
    pub stop_long: Option<f64>,
    stop_coord: Option<Coord>,
}

#[derive(Debug, Serialize)]
pub struct Route {
    // TODO: route struct impl
}

#[derive(Debug, Serialize)]
pub struct Trip {
    // TODO: trip struct impl
}

#[derive(Debug, Serialize)]
pub struct StopTime {
    // TODO: Stop time impl
}

#[derive(Debug, Serialize)]
pub struct Calendar {
    // TODO: calendar impl
}

#[derive(Debug, Serialize)]
pub struct CalendarDate {
    // TODO: Calendar date impl
}

// TODO: SHAPES!! use geo crate to translate shapes to points/lines,etc

// TODO: FARE STUFF: fare attributes, rules, leg rules, products, etc.

// TODO: locations.geojson?? for ADA/paratransit stuff

// Eventual goal is to support and use (if necessary) every single gtfs file in the specification
