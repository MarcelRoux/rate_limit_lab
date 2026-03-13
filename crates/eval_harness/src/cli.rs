use std::path::PathBuf;

use clap::{Parser, Subcommand};

#[derive(Debug, Parser)]
#[command(
    name = "eval_harness",
    version,
    about = "Acceptance evaluation harness"
)]
pub(crate) struct Cli {
    #[command(subcommand)]
    pub(crate) command: Command,
}

#[derive(Debug, Subcommand)]
pub(crate) enum Command {
    /// Run acceptance evaluation for a profile or a single AT id.
    Run {
        /// Profile id, e.g. smoke_ready or full_matrix.
        #[arg(long)]
        profile: Option<String>,
        /// Single acceptance-test id, e.g. AT-004.
        #[arg(long)]
        at: Option<String>,
        /// Number of repeated attempts for reproducibility scoring.
        #[arg(long, default_value_t = 1)]
        repeat: u32,
    },
    /// Compile existing runs into aggregate reports.
    Compile {
        /// Input runs directory.
        #[arg(long)]
        input: PathBuf,
        /// Output reports directory.
        #[arg(long)]
        output: PathBuf,
    },
}
