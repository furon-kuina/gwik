# gwik

A CLI tool for simplified Git worktree management.

## Installation

### From crates.io

```bash
cargo install gwik
```

### From prebuilt binaries

```bash
cargo binstall gwik
```

Or download from [GitHub Releases](https://github.com/furon-kuina/gwik/releases).

### From source

```bash
git clone https://github.com/furon-kuina/gwik.git
cd gwik
cargo install --path .
```

## Quick Start

```bash
# Set up shell integration (add to .bashrc or .zshrc)
eval "$(gwik init bash)"  # or zsh

# Create a new worktree for a feature branch
gwik open feature/new-api
# -> Creates .worktrees/feature-new-api and cd into it

# List all worktrees
gwik list

# Switch to another worktree
gwik cd bugfix-auth

# Clean up when done
gwik close feature-new-api
```

## Commands

### `gwik open <branch-name>`

Creates a new worktree with an associated branch.

```bash
gwik open feature/login
# Created worktree at .worktrees/feature-login
# (automatically cd into it with shell integration)
```

- Creates a new branch from current HEAD
- Branch names with `/` are converted to `-` for the directory name
- Use `--yes` to skip confirmation when using an existing branch

**Remote branches:**

```bash
gwik open origin/feature-x
# Creates local tracking branch automatically
```

### `gwik close [worktree-name]`

Removes a worktree (branch is kept).

```bash
gwik close feature-login
# Removed worktree: feature-login
# (automatically cd to main repo with shell integration)
```

- Without arguments, closes the current worktree
- Refuses to delete if there are uncommitted changes
- Use `--yes` to skip confirmation when inside the target worktree

### `gwik list`

Lists all worktrees in the current repository.

```bash
gwik list
# /path/to/repo/.worktrees/feature-login
# /path/to/repo/.worktrees/bugfix-auth
```

Output is one path per line, suitable for piping to `fzf` or `peco`.

**List across all repositories:**

```bash
gwik list --all
```

### `gwik cd <worktree-name>`

Outputs a `cd` command to navigate to the worktree.

```bash
gwik cd feature-login
# cd /path/to/repo/.worktrees/feature-login
```

With shell integration, this automatically changes directory.

### `gwik init <shell>`

Outputs shell integration code.

```bash
# Add to your .bashrc or .zshrc
eval "$(gwik init bash)"
eval "$(gwik init zsh)"
```

Shell integration enables:
- Automatic `cd` after `gwik open`, `gwik close`, `gwik cd`
- Tab completion for commands and worktree names

## Configuration

### Global Configuration

`~/.config/gwik/config.toml`

```toml
# Directories to scan for `gwik list --all`
roots = [
    "~/dev",
    "~/work",
]

# Default worktree directory (optional)
worktree_dir = ".worktrees"
```

### Local Configuration

`.git/gwik.toml` (per-repository)

```toml
# Override worktree directory for this repo
worktree_dir = ".git/.worktrees"

# Commands to run after creating a worktree
cmds = [
    "cp $SRC/.env $DST/.env",
    "cd $DST && npm install",
]
```

**Post-creation commands:**
- `$SRC` - path to the main repository
- `$DST` - path to the newly created worktree
- Commands run in order; execution stops on first failure

## Directory Structure

By default, worktrees are created under `.worktrees/`:

```
repo/
├── .git/
├── .worktrees/
│   ├── .gitignore      # Auto-created, contains "*"
│   ├── feature-login/
│   └── bugfix-auth/
└── src/
```

Set `worktree_dir = ".git/.worktrees"` to keep worktrees inside `.git/` (no `.gitignore` needed).

## License

MIT
