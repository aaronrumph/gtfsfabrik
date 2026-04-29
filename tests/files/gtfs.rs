// Test module for testing gtfs file handling
use gtfsfabrik::read_gtfs;

// All four of the following print... tests use google's sample GTFS data from:
// https://github.com/google/transit/tree/master/gtfs/spec/en/examples/sample-feed-1

// idk why Box<dyn std::error::Error>, compiler told me to do it
// NOTE: run test with cargo test -- --nocapture to get it to print
#[test]
#[ignore]
fn print_stops_df() -> Result<(), Box<dyn std::error::Error>> {
    let df = read_gtfs!("tests/inputs/files/example_gtfs", GtfsFiles::Stops);
    println!("{:?}", df);
    Ok(())
}

#[test]
#[ignore]
fn print_stop_times_df() -> Result<(), Box<dyn std::error::Error>> {
    let df = read_gtfs!("tests/inputs/files/example_gtfs", GtfsFiles::StopTimes);
    println!("{:?}", df);
    Ok(())
}

#[test]
#[ignore]
fn print_routes_df() -> Result<(), Box<dyn std::error::Error>> {
    let df = read_gtfs!("tests/inputs/files/example_gtfs", GtfsFiles::Routes);
    println!("{:?}", df);
    Ok(())
}

#[test]
#[ignore]
fn print_trips_df() -> Result<(), Box<dyn std::error::Error>> {
    let df = read_gtfs!("tests/inputs/files/example_gtfs", GtfsFiles::Trips);
    println!("{:?}", df);
    Ok(())
}

// this is a test to download, and then load CTA GTFS data

#[test]
#[ignore]
fn cta_gtfs_data_loading() -> Result<(), Box<dyn std::error::Error>> {
    println!("CTA GTFS data fetching and polars loading test: \n");
    let cta_gtfs_url = "https://www.transitchicago.com/downloads/sch_data/google_transit.zip";
    let cta_gtfs_dir = "tests/inputs/files/cta_gtfs";
    std::fs::create_dir_all(cta_gtfs_dir)?;

    // download CTA GTFS zip from the url
    let bytes = reqwest::blocking::get(cta_gtfs_url)?.bytes()?;

    // unzip into cta_gtfs_dir
    let mut archive = zip::ZipArchive::new(std::io::Cursor::new(bytes))?;
    for i in 0..archive.len() {
        let mut file = archive.by_index(i)?;
        let path = std::path::Path::new(cta_gtfs_dir).join(file.name());
        let mut output_file = std::fs::File::create(&path)?;
        std::io::copy(&mut file, &mut output_file)?;
    }

    println!("STOPS: \n");
    let df = read_gtfs!(cta_gtfs_dir, GtfsFiles::Stops);
    println!("{:?}", df);

    println!("AGENCY: \n");
    let df = read_gtfs!(cta_gtfs_dir, GtfsFiles::Agency);
    println!("{:?}", df);

    println!("ROUTES: \n");
    let df = read_gtfs!(cta_gtfs_dir, GtfsFiles::Routes);
    println!("{:?}", df);

    println!("TRIPS: \n");
    let df = read_gtfs!(cta_gtfs_dir, GtfsFiles::Trips);
    println!("{:?}", df);

    println!("CALENDAR \n");
    let df = read_gtfs!(cta_gtfs_dir, GtfsFiles::Calendar);
    println!("{:?}", df);
    Ok(())
}
