use clap::{Parser, Subcommand};

use crate::{
    commands::{
        add::plumb_add, diff::plumb_diff, done::plumb_done, finish::plumb_finish, go::plumb_go,
        next::plumb_next, restore::plumb_restore, rm::plumb_rm, start::plumb_start,
        status::plumb_status,
    },
    error::PlumbError,
    verbosity::set_verbose,
};

#[derive(Parser)]
#[command(
    name = "plumb",
    version,
    about = "Run refactor sessions as a disciplined file queue.",
    long_about = "A local CLI for running refactor sessions as a disciplined queue of files."
)]
pub struct Cli {
    /// Show detailed structured output and diagnostics.
    #[arg(short = 'v', long = "verbose", global = true)]
    pub verbose: bool,

    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand)]
pub enum Commands {
    /// Start a new refactor session.
    Start {
        /// Optional session label.
        /// Example: "refactor auth guards"
        ///
        /// Stored in session metadata only.
        #[arg(verbatim_doc_comment)]
        name: Option<String>,
    },

    /// Add a file or folder contents to the active session queue.
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

    /// Remove an item from the active session queue.
    Rm {
        /// File path or item ID to remove.
        /// Example: "src/auth/guards.rs" or 3
        #[arg(verbatim_doc_comment)]
        target: String,
    },

    /// Show queue counts and items for the active session.
    Status {},

    /// Start or reopen work on a queued item.
    Go {
        /// File path or item ID of the file to refactor.
        /// Example: "src/auth/guards.rs"
        #[arg(verbatim_doc_comment)]
        target: String,
    },

    /// Show baseline versus current diff for one item or all in-progress items.
    Diff {
        /// File path or item ID of the file to diff.
        /// Omit to diff all items currently `In Progress`.
        /// Example: "src/auth/guards.rs"
        #[arg(verbatim_doc_comment)]
        target: Option<String>,
    },

    /// Mark an in-progress item as done.
    Done {
        /// File path or item ID of the file to mark as done.
        /// Example: "src/auth/guards.rs"
        target: String,
    },

    /// Show the next todo item in queue order.
    Next {},

    /// Restore a file from its baseline snapshot.
    Restore {
        /// File path or item ID of the file to restore.
        /// Example: "src/auth/guards.rs"
        target: String,
    },

    /// Finish the active session.
    Finish {},
}

pub fn run() -> Result<(), PlumbError> {
    let cli = Cli::parse();
    set_verbose(cli.verbose);

    match cli.command {
        Commands::Start { name } => plumb_start(name)?,
        Commands::Add { folder, target } => plumb_add(target, folder)?,
        Commands::Status {} => plumb_status()?,
        Commands::Rm { target } => plumb_rm(target)?,
        Commands::Go { target } => plumb_go(target)?,
        Commands::Diff { target } => plumb_diff(target)?,
        Commands::Done { target } => plumb_done(target)?,
        Commands::Next {} => plumb_next()?,
        Commands::Restore { target } => plumb_restore(target)?,
        Commands::Finish {} => plumb_finish()?,
    }

    Ok(())
}
