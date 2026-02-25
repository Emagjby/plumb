use clap::{Parser, Subcommand};

use crate::{
    commands::{add::plumb_add, rm::plumb_rm, start::plumb_start, status::plumb_status},
    error::PlumbError,
};

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

    /// Add a file (or a folder with -f --folder) to the current session's queue.
    Add {
        /// Treat the given path as a folder,
        /// and enqueue all files recursively.
        #[arg(short = 'f', long = "folder", verbatim_doc_comment)]
        folder: bool,

        /// Path to the file to add.
        /// Example: "src/auth/guards.rs"
        #[arg(verbatim_doc_comment)]
        file: String,
    },

    /// Remove a file from the current session's queue.
    Rm {
        /// File path or item ID to remove.
        /// Example: "src/auth/guards.rs" or 3
        #[arg(verbatim_doc_comment)]
        file: String,
    },

    /// Prints the current session's queue of files to be refactored.
    Status {},
}

pub fn run() -> Result<(), PlumbError> {
    let cli = Cli::parse();

    match cli.command {
        Commands::Start { name } => plumb_start(name)?,
        Commands::Add { folder, file } => plumb_add(file, folder)?,
        Commands::Status {} => plumb_status()?,
        Commands::Rm { file } => plumb_rm(file)?,
    }

    Ok(())
}
