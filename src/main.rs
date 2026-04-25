// local modules
pub mod algorithms;
pub mod commands;
pub mod gtfs;
pub mod osm;
pub mod utils;

use algorithms::raptor::api::Raptor;
use utils::geotypes::GeoScope;
use utils::logging::*;

// external crates
use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(
    name = "gtfsfabrik",
    about = "An all-in-one, user-friendly, (blazingly?) fast tool for all your GTFS needs",
    version,
    propagate_version = true, // so can use --version on any subcommand,
    color = clap::ColorChoice::Always,
)]
struct CLI {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    // TODO: create getting-started page
    /// Creates a new gtfsfabrik plant (run gtfsfabrik docs getting-started for help)
    Init {
        /// Path where the fabrik plant will be created. Relative or absolute
        /// paths accepted. Defaults to the current working directory.
        path: Option<String>,

        /// Path to GTFS data. Can be a zip file, unzipped folder, or a folder
        /// containing multiple GTFS zip files or unzipped folders. Seperate
        /// multiple paths with commas or spaces
        #[arg(long, num_args = 1..)]
        gtfs: Option<Vec<String>>,

        /// Path to OSM PBF data.
        #[arg(long)]
        osm: Option<String>,

        /// Name of the base place to use as the plant's main location.
        /// Required if using --geoscope.
        #[arg(long)]
        place: Option<String>,

        /// Geographic scope for the plant. Used in conjunction with --place.
        #[arg(long)]
        geoscope: Option<GeoScope>,

        /// Path to a ridership CSV file. See docs/inputs/ridership.md.
        #[arg(long)]
        ridership: Option<String>,

        /// Initialize a git repository in the plant. Defaults to true.
        #[arg(long, default_value_t = true)]
        usegit: bool,
    },

    Scenario {
        // TODO: scenario command implementation
    },

    Add {
        // TODO: add command implementation
    },

    Remove {
        // TODO: remove command implementation
    },

    Stash {
        // TODO: stash command implementation
    },

    Unstash {
        // TODO: unstash command implementation
    },

    Version {
        // similar to committing in git. Better name??
        // TODO: implement version command
    },

    List {
        // TODO: list command implemenation
    },

    Summary {
        // TODO: summary command implementation
    },
}

fn main() {
    let cli = CLI::parse();

    match cli.command {
        Commands::Init {
            path,
            gtfs,
            osm,
            place,
            geoscope,
            ridership,
            usegit,
        } => {
            // TODO: add error handling for no path
            // BUG: will panic if no path provided. Give user error message
            let input_path = path.expect("ERROR: You must provide a path for the fabrik! Could not create fabrik");
            // TODO: change from unwrap for safety once know
            // commands work

            let path_copy = input_path.clone();

            let options = commands::init::InitOptions {
                path: input_path,
                gtfs,
                osm,
                place,
                geoscope: geoscope.map(|g| format!("{:?}", g).to_lowercase()),
                ridership,
                usegit,
            };

            match commands::init::init_fabrik(options) {
                Ok(_) => {
                    let init_success_message = &format!("Successfully created a new fabrik at {}", path_copy);
                    print_success(init_success_message);
                }
                Err(e) => {
                    let init_error_message = &format!("Couldn't create a new fabrik at {}.\n{}", path_copy, e);
                    print_error(init_error_message);
                }
            }
        }

        _ => println!("Sorry that command is not available yet!"),
        // TODO: match statement/error handling for the other commands once implemented
    }
}
