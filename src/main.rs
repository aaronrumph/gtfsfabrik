// using from elsewhere
use clap::Parser;
use gtfsfabrik::fabrik_io::logging::print_error;
use gtfsfabrik::{Cli, commands::run::run_command};

fn main() {
    // everything that would go in main is handled in run_command in
    // commands::run::run_command

    let cli = Cli::parse();

    // propogating all errors to top level
    if let Err(err) = run_command(cli.command) {
        print_error(&format!("{err}"));
        std::process::exit(1);
    }
}
