// This module creates the transfers needed for RAPTOR

use crate::{
    algorithms::raptor::types::{IdMap, RaptorGtfsFeed, RaptorStopID, RaptorTransfer, TransfersServingStop},
    utils::{errors::RaptorError, files::gtfs::StopColumns},
};
use rayon::prelude::*;

// TODO: Move to sensible place so can be reused
fn haversine_meters(lat1: f64, lon1: f64, lat2: f64, lon2: f64) -> f64 {
    let radius = 6_371_000.0; // Earth radius in meters
    let distance_lat = (lat2 - lat1).to_radians();
    let distance_long = (lon2 - lon1).to_radians();
    let a = (distance_lat / 2.0).sin().powi(2)
        + lat1.to_radians().cos() * lat2.to_radians().cos() * (distance_long / 2.0).sin().powi(2);
    2.0 * radius * a.sqrt().asin()
}

/// Calculates walking transfer times between stations for use in Raptor.
/// Naive because simply takes the distance between stations and calculates walking times at
/// 85 meters/min walking speed. Requires that feed has stop lats and lons so if not will error.
/// # Returns:
/// Vec<Vec<RaptorTransfer>> where the outer Vec represents the set of transfers for all stops,
/// with the index corresponding to the stops raptor ID, and the inner Vec represents the set of
/// transfers for that stop.
/// # Example:
/// For a stop with Raptor ID 10, all the transfers from it to other stations are:
/// ```
/// use crate::algorithms::raptor::transfers::calculate_naive_transfers;
/// let all_transfers = calculate_naive_transfers(some_feed, corresponding_id_map)?;
/// let our_stops_transfers: Vec<RaptorTransfer> = all_transfers[10];
///```
///
pub fn calculate_naive_transfers(
    feed: &RaptorGtfsFeed,
    id_map: &IdMap,
) -> Result<Vec<Vec<RaptorTransfer>>, RaptorError> {
    const WALK_SPEED_MPS: f64 = 1.4; // 85 meters/min (Vuchic book)
    const MAX_WALK_METERS: f64 = 1000.0; // 1 km rough max for a reasonable walk

    // using maximum possible lats and lons that could be 1km for quick bounds check
    const MAX_DISTANCE_DEGREES: f64 = MAX_WALK_METERS / 112_000.0;

    let num_stops = id_map.stops.len();
    let stop_coords = feed.get_stop_locations(id_map)?;

    // using rayon for parallelism
    // TODO: Use GPU for masssively parallel?
    let transfers_per_stop: Vec<(usize, TransfersServingStop)> = stop_coords
        .par_iter()
        .map(|&(stop_a, lat_a, lon_a)| {
            let mut transfers = TransfersServingStop::with_capacity(8);
            for &(stop_b, lat_b, lon_b) in &stop_coords {
                if stop_a == stop_b {
                    continue;
                }
                if (lat_a - lat_b).abs() > MAX_DISTANCE_DEGREES {
                    continue;
                }
                if (lon_a - lon_b).abs() > MAX_DISTANCE_DEGREES {
                    continue;
                }
                let dist_meters = haversine_meters(lat_a, lon_a, lat_b, lon_b);
                if dist_meters > MAX_WALK_METERS {
                    continue;
                }
                let walk_time = (dist_meters / WALK_SPEED_MPS) as usize;
                transfers.push(RaptorTransfer {
                    to_stop: stop_b,
                    walk_time,
                });
            }
            (stop_a.id, transfers)
        })
        .collect();

    // now moving them over once have calculated in parallel
    let mut all_transfers: Vec<TransfersServingStop> = vec![TransfersServingStop::new(); num_stops];
    for (stop_id, stop_transfers) in transfers_per_stop {
        all_transfers[stop_id] = stop_transfers;
    }

    Ok(all_transfers)
}

// TODO: OSM based walking transfers
