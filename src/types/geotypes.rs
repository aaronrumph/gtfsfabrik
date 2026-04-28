use clap::ValueEnum;
use geo::Polygon;
use serde::Serialize;

#[derive(Debug, Clone, ValueEnum, Serialize)]
pub enum GeoScope {
    Place,
    County,
    Msa,
    Csa,
}

#[derive(Debug, Serialize)]
pub struct Place {
    pub name: String,
    pub state: Option<String>,
    pub country: Option<String>,
    polygon: Option<Polygon>,
}

impl std::str::FromStr for Place {
    type Err = String;

    /// Create a place struct from a string. Includes name, state, country, lat, and lon
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let parts_of_place: Vec<&str> = s.split(',').map(|p| p.trim()).collect();

        Ok(Place {
            name: parts_of_place.get(0).unwrap_or(&"").to_string(),
            state: parts_of_place.get(1).map(|s| s.to_string()),
            country: parts_of_place.get(2).map(|s| s.to_string()),
            polygon: None, // TODO: Figure out if users should be able to give geojson as place
                           // input?
        })
    }
}

impl Place {
    //    pub fn geocode(&mut self) -> Result<(), GeocodingError> {
    // TODO: geocoding implementation using nomanitim
    // NOTE:: Will need to be able to geocode for whole geoscope (should update
    // polygon attr of Place struct)
    //  }
}
