mod at_engine;
mod backend;
mod cli;
mod compile;
mod metrics;
mod model;
mod preflight;
mod profiles;
mod report_writer;
mod run;
mod trace_io;
mod util;

use clap::Parser;

use crate::cli::{Cli, Command};

fn main() {
    let cli = Cli::parse();
    let result = match cli.command {
        Command::Run {
            profile,
            at,
            repeat,
        } => run::run_command(profile, at, repeat),
        Command::Compile { input, output } => compile::compile_command(&input, &output),
    };

    if let Err(err) = result {
        eprintln!("error: {err}");
        std::process::exit(1);
    }
}
