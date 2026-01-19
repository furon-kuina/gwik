use std::path::{Path, PathBuf};
use std::process::Command;

use anyhow::{bail, Context, Result};

pub struct GitRepo {
    pub root: PathBuf,
    pub git_dir: PathBuf,
    pub is_bare: bool,
}

impl GitRepo {
    /// Detect the git repository from the current directory
    pub fn detect() -> Result<Self> {
        // Check if this is a bare repository
        let is_bare_output = Command::new("git")
            .args(["rev-parse", "--is-bare-repository"])
            .output()
            .context("Failed to run git")?;

        if !is_bare_output.status.success() {
            bail!("Not in a git repository");
        }

        let is_bare = String::from_utf8_lossy(&is_bare_output.stdout)
            .trim()
            .eq("true");

        // Get the common git directory (works from both main repo and worktrees)
        let output = Command::new("git")
            .args(["rev-parse", "--git-common-dir"])
            .output()
            .context("Failed to run git")?;

        if !output.status.success() {
            bail!("Not in a git repository");
        }

        let git_common_dir = String::from_utf8_lossy(&output.stdout).trim().to_string();
        let git_dir = PathBuf::from(&git_common_dir)
            .canonicalize()
            .context("Failed to resolve git directory")?;

        // In a bare repo, the git directory IS the root
        // In a regular repo, the root is the parent of the .git directory
        let root = if is_bare {
            git_dir.clone()
        } else {
            git_dir
                .parent()
                .context("Invalid git directory structure")?
                .to_path_buf()
        };

        Ok(Self {
            root,
            git_dir,
            is_bare,
        })
    }

    /// Get worktree directory path based on config
    pub fn worktree_dir(&self, worktree_dir_name: &str) -> PathBuf {
        self.root.join(worktree_dir_name)
    }

    /// Get the working directory for running git commands
    /// For bare repos with source_worktree configured, returns that path
    /// Otherwise returns the repo root
    pub fn working_dir(&self, source_worktree: Option<&str>) -> PathBuf {
        if self.is_bare {
            if let Some(source) = source_worktree {
                return self.root.join(source);
            }
        }
        self.root.clone()
    }

    /// Convert branch name to directory name (slashes to hyphens)
    pub fn branch_to_dirname(branch: &str) -> String {
        branch.replace('/', "-")
    }

    /// List all worktrees (excluding main)
    pub fn list_worktrees(&self) -> Result<Vec<WorktreeInfo>> {
        let output = Command::new("git")
            .current_dir(&self.root)
            .args(["worktree", "list", "--porcelain"])
            .output()
            .context("Failed to run git worktree list")?;

        if !output.status.success() {
            bail!(
                "git worktree list failed: {}",
                String::from_utf8_lossy(&output.stderr)
            );
        }

        let stdout = String::from_utf8_lossy(&output.stdout);
        let worktrees = parse_worktree_list(&stdout, &self.root);
        Ok(worktrees)
    }

    /// Check if a branch exists locally
    pub fn branch_exists(&self, branch: &str) -> Result<bool> {
        let status = Command::new("git")
            .current_dir(&self.root)
            .args([
                "show-ref",
                "--verify",
                "--quiet",
                &format!("refs/heads/{}", branch),
            ])
            .status()
            .context("Failed to run git show-ref")?;

        Ok(status.success())
    }

    /// Check if a remote branch exists
    pub fn remote_branch_exists(&self, branch: &str) -> Result<Option<String>> {
        // Check if it's in the format "origin/branch"
        if branch.contains('/') {
            let status = Command::new("git")
                .current_dir(&self.root)
                .args([
                    "show-ref",
                    "--verify",
                    "--quiet",
                    &format!("refs/remotes/{}", branch),
                ])
                .status()
                .context("Failed to run git show-ref")?;

            if status.success() {
                return Ok(Some(branch.to_string()));
            }
        }
        Ok(None)
    }

    /// Create a worktree with a new branch
    pub fn create_worktree(&self, path: &Path, branch: &str) -> Result<()> {
        let output = Command::new("git")
            .current_dir(&self.root)
            .args(["worktree", "add", "-b", branch, path.to_str().unwrap()])
            .output()
            .context("Failed to run git worktree add")?;

        if !output.status.success() {
            bail!(
                "git worktree add failed: {}",
                String::from_utf8_lossy(&output.stderr)
            );
        }

        Ok(())
    }

    /// Create a worktree with an existing branch
    pub fn create_worktree_existing_branch(&self, path: &Path, branch: &str) -> Result<()> {
        let output = Command::new("git")
            .current_dir(&self.root)
            .args(["worktree", "add", path.to_str().unwrap(), branch])
            .output()
            .context("Failed to run git worktree add")?;

        if !output.status.success() {
            bail!(
                "git worktree add failed: {}",
                String::from_utf8_lossy(&output.stderr)
            );
        }

        Ok(())
    }

    /// Create a worktree tracking a remote branch
    pub fn create_worktree_tracking(
        &self,
        path: &Path,
        local_branch: &str,
        remote_branch: &str,
    ) -> Result<()> {
        let output = Command::new("git")
            .current_dir(&self.root)
            .args([
                "worktree",
                "add",
                "-b",
                local_branch,
                path.to_str().unwrap(),
                remote_branch,
            ])
            .output()
            .context("Failed to run git worktree add")?;

        if !output.status.success() {
            bail!(
                "git worktree add failed: {}",
                String::from_utf8_lossy(&output.stderr)
            );
        }

        Ok(())
    }

    /// Remove a worktree
    pub fn remove_worktree(&self, path: &Path) -> Result<()> {
        let output = Command::new("git")
            .current_dir(&self.root)
            .args(["worktree", "remove", path.to_str().unwrap()])
            .output()
            .context("Failed to run git worktree remove")?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            if stderr.contains("contains modified or untracked files") {
                bail!("Cannot remove worktree: uncommitted changes exist");
            }
            bail!("git worktree remove failed: {}", stderr);
        }

        Ok(())
    }

    /// Check if the current directory is inside a worktree
    pub fn current_worktree(&self) -> Result<Option<PathBuf>> {
        let cwd = std::env::current_dir()?;
        let worktrees = self.list_worktrees()?;

        for wt in worktrees {
            if cwd.starts_with(&wt.path) {
                return Ok(Some(wt.path));
            }
        }

        Ok(None)
    }
}

#[derive(Debug, Clone)]
pub struct WorktreeInfo {
    pub path: PathBuf,
}

fn parse_worktree_list(output: &str, main_root: &Path) -> Vec<WorktreeInfo> {
    let mut worktrees = Vec::new();
    let mut current_path: Option<PathBuf> = None;

    for line in output.lines() {
        if let Some(path_str) = line.strip_prefix("worktree ") {
            // Save previous worktree if exists
            if let Some(path) = current_path.take() {
                worktrees.push(WorktreeInfo { path });
            }
            current_path = Some(PathBuf::from(path_str));
        }
    }

    // Don't forget the last worktree
    if let Some(path) = current_path {
        worktrees.push(WorktreeInfo { path });
    }

    // Filter out the main worktree
    worktrees
        .into_iter()
        .filter(|wt| wt.path != main_root)
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_branch_to_dirname() {
        assert_eq!(GitRepo::branch_to_dirname("feature/login"), "feature-login");
        assert_eq!(GitRepo::branch_to_dirname("bugfix-auth"), "bugfix-auth");
        assert_eq!(GitRepo::branch_to_dirname("a/b/c"), "a-b-c");
    }
}
