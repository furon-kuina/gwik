use std::fs;
use std::io::{self, Write};
use std::process::Command;

use anyhow::{bail, Result};

use crate::config::Config;
use crate::git::GitRepo;

pub fn run(branch: &str, yes: bool) -> Result<()> {
    let repo = GitRepo::detect()?;
    let config = Config::load(&repo.git_dir)?;

    let dirname = GitRepo::branch_to_dirname(branch);
    let worktree_dir = repo.worktree_dir(&config.worktree_dir);
    let worktree_path = worktree_dir.join(&dirname);

    // Check if worktree already exists
    if worktree_path.exists() {
        bail!("Worktree '{}' already exists", dirname);
    }

    // Ensure worktree directory exists
    if !worktree_dir.exists() {
        fs::create_dir_all(&worktree_dir)?;

        // Create .gitignore if not inside .git
        if !config.worktree_dir_in_git() {
            let gitignore_path = worktree_dir.join(".gitignore");
            fs::write(&gitignore_path, "*\n")?;
        }
    }

    // Check if it's a remote branch (e.g., origin/feature-x)
    if let Some(remote_branch) = repo.remote_branch_exists(branch)? {
        // Extract local branch name from remote (origin/feature-x -> feature-x)
        let local_branch = remote_branch
            .split('/')
            .skip(1)
            .collect::<Vec<_>>()
            .join("/");

        repo.create_worktree_tracking(&worktree_path, &local_branch, &remote_branch)?;
        eprintln!(
            "Created worktree at {} tracking {}",
            worktree_path.display(),
            remote_branch
        );
    } else if repo.branch_exists(branch)? {
        // Branch exists locally
        if !yes {
            eprint!(
                "Branch '{}' already exists. Use existing branch? [y/N] ",
                branch
            );
            io::stderr().flush()?;

            let mut input = String::new();
            io::stdin().read_line(&mut input)?;
            let input = input.trim().to_lowercase();

            if input != "y" && input != "yes" {
                eprintln!("Aborted.");
                return Ok(());
            }
        }

        repo.create_worktree_existing_branch(&worktree_path, branch)?;
        eprintln!("Created worktree at {}", worktree_path.display());
    } else {
        // Create new branch
        repo.create_worktree(&worktree_path, branch)?;
        eprintln!("Created worktree at {}", worktree_path.display());
    }

    // Run post-creation commands
    if !config.cmds.is_empty() {
        run_post_commands(&config.cmds, &repo.root, &worktree_path)?;
    }

    // Output cd command
    println!("cd {}", worktree_path.display());

    Ok(())
}

fn run_post_commands(cmds: &[String], src: &std::path::Path, dst: &std::path::Path) -> Result<()> {
    let src_str = src.to_string_lossy();
    let dst_str = dst.to_string_lossy();

    for cmd in cmds {
        let expanded = cmd.replace("$SRC", &src_str).replace("$DST", &dst_str);

        eprintln!("Running: {}", expanded);

        let status = Command::new("sh").arg("-c").arg(&expanded).status()?;

        if !status.success() {
            bail!("Post-creation command failed: {}", expanded);
        }
    }

    Ok(())
}
