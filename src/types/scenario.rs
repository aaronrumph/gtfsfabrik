// model for scenario

use crate::types::{
    agency::FabrikAgency, gtfs_dir::FabrikGtfsDir, line::FabrikLine, station::FabrikStation, vehicle::FabrikVehicle,
};

pub struct FabrikScenario {
    pub agencies: Option<Vec<FabrikAgency>>,
    pub vehicles: Option<Vec<FabrikVehicle>>,
    pub gtfs_dirs: Option<Vec<FabrikGtfsDir>>,
    pub lines: Option<Vec<FabrikLine>>,
    pub stations: Option<Vec<FabrikStation>>,
}
