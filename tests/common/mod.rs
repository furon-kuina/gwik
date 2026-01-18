#![allow(dead_code)]

use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, Output};

use tempfile::TempDir;

/// Test fixture that provides a temporary git repository
pub struct TestRepo {
    pub temp_dir: TempDir,
    pub root: PathBuf,
}

impl TestRepo {
    /// Create a new test repository with initial commit
    pub fn new() -> Self {
        let temp_dir = TempDir::new().expect("Failed to create temp dir");
        let root = temp_dir.path().to_path_buf();

        // Initialize git repository
        run_git(&root, &["init"]).expect("git init failed");

        // Configure git user for commits
        run_git(&root, &["config", "user.email", "test@test.com"])
            .expect("git config email failed");
        run_git(&root, &["config", "user.name", "Test User"]).expect("git config name failed");

        // Create initial commit
        let readme = root.join("README.md");
        fs::write(&readme, "# Test Repository\n").expect("Failed to write README");
        run_git(&root, &["add", "."]).expect("git add failed");
        run_git(&root, &["commit", "-m", "Initial commit"]).expect("git commit failed");

        Self { temp_dir, root }
    }

    /// Get the gwik binary path
    pub fn gwik_bin() -> PathBuf {
        // Use cargo to find the binary
        let output = Command::new("cargo")
            .args(["build", "--quiet"])
            .current_dir(env!("CARGO_MANIFEST_DIR"))
            .output()
            .expect("Failed to build gwik");

        if !output.status.success() {
            panic!(
                "Failed to build gwik: {}",
                String::from_utf8_lossy(&output.stderr)
            );
        }

        PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("target")
            .join("debug")
            .join("gwik")
    }

    /// Run gwik command in this repository
    pub fn gwik(&self, args: &[&str]) -> Output {
        Command::new(Self::gwik_bin())
            .args(args)
            .current_dir(&self.root)
            .output()
            .expect("Failed to run gwik")
    }

    /// Run gwik command in a specific directory
    pub fn gwik_in(&self, dir: &Path, args: &[&str]) -> Output {
        Command::new(Self::gwik_bin())
            .args(args)
            .current_dir(dir)
            .output()
            .expect("Failed to run gwik")
    }

    /// Run git command in this repository
    pub fn git(&self, args: &[&str]) -> Output {
        run_git(&self.root, args).expect("git command failed")
    }

    /// Create a branch at current HEAD
    pub fn create_branch(&self, name: &str) {
        self.git(&["branch", name]);
    }

    /// Create a remote and push a branch to it (without leaving local branch)
    pub fn create_remote_branch(&self, remote: &str, branch: &str) {
        // Create a bare remote repository
        let remote_dir = self.temp_dir.path().join("remote");
        fs::create_dir_all(&remote_dir).expect("Failed to create remote dir");
        run_git(&remote_dir, &["init", "--bare"]).expect("git init --bare failed");

        // Add remote
        self.git(&["remote", "add", remote, remote_dir.to_str().unwrap()]);

        // Create, push, and delete the local branch
        self.git(&["branch", branch]);
        self.git(&["push", remote, branch]);
        self.git(&["branch", "-D", branch]);

        // Fetch to update remote tracking refs
        self.git(&["fetch", remote]);
    }

    /// Get path to a worktree
    pub fn worktree_path(&self, name: &str) -> PathBuf {
        self.root.join(".worktrees").join(name)
    }

    /// Check if worktree exists
    pub fn worktree_exists(&self, name: &str) -> bool {
        self.worktree_path(name).exists()
    }

    /// Create a file in a worktree
    pub fn create_file_in_worktree(&self, worktree: &str, file: &str, content: &str) {
        let path = self.worktree_path(worktree).join(file);
        fs::write(&path, content).expect("Failed to write file");
    }

    /// Stage a file in a worktree
    pub fn stage_file_in_worktree(&self, worktree: &str, file: &str) {
        let worktree_path = self.worktree_path(worktree);
        run_git(&worktree_path, &["add", file]).expect("git add failed");
    }

    /// Write local gwik config
    pub fn write_local_config(&self, content: &str) {
        let config_path = self.root.join(".git").join("gwik.toml");
        fs::write(&config_path, content).expect("Failed to write local config");
    }
}

fn run_git(dir: &Path, args: &[&str]) -> Result<Output, std::io::Error> {
    Command::new("git").args(args).current_dir(dir).output()
}

/// Helper to get stdout as string
pub fn stdout(output: &Output) -> String {
    String::from_utf8_lossy(&output.stdout).to_string()
}

/// Helper to get stderr as string
pub fn stderr(output: &Output) -> String {
    String::from_utf8_lossy(&output.stderr).to_string()
}
