use clap::{Parser, Subcommand};

use crate::{commands::start::plumb_start, error::PlumbError};

#[derive(Parser)]
#[command(
    name = "plumb",
    version,
    about = "plumb CLI",
    long_about = "A CLI for running refactor sessions as a disciplined queue of files."
)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand)]
pub enum Commands {
    /// Begin a new refactor session.
    Start {
        /// Optional session label.
        /// Example: "refactor auth guards"
        ///
        /// Stored in session metadata only.
        #[arg(verbatim_doc_comment)]
        name: Option<String>,
    },
}

pub fn run() -> Result<(), PlumbError> {
    let cli = Cli::parse();

    match cli.command {
        Commands::Start { name } => plumb_start(name)?,
    }

    Ok(())
}
