pub mod algorithms;
pub mod commands;
pub mod errors;
pub mod fabrik_io;
pub mod files;
pub mod gtfs;
pub mod types;

// external crates
use crate::types::geotypes::GeoScope;
use clap::{Parser, Subcommand};

// SECTION: Main CLI!

#[derive(Parser)]
#[command(
    name = "gtfsfabrik",
    about = "An all-in-one, user-friendly, (blazingly?) fast tool for all your GTFS needs",
    version,
    propagate_version = true, // so can use --version on any subcommand,
    color = clap::ColorChoice::Always,
)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand)]
pub enum Commands {
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

    Interactive {
        // TODO: Interactive shell a-la MariaDB/MySQL
    },
}
