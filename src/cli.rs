use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(name = "gwik")]
#[command(about = "Git worktree manager", long_about = None)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Command,
}

#[derive(Subcommand)]
pub enum Command {
    /// Create a new worktree with an associated branch
    Open {
        /// Branch name to create or use
        branch: String,

        /// Skip confirmation prompts
        #[arg(short, long)]
        yes: bool,
    },

    /// Remove a worktree (branch is kept)
    Close {
        /// Worktree name to close (defaults to current if inside a worktree)
        worktree: Option<String>,

        /// Skip confirmation prompts
        #[arg(short, long)]
        yes: bool,
    },

    /// List worktrees
    List {
        /// List worktrees across all registered repositories
        #[arg(long)]
        all: bool,
    },

    /// Output cd command to navigate to a worktree
    Cd {
        /// Worktree name to navigate to
        worktree: String,
    },

    /// Output shell integration code
    Init {
        /// Shell to generate integration for (bash, zsh)
        shell: String,
    },
}
