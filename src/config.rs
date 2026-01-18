use std::fs;
use std::path::{Path, PathBuf};

use anyhow::Result;
use serde::Deserialize;

const DEFAULT_WORKTREE_DIR: &str = ".worktrees";

#[derive(Debug, Deserialize, Default)]
pub struct GlobalConfig {
    pub worktree_dir: Option<String>,
    #[serde(default)]
    pub roots: Vec<String>,
}

#[derive(Debug, Deserialize, Default)]
pub struct LocalConfig {
    pub worktree_dir: Option<String>,
    #[serde(default)]
    pub cmds: Vec<String>,
}

#[derive(Debug)]
pub struct Config {
    pub worktree_dir: String,
    pub roots: Vec<PathBuf>,
    pub cmds: Vec<String>,
}

impl Config {
    /// Load and merge global and local configurations
    pub fn load(git_dir: &Path) -> Result<Self> {
        let global = load_global_config();
        let local = load_local_config(git_dir);

        // Local takes precedence over global, defaults as fallback
        let worktree_dir = local
            .worktree_dir
            .or(global.worktree_dir)
            .unwrap_or_else(|| DEFAULT_WORKTREE_DIR.to_string());

        // Expand ~ in root paths
        let roots = global
            .roots
            .into_iter()
            .filter_map(|r| expand_tilde(&r))
            .collect();

        // cmds is local only
        let cmds = local.cmds;

        Ok(Config {
            worktree_dir,
            roots,
            cmds,
        })
    }

    /// Check if worktree_dir is inside .git (no .gitignore needed)
    pub fn worktree_dir_in_git(&self) -> bool {
        self.worktree_dir.starts_with(".git/") || self.worktree_dir.starts_with(".git\\")
    }
}

fn global_config_path() -> Option<PathBuf> {
    dirs::config_dir().map(|p| p.join("gwik").join("config.toml"))
}

fn load_global_config() -> GlobalConfig {
    let Some(path) = global_config_path() else {
        return GlobalConfig::default();
    };

    if !path.exists() {
        return GlobalConfig::default();
    }

    match fs::read_to_string(&path) {
        Ok(content) => toml::from_str(&content).unwrap_or_default(),
        Err(_) => GlobalConfig::default(),
    }
}

fn load_local_config(git_dir: &Path) -> LocalConfig {
    let path = git_dir.join("gwik.toml");

    if !path.exists() {
        return LocalConfig::default();
    }

    match fs::read_to_string(&path) {
        Ok(content) => toml::from_str(&content).unwrap_or_default(),
        Err(_) => LocalConfig::default(),
    }
}

fn expand_tilde(path: &str) -> Option<PathBuf> {
    if path.starts_with("~/") {
        dirs::home_dir().map(|home| home.join(&path[2..]))
    } else if path == "~" {
        dirs::home_dir()
    } else {
        Some(PathBuf::from(path))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_worktree_dir_in_git() {
        let config = Config {
            worktree_dir: ".git/.worktrees".to_string(),
            roots: vec![],
            cmds: vec![],
        };
        assert!(config.worktree_dir_in_git());

        let config2 = Config {
            worktree_dir: ".worktrees".to_string(),
            roots: vec![],
            cmds: vec![],
        };
        assert!(!config2.worktree_dir_in_git());
    }
}
