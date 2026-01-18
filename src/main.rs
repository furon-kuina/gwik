mod cli;
mod commands;
mod config;
mod git;
mod scanner;
mod shell;

use clap::Parser;

use cli::{Cli, Command};

fn main() {
    let cli = Cli::parse();

    let result = match cli.command {
        Command::Open { branch, yes } => commands::open::run(&branch, yes),
        Command::Close { worktree, yes } => commands::close::run(worktree.as_deref(), yes),
        Command::List { all } => commands::list::run(all),
        Command::Cd { worktree } => commands::cd::run(&worktree),
        Command::Init { shell } => commands::init::run(&shell),
    };

    if let Err(e) = result {
        eprintln!("Error: {}", e);
        std::process::exit(1);
    }
}
