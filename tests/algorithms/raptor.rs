use std::error::Error;

use gtfsfabrik::{
    algorithms::raptor::{api::Raptor, types::RaptorQueryResult},
    gtfs::datetime::Seconds,
    utils::errors::RaptorError,
};

#[test]
#[ignore]
fn quick_travel_time() {
    // quick test to check that quick travel time returns good results!
    println!("Starting quick travel time test");

    let gtfs_stop_id_1 = "60004"; // Calrk & Div red line
    let gtfs_stop_id_2 = "1"; // Jackson Austin bus term
    let departure_time = 28800;
    let max_transfers = 10;

    println!("Creating raptor obj");
    // if cta_gtfs not run yet won't work
    // FIX: fix so that don't have to run cta_gtfs download test first
    let raptor_obj = Raptor::new("tests/inputs/files/cta_gtfs");

    // quick error handling fix for this test
    let mut raptor_obj = match raptor_obj {
        Ok(obj) => obj,
        Err(errmsg) => panic!("Error: {}", errmsg),
    };

    println!("Getting travel time");
    let query_result = raptor_obj.travel_time(gtfs_stop_id_1, gtfs_stop_id_2, departure_time, 10);

    // a bit quick and dirty
    match query_result {
        Ok(result) => println!(
            "Travel time from {} to {} is {}",
            gtfs_stop_id_1, gtfs_stop_id_2, result
        ),
        Err(_) => println!("ERROR"),
    }
}
