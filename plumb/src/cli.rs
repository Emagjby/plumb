use clap::{Parser, Subcommand};

use crate::{
    commands::{
        add::plumb_add, diff::plumb_diff, go::plumb_go, rm::plumb_rm, start::plumb_start,
        status::plumb_status,
    },
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
        target: String,
    },

    /// Remove a file from the current session's queue.
    Rm {
        /// File path or item ID to remove.
        /// Example: "src/auth/guards.rs" or 3
        #[arg(verbatim_doc_comment)]
        target: String,
    },

    /// Prints the current session's queue of files to be refactored.
    Status {},

    /// Opens the specified file in the editor, captures baseline, and updates status to "In
    /// Progress".
    Go {
        /// File path or item ID of the file to refactor.
        /// Example: "src/auth/guards.rs"
        #[arg(verbatim_doc_comment)]
        target: String,
    },

    /// Compares the current version of the file with the baseline, and
    /// prints the diff to the terminal.
    Diff {
        /// File path or item ID of the file to diff.
        /// Omit to diff all items currently `In Progress`.
        /// Example: "src/auth/guards.rs"
        #[arg(verbatim_doc_comment)]
        target: Option<String>,
    },
}

pub fn run() -> Result<(), PlumbError> {
    let cli = Cli::parse();

    match cli.command {
        Commands::Start { name } => plumb_start(name)?,
        Commands::Add { folder, target } => plumb_add(target, folder)?,
        Commands::Status {} => plumb_status()?,
        Commands::Rm { target } => plumb_rm(target)?,
        Commands::Go { target } => plumb_go(target)?,
        Commands::Diff { target } => plumb_diff(target)?,
    }

    Ok(())
}
