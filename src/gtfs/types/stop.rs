use geo::Coord;
use serde::Serialize;

// putting StopID struct seperately so can reuse in raptor
#[derive(Debug, Serialize, PartialEq, Eq, Hash, Clone)]
pub struct StopID {
    id: String,
}

impl StopID {
    pub fn new(id: &str) -> Self {
        Self { id: id.to_string() }
    }
}

#[derive(Debug, Serialize)]
pub struct Stop {
    // TODO: stop struct implementation
    pub stop_id: StopID,
    pub stop_code: Option<String>,
    pub stop_name: Option<String>,
    pub tts_stop_name: Option<String>,
    pub stop_desc: Option<String>,
    pub stop_lat: Option<f64>,
    pub stop_lon: Option<f64>,
    stop_coord: Option<Coord>,
}

impl Stop {
    /// Creates a new Stop, taking the stop_id as an input
    pub fn new(stop_id: &str) -> Self {
        Stop {
            stop_id: StopID::new(stop_id),
            stop_code: None,
            stop_name: None,
            tts_stop_name: None,
            stop_desc: None,
            stop_lat: None,
            stop_lon: None,
            stop_coord: None,
        }
    }

    /// Add coordinates to a Stop
    pub fn add_coordinates(&mut self, lat: f64, lon: f64) {
        self.stop_lat = Some(lat);
        self.stop_lon = Some(lon);
        self.stop_coord = Some(Coord { x: lon, y: lat });
    }

    /// Getter for coordinate
    pub fn coord(&self) -> Option<Coord> {
        self.stop_coord
    }
}
