use clap::{CommandFactory, Parser};
use rustyline::{DefaultEditor, error::ReadlineError};

use crate::{errors::commands::InteractiveError, types::scenario::FabrikScenario};

pub struct InteractiveOptions {
    // TODO: Add SQL?
    // FIX: LOAD SCENARIO ON REPL LOAD
    pub scenario_name: String,
    pub use_semicolons: bool,
}

pub fn run_interactive_repl(options: Option<InteractiveOptions>) -> Result<(), InteractiveError> {
    // TODO:
    Ok(())
}
