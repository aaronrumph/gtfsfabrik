use std::error::Error;

// for random sampling of stops
use rand::seq::SliceRandom;

use gtfsfabrik::{
    algorithms::raptor::{api::Raptor, types::RaptorQueryResult},
    gtfs::datetime::{Seconds, seconds_to_gtfs_time},
    read_gtfs,
    utils::{errors::RaptorError, files::gtfs::GtfsFiles},
};

#[test]
#[ignore]
fn quick_travel_time() {
    // quick test to check that quick travel time returns good results!
    println!("Starting quick travel time test");

    let gtfs_stop_id_1 = "30171"; // O'Hare
    let gtfs_stop_id_2 = "7318"; // 130th Baltimore
    let departure_time = 43200;
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

#[test]
#[ignore]
fn diary_for_simple_trip() {
    // quick test to check that quick travel time returns good results!
    println!("Starting Trip Diary Test");

    let gtfs_stop_id_1 = "18604"; // Clark & Div red line
    let gtfs_stop_id_2 = "30193"; // Jackson Austin bus term
    let departure_time = 28800;
    let max_transfers = 10;

    println!("Creating RAPTOR object");
    // if cta_gtfs not run yet won't work
    // FIX: fix so that don't have to run cta_gtfs download test first
    let raptor_obj = Raptor::new("tests/inputs/files/cta_gtfs");

    // quick error handling fix for this test
    let mut raptor_obj = match raptor_obj {
        Ok(obj) => obj,
        Err(errmsg) => panic!("Error: {}", errmsg),
    };

    println!("Getting Trip Diary");
    println!();

    // yay simple functions
    let query_result = raptor_obj.trip_diary(gtfs_stop_id_1, gtfs_stop_id_2, departure_time, max_transfers);

    // a bit quick and dirty
    let diary = match query_result {
        Ok(result) => {
            println!(
                "Trip Diary from {} to {} is {:?}",
                gtfs_stop_id_1, gtfs_stop_id_2, result
            );
            result
        }
        Err(_) => panic!("ERROR"),
    };
}

#[test]
#[ignore]
fn readable_diary_for_simple_trip() {
    // NOTE: THIS IS THE TEST TO RUN IF YOU WANT TO CHECK WHETHER RAPTOR WORKS!!
    println!("Starting Trip Diary Test");

    let gtfs_stop_id_1 = "18604"; // Clark & Div red line
    let gtfs_stop_id_2 = "30193"; // Jackson Austin bus term
    let departure_time = 28800;
    let max_transfers = 10;

    println!("Creating RAPTOR object");

    // if cta_gtfs not run yet won't work
    // FIX: fix so that don't have to run cta_gtfs download test first
    let raptor_obj = Raptor::new("tests/inputs/files/cta_gtfs");

    // quick error handling fix for this test
    let mut raptor_obj = match raptor_obj {
        Ok(obj) => obj,
        Err(errmsg) => panic!("Error: {}", errmsg),
    };

    println!("Running Trip Diary query");
    println!();

    let query_result = raptor_obj.trip_diary_readable(gtfs_stop_id_1, gtfs_stop_id_2, departure_time, max_transfers);

    // just printing out the (now readable!) trip diary
    match query_result {
        Ok(readable_diary) => {
            println!("{}", readable_diary);
        }
        Err(err) => panic!("ERROR: {}", err),
    }
}

#[test]
#[ignore]
fn many_travel_time_queries() -> Result<(), Box<dyn std::error::Error>> {
    println!("Starting longer travel time test!");

    // NOTE: requires that cta_gtfs dir exists!!
    let cta_gtfs_dir = "tests/inputs/files/cta_gtfs";

    // read in stops.txt to dataframe
    let cta_gtfs_stops = read_gtfs!(cta_gtfs_dir, GtfsFiles::Stops);

    // pick 100 random stops from the stops dataframe to use as origins and destinations
    let mut rng = rand::thread_rng();
    let stop_ids: Vec<String> = cta_gtfs_stops
        .column("stop_id")?
        .str()?
        .into_iter()
        .filter_map(|opt| opt.map(|s| s.to_string()))
        .collect();

    // 20 stops gives 400 od-pairs/tests
    let random_stop_ids: Vec<String> = stop_ids.choose_multiple(&mut rng, 20).cloned().collect();

    // create raptor object
    let mut raptor_obj = Raptor::new(cta_gtfs_dir)?;

    // loop through 100 random origin destination pairs and get travel time
    let departure_time = 28800; // 8
    let max_transfers = 10;

    for origin in &random_stop_ids {
        for destination in &random_stop_ids {
            let query_result = raptor_obj.travel_time(origin, destination, departure_time, max_transfers);

            match query_result {
                Ok(result) => println!(
                    "Travel time from {} to {} is {}",
                    origin,
                    destination,
                    seconds_to_gtfs_time(result)
                ),
                Err(err) => println!("Error getting travel time from {} to {}: {}", origin, destination, err),
            }
        }
    }

    Ok(())
}
