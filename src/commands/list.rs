use anyhow::Result;

use crate::config::Config;
use crate::git::GitRepo;
use crate::scanner::scan_all_worktrees;

pub fn run(all: bool) -> Result<()> {
    if all {
        run_all()
    } else {
        run_current_repo()
    }
}

fn run_current_repo() -> Result<()> {
    let repo = GitRepo::detect()?;
    let worktrees = repo.list_worktrees()?;

    for wt in worktrees {
        println!("{}", wt.path.display());
    }

    Ok(())
}

fn run_all() -> Result<()> {
    let repo = GitRepo::detect()?;
    let config = Config::load(&repo.git_dir)?;

    let worktrees = scan_all_worktrees(&config.roots)?;

    for path in worktrees {
        println!("{}", path.display());
    }

    Ok(())
}
