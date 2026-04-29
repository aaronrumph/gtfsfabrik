// Goal for this module is to group run_command and run file command because they are essentially
// the same thing

use crate::{
    Commands,
    commands::{self},
    errors::commands::{FabrikCommandError, *},
    fabrik_io::logging::print_success,
};

/// Thin wrapper around cli that gives function that runs the given command using clap
pub fn run_command(command: Commands) -> Result<(), FabrikCommandError> {
    // simple match on command
    match command {
        // if interactive, run repl
        Commands::Interactive {
            use_semicolons,
            scenario,
        } => {
            // TODO: change from Option? GET RID OF PANIC
            let interactive_options = commands::interactive::InteractiveOptions {
                use_semicolons,
                scenario_name: scenario,
            };
            commands::interactive::run_interactive_repl(Some(interactive_options))?;
            Ok(())
        }
        Commands::Init {
            path,
            gtfs,
            osm,
            place,
            geoscope,
            ridership,
            usegit,
        } => {
            let input_path = path.ok_or(InitError::NoPathProvided)?;
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

            commands::init::init_fabrik(options)?;

            print_success(&format!("Successfully created a new fabrik at {}", path_copy));

            Ok(())
        }
        _ => Err(FabrikCommandError::CommandNotImplemented),
    }
}
