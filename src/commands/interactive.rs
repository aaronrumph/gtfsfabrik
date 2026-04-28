use clap::{CommandFactory, Parser};
use rustyline::{DefaultEditor, error::ReadlineError};

use crate::{errors::commands::InteractiveError, types::scenario::FabrikScenario};

pub struct InteractiveOptions {
    // TODO: Add SQL?
    scenario: FabrikScenario,
    use_semicolons: bool,
}

pub fn run_interactive_repl(options: Option<InteractiveOptions>) -> Result<(), InteractiveError> {
    // TODO:
    Ok(())
}
