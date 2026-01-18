# gwik - Git Worktree Manager Specification

## Overview

`gwik` is a CLI tool written in Rust that simplifies Git worktree management. It provides intuitive commands for creating, navigating, and managing worktrees across multiple repositories.

---

## Commands

### `gwik open <branch-name>`

Creates a new worktree with an associated branch.

**Behavior:**
- Creates a new branch from the current HEAD (follows `git worktree add` default)
- Creates worktree in `.worktrees/<branch-name>` (slashes replaced with hyphens)
- If a branch with the same name already exists:
  - Shows interactive confirmation prompt: "Branch 'X' already exists. Use existing branch? [y/N]"
  - `--yes` flag skips the confirmation and uses the existing branch
- Remote branches (e.g., `origin/feature-x`) are supported:
  - Automatically creates a local tracking branch
- After creation, outputs `cd` command for the user to copy
- Executes configured post-creation commands (see Configuration section)

**Options:**
- `--yes`: Skip confirmation prompts

**Example:**
```bash
$ gwik open feature-login
Created worktree at .worktrees/feature-login
Run: cd /path/to/repo/.worktrees/feature-login
```

---

### `gwik close [worktree-name]`

Removes a worktree. The associated branch is NOT deleted.

**Behavior:**
- If no argument provided, closes the current worktree (if inside one)
- If the current directory is inside the target worktree:
  - Shows confirmation prompt: "You are inside this worktree. Delete anyway? [y/N]"
  - After deletion, outputs `cd` command to main repository
- If there are uncommitted changes or untracked files:
  - Shows error and refuses to delete (safe default)
- After deletion, outputs `cd` command to main repository

**Example:**
```bash
$ gwik close feature-login
Removed worktree: feature-login
Run: cd /path/to/repo
```

---

### `gwik list`

Lists all worktrees in the current repository.

**Behavior:**
- Outputs only paths, one per line (suitable for piping to fzf/peco)
- Does not include the main worktree

**Example:**
```bash
$ gwik list
/path/to/repo/.worktrees/feature-login
/path/to/repo/.worktrees/bugfix-auth
```

---

### `gwik list --all`

Lists all worktrees across all registered repositories.

**Behavior:**
- Scans all root directories for Git repositories (ghq-compatible structure)
- Outputs all worktree paths flattened, one per line

**Example:**
```bash
$ gwik list --all
/home/user/dev/github.com/user/repo-a/.worktrees/feature-x
/home/user/dev/github.com/user/repo-b/.worktrees/bugfix-y
```

---

### `gwik cd <worktree-name>`

Outputs a `cd` command to navigate to the specified worktree.

**Behavior:**
- Requires exact match of worktree name (partial matching delegated to shell completion)
- If worktree not found, shows error with available worktrees

**Example:**
```bash
$ gwik cd feature-login
cd /path/to/repo/.worktrees/feature-login
```

---

### `gwik init <shell>`

Outputs shell integration code for the specified shell.

**Supported shells:** `bash`, `zsh`

**Behavior:**
- Outputs shell function that wraps `gwik` commands
- Enables automatic `cd` after `gwik open`, `gwik close`, `gwik cd`
- Generates shell completion scripts

**Usage:**
```bash
# Add to .bashrc or .zshrc
eval "$(gwik init bash)"
# or
eval "$(gwik init zsh)"
```

**Generated shell function behavior:**
- `gwik open`: Executes the output `cd` command automatically
- `gwik close`: Executes the output `cd` command automatically
- `gwik cd`: Executes the output `cd` command automatically

---

## Directory Structure

### Worktree Location

By default, worktrees are created under `.worktrees/` directory in the repository root. This can be customized via the `worktree_dir` setting.

**Default (`.worktrees/`):**
```
repo/
├── .git/
├── .worktrees/
│   ├── .gitignore      # Contains "*" to ignore all contents
│   ├── feature-login/  # worktree for feature/login branch
│   └── bugfix-auth/    # worktree for bugfix-auth branch
└── src/
```

**Alternative (`.git/.worktrees/`):**
```
repo/
├── .git/
│   ├── .worktrees/
│   │   ├── feature-login/
│   │   └── bugfix-auth/
│   └── ...
└── src/
```

### Branch Name to Directory Name Conversion

- Slashes (`/`) are replaced with hyphens (`-`)
- Example: `feature/login` → `<worktree_dir>/feature-login`

### .gitignore Handling

- If `worktree_dir` is inside `.git/` (e.g., `.git/.worktrees`): No `.gitignore` needed (already excluded from version control)
- Otherwise: On first worktree creation, `gwik` automatically creates `<worktree_dir>/.gitignore` with content `*`

---

## Configuration

Configuration uses TOML format.

### File Locations

