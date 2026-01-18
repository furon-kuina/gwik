mod common;

use common::{stderr, stdout, TestRepo};

/// gwik open <branch-name>: Creates a new worktree with an associated branch
/// Spec: Creates a new branch from the current HEAD
#[test]
fn test_open_creates_new_worktree_and_branch() {
    let repo = TestRepo::new();

    let output = repo.gwik(&["open", "feature-test"]);

    assert!(
        output.status.success(),
        "gwik open failed: {}",
        stderr(&output)
    );
    assert!(
        repo.worktree_exists("feature-test"),
        "Worktree directory should exist"
    );

    // Verify cd command is output
    let out = stdout(&output);
    assert!(out.contains("cd "), "Should output cd command");
    assert!(
        out.contains("feature-test"),
        "cd command should point to worktree"
    );

    // Verify branch was created
    let git_output = repo.git(&["branch", "--list", "feature-test"]);
    let branches = stdout(&git_output);
    assert!(
        branches.contains("feature-test"),
        "Branch should be created"
    );
}

/// Spec: Slashes in branch names are replaced with hyphens for directory name
#[test]
fn test_open_converts_slashes_to_hyphens() {
    let repo = TestRepo::new();

    let output = repo.gwik(&["open", "feature/login"]);

    assert!(
        output.status.success(),
        "gwik open failed: {}",
        stderr(&output)
    );
    assert!(
        repo.worktree_exists("feature-login"),
        "Worktree should use hyphens instead of slashes"
    );

    // Verify branch has original name with slash
    let git_output = repo.git(&["branch", "--list", "feature/login"]);
    let branches = stdout(&git_output);
    assert!(
        branches.contains("feature/login"),
        "Branch should have original name with slash"
    );
}

/// Spec: If a branch with the same name already exists, use --yes to skip confirmation
#[test]
fn test_open_existing_branch_with_yes_flag() {
    let repo = TestRepo::new();

    // Create branch first
    repo.create_branch("existing-branch");

    // Open with --yes should succeed
    let output = repo.gwik(&["open", "--yes", "existing-branch"]);

    assert!(
        output.status.success(),
        "gwik open --yes should succeed: {}",
        stderr(&output)
    );
    assert!(
        repo.worktree_exists("existing-branch"),
        "Worktree should be created"
    );
}

/// Spec: Worktrees are created under .worktrees/ directory
#[test]
fn test_open_creates_worktree_in_correct_directory() {
    let repo = TestRepo::new();

    let output = repo.gwik(&["open", "my-feature"]);

    assert!(output.status.success());

    let expected_path = repo.root.join(".worktrees").join("my-feature");
    assert!(
        expected_path.exists(),
        "Worktree should be at .worktrees/my-feature"
    );
    assert!(
        expected_path.join(".git").exists(),
        "Worktree should have .git file"
    );
}

/// Spec: On first worktree creation, gwik creates .worktrees/.gitignore with content "*"
#[test]
fn test_open_creates_gitignore_in_worktrees_dir() {
    let repo = TestRepo::new();

    let output = repo.gwik(&["open", "feature-a"]);

    assert!(output.status.success());

    let gitignore_path = repo.root.join(".worktrees").join(".gitignore");
    assert!(
        gitignore_path.exists(),
        ".gitignore should be created in .worktrees"
    );

    let content = std::fs::read_to_string(&gitignore_path).unwrap();
    assert!(content.trim() == "*", ".gitignore should contain '*'");
}

/// Spec: Error when worktree already exists
#[test]
fn test_open_fails_when_worktree_already_exists() {
    let repo = TestRepo::new();

    // Create first worktree
    let output1 = repo.gwik(&["open", "duplicate"]);
    assert!(output1.status.success());

    // Try to create again
    let output2 = repo.gwik(&["open", "duplicate"]);
    assert!(
        !output2.status.success(),
        "Should fail when worktree exists"
    );

    let err = stderr(&output2);
    assert!(
        err.contains("already exists"),
        "Error should mention worktree exists"
    );
}

/// Spec: Remote branches are supported - creates local tracking branch
#[test]
fn test_open_remote_branch_creates_tracking_branch() {
    let repo = TestRepo::new();

    // Create a remote branch
    repo.create_remote_branch("origin", "remote-feature");

    // Open with remote branch
    let output = repo.gwik(&["open", "origin/remote-feature"]);

    assert!(
        output.status.success(),
        "Should succeed with remote branch: {}",
        stderr(&output)
    );

    // The worktree should exist with hyphenated name
    assert!(
        repo.worktree_exists("origin-remote-feature") || repo.worktree_exists("remote-feature"),
        "Worktree should be created"
    );
}

/// Spec: Post-creation commands are executed (cmds in local config)
#[test]
fn test_open_runs_post_creation_commands() {
    let repo = TestRepo::new();

    // Write a local config with a command that creates a marker file
    repo.write_local_config(
        r#"
cmds = [
    "touch $DST/.gwik-marker"
]
"#,
    );

    let output = repo.gwik(&["open", "feature-cmds"]);

    assert!(
        output.status.success(),
        "gwik open should succeed: {}",
        stderr(&output)
    );

    // Check that the marker file was created
    let marker_path = repo.worktree_path("feature-cmds").join(".gwik-marker");
    assert!(
        marker_path.exists(),
        "Post-creation command should have run"
    );
}

/// Spec: If post-creation command fails, execution stops
#[test]
fn test_open_stops_on_post_command_failure() {
    let repo = TestRepo::new();

    // Write a config with a failing command followed by another
    repo.write_local_config(
        r#"
cmds = [
    "exit 1",
    "touch $DST/.should-not-exist"
]
"#,
    );

    let output = repo.gwik(&["open", "feature-fail"]);

    // Command should fail
    assert!(!output.status.success(), "Should fail when command fails");

    // The worktree should still exist (as per spec)
    assert!(
        repo.worktree_exists("feature-fail"),
        "Worktree should remain after command failure"
    );

    // Second command should not have run
    let marker_path = repo.worktree_path("feature-fail").join(".should-not-exist");
    assert!(
        !marker_path.exists(),
        "Second command should not run after failure"
    );
}

/// Spec: After creation, outputs cd command
#[test]
fn test_open_outputs_cd_command() {
    let repo = TestRepo::new();

    let output = repo.gwik(&["open", "feature-cd"]);

    assert!(output.status.success());

    let out = stdout(&output);
    assert!(out.starts_with("cd "), "Output should be a cd command");
    assert!(
        out.contains(".worktrees/feature-cd"),
        "cd should point to worktree path"
    );
}

/// Test opening multiple worktrees
#[test]
fn test_open_multiple_worktrees() {
    let repo = TestRepo::new();

    let output1 = repo.gwik(&["open", "feature-a"]);
    let output2 = repo.gwik(&["open", "feature-b"]);
    let output3 = repo.gwik(&["open", "feature-c"]);

    assert!(output1.status.success());
    assert!(output2.status.success());
    assert!(output3.status.success());

    assert!(repo.worktree_exists("feature-a"));
    assert!(repo.worktree_exists("feature-b"));
    assert!(repo.worktree_exists("feature-c"));
}

/// Test nested branch name conversion (e.g., a/b/c -> a-b-c)
#[test]
fn test_open_nested_branch_name() {
    let repo = TestRepo::new();

    let output = repo.gwik(&["open", "feature/auth/login"]);

    assert!(output.status.success());
    assert!(
        repo.worktree_exists("feature-auth-login"),
        "Nested slashes should all be converted"
    );
}
