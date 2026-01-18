use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

use anyhow::Result;

/// Scan all configured root directories for Git repositories and their worktrees
/// Uses ghq-compatible structure: root/host/owner/repo
pub fn scan_all_worktrees(roots: &[PathBuf]) -> Result<Vec<PathBuf>> {
    let mut all_worktrees = Vec::new();

    for root in roots {
        if !root.exists() {
            continue;
        }

        // Scan at fixed depth of 3 (host/owner/repo)
        let repos = find_repos_at_depth(root, 3);

        for repo_path in repos {
            if let Ok(worktrees) = get_worktrees_for_repo(&repo_path) {
                all_worktrees.extend(worktrees);
            }
        }
    }

    Ok(all_worktrees)
}

/// Find Git repositories at a specific depth
fn find_repos_at_depth(root: &Path, depth: usize) -> Vec<PathBuf> {
    let mut repos = Vec::new();
    find_repos_recursive(root, depth, 0, &mut repos);
    repos
}

fn find_repos_recursive(
    path: &Path,
    target_depth: usize,
    current_depth: usize,
    repos: &mut Vec<PathBuf>,
) {
    if current_depth == target_depth {
        // Check if this is a git repository
        if path.join(".git").exists() {
            repos.push(path.to_path_buf());
        }
        return;
    }

    let Ok(entries) = fs::read_dir(path) else {
        return;
    };

    for entry in entries.flatten() {
        let entry_path = entry.path();
        if entry_path.is_dir() {
            // Skip hidden directories except at depth 0
            if let Some(name) = entry_path.file_name() {
                let name_str = name.to_string_lossy();
                if name_str.starts_with('.') {
                    continue;
                }
            }
            find_repos_recursive(&entry_path, target_depth, current_depth + 1, repos);
        }
    }
}

/// Get worktrees for a specific repository (excluding main worktree)
fn get_worktrees_for_repo(repo_path: &Path) -> Result<Vec<PathBuf>> {
    let output = Command::new("git")
        .current_dir(repo_path)
        .args(["worktree", "list", "--porcelain"])
        .output()?;

    if !output.status.success() {
        return Ok(Vec::new());
    }

    let stdout = String::from_utf8_lossy(&output.stdout);
    let mut worktrees = Vec::new();
    let mut current_path: Option<PathBuf> = None;
    let mut is_first = true;

    for line in stdout.lines() {
        if let Some(path_str) = line.strip_prefix("worktree ") {
            // Save previous worktree (skip main which is first)
            if let Some(path) = current_path.take() {
                if !is_first {
                    worktrees.push(path);
                }
                is_first = false;
            } else {
                is_first = false;
            }
            current_path = Some(PathBuf::from(path_str));
        }
    }

    // Don't forget the last worktree
    if let Some(path) = current_path {
        if !is_first {
            worktrees.push(path);
        }
    }

    Ok(worktrees)
}
