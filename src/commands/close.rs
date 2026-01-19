use std::io::{self, Write};

use anyhow::{bail, Result};

use crate::config::Config;
use crate::git::GitRepo;

pub fn run(worktree: Option<&str>, yes: bool) -> Result<()> {
    let repo = GitRepo::detect()?;
    let config = Config::load(&repo.git_dir)?;
    let worktrees = repo.list_worktrees()?;

    let cwd = std::env::current_dir()?;

    // Determine target worktree
    let target_path = if let Some(name) = worktree {
        // Specified worktree name
        let worktree_dir = repo.worktree_dir(&config.worktree_dir);
        let path = worktree_dir.join(name);

        // Verify it exists
        let exists = worktrees.iter().any(|wt| wt.path == path);
        if !exists {
            eprintln!("Error: Worktree '{}' not found", name);
            if !worktrees.is_empty() {
                eprintln!("\nAvailable worktrees:");
                for wt in &worktrees {
                    if let Some(name) = wt.path.file_name() {
                        eprintln!("  {}", name.to_string_lossy());
                    }
                }
            }
            bail!("Worktree '{}' not found", name);
        }
        path
    } else {
        // No argument - try to use current worktree
        match repo.current_worktree()? {
            Some(path) => path,
            None => {
                bail!("Not inside a worktree. Specify a worktree name to close.");
            }
        }
    };

    // Check if we're inside the target worktree
    let inside_target = cwd.starts_with(&target_path);

    if inside_target && !yes {
        eprint!("You are inside this worktree. Delete anyway? [y/N] ");
        io::stderr().flush()?;

        let mut input = String::new();
        io::stdin().read_line(&mut input)?;
        let input = input.trim().to_lowercase();

        if input != "y" && input != "yes" {
            eprintln!("Aborted.");
            return Ok(());
        }
    }

    // Remove the worktree
    repo.remove_worktree(&target_path)?;

    let worktree_name = target_path
        .file_name()
        .map(|n| n.to_string_lossy().to_string())
        .unwrap_or_default();

    eprintln!("Removed worktree: {}", worktree_name);

    // Output cd command to main repository (or source_worktree for bare repos)
    let target_dir = repo.working_dir(config.source_worktree.as_deref());
    println!("cd {}", target_dir.display());

    Ok(())
}