- **Global:** `~/.config/gwik/config.toml` - Applied to all repositories
- **Local:** `.git/gwik.toml` - Applied to specific repository only (not version controlled)

**Merge behavior:**
- Local settings take precedence over global
- Some settings are scope-specific (see below)

### Settings

#### `worktree_dir`

Directory where worktrees are created, relative to the repository root.

```toml
worktree_dir = ".git/.worktrees"
```

- Default: `.worktrees`
- Common alternatives: `.git/.worktrees`, `.wt`
- If inside `.git/`, no `.gitignore` is created

#### `roots` (Global only)

Directories to scan for Git repositories when using `gwik list --all`.

```toml
roots = [
    "~/dev",
    "~/work",
]
```

- Array of directory paths
- Uses ghq-compatible directory structure (fixed depth: `host/owner/repo`)
- Example: `~/dev/github.com/user/repo`

#### `cmds` (Local only)

Commands to execute after worktree creation. Repository-specific setting.

```toml
cmds = [
    "cp $SRC/.env $DST/.env",
    "cd $DST && uv sync",
    "cd $DST && npm install",
]
```

- Array of commands (executed in order)
- Available variables:
  - `$SRC`: Path to the main repository
  - `$DST`: Path to the newly created worktree
- Commands are executed using the system shell
- **Error handling:** If any command fails, execution stops immediately. The worktree remains created, but subsequent commands are not executed.
- **Note:** This setting is only available in local configuration (`.git/gwik.toml`) because commands are repository-specific.

---

## Shell Completion

### Supported Shells

- **bash** (initial release)
- **zsh** (initial release)
- fish (future release)

### Completion Features

- Subcommand completion (`open`, `close`, `list`, `cd`, `init`)
- Worktree name completion for `gwik cd` and `gwik close`
- Branch name completion for `gwik open`

---

## Technical Implementation

### Language

Rust

### Git Operations

Uses `git` CLI commands via `std::process::Command`. Requires `git` to be installed.

Key commands used:
- `git worktree add` - Create worktree
- `git worktree remove` - Remove worktree
- `git worktree list` - List worktrees
- `git rev-parse --git-common-dir` - Find main repository from worktree
- `git branch` - Branch operations

### Main Repository Detection

Uses `git rev-parse --git-common-dir` to determine the main repository when running `gwik` from within a worktree.

### Repository Scanning

For `gwik list --all`, scans root directories with ghq-compatible fixed depth structure:
```
root/
└── github.com/
    └── owner/
        └── repo/  <- Git repository detected here
```

---

## User Interface

### Language

English only (all messages, errors, prompts)

### Output Style

- Minimal output for scriptability
- `cd` commands are output for user to execute (or auto-executed via shell function)
- Errors go to stderr

### Interactive Prompts

- Confirmation prompts use `[y/N]` format (default: No)
- `--yes` flag available to skip confirmations for scripting

---

## Error Handling

### Worktree Creation Errors

| Condition | Behavior |
|-----------|----------|
| Branch name invalid | Error with message |
| Worktree already exists | Error with message |
| `.worktrees` directory not writable | Error with message |
| Post-creation command fails | Error, stop execution, worktree remains |

### Worktree Deletion Errors

| Condition | Behavior |
|-----------|----------|
| Worktree not found | Error with available worktrees list |
| Uncommitted changes exist | Error, refuse to delete |
| Currently inside target worktree | Confirmation prompt |

---

## Example Workflow

```bash
# Initial setup
eval "$(gwik init zsh)"

# Create a new feature branch and worktree
gwik open feature/new-api
# -> Creates .worktrees/feature-new-api
# -> Automatically cd into the worktree

# Work on the feature...
git add . && git commit -m "Implement new API"

# Switch to another worktree
gwik cd bugfix-auth
# -> Automatically cd to .worktrees/bugfix-auth

# List all worktrees
gwik list
# -> /path/to/repo/.worktrees/feature-new-api
# -> /path/to/repo/.worktrees/bugfix-auth

# Clean up after merging
gwik close feature-new-api
# -> Removes worktree (branch remains)
# -> Automatically cd to main repo
```

---

## Configuration Example

### Global Configuration (`~/.config/gwik/config.toml`)

```toml
# Directories to scan for repositories
roots = [
    "~/dev",
    "~/work",
]

# Default worktree directory (optional)
worktree_dir = ".git/.worktrees"
```

### Local Configuration (`.git/gwik.toml`)

```toml
# Worktree directory for this repository (optional)
worktree_dir = ".worktrees"

# Commands to run after worktree creation
cmds = [
    "cp $SRC/.env $DST/.env",
    "cd $DST && uv sync",
    "cd $DST && npm install",
]
```

---

## Future Considerations (Out of Scope for Initial Release)

- fish shell support
- `gwik status` command to show status of all worktrees
- fzf/peco integration with preview
- Verbose/debug logging option
