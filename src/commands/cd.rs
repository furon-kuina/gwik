use anyhow::{bail, Result};

use crate::config::Config;
use crate::git::GitRepo;

pub fn run(worktree: &str) -> Result<()> {
    let repo = GitRepo::detect()?;
    let config = Config::load(&repo.git_dir)?;
    let worktrees = repo.list_worktrees()?;

    // Try to find exact match by directory name
    let worktree_dir = repo.worktree_dir(&config.worktree_dir);
    let target_path = worktree_dir.join(worktree);

    // Check if this worktree exists
    for wt in &worktrees {
        if wt.path == target_path {
            println!("cd {}", wt.path.display());
            return Ok(());
        }
    }

    // Not found - show available worktrees
    eprintln!("Error: Worktree '{}' not found", worktree);
    if !worktrees.is_empty() {
        eprintln!("\nAvailable worktrees:");
        for wt in &worktrees {
            if let Some(name) = wt.path.file_name() {
                eprintln!("  {}", name.to_string_lossy());
            }
        }
    }

    bail!("Worktree '{}' not found", worktree);
}
