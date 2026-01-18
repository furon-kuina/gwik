mod common;

use common::{stderr, stdout, TestRepo};

/// gwik cd <worktree-name>: Outputs a cd command to navigate to the worktree
/// Spec: Requires exact match of worktree name
#[test]
fn test_cd_outputs_cd_command() {
    let repo = TestRepo::new();

    repo.gwik(&["open", "feature-nav"]);

    let output = repo.gwik(&["cd", "feature-nav"]);

    assert!(
        output.status.success(),
        "gwik cd failed: {}",
        stderr(&output)
    );

    let out = stdout(&output);
    assert!(out.starts_with("cd "), "Output should be a cd command");
    assert!(
        out.contains("feature-nav"),
        "cd should point to the worktree"
    );
    assert!(
        out.contains(".worktrees"),
        "Path should include .worktrees directory"
    );
}

/// Spec: If worktree not found, shows error with available worktrees
#[test]
fn test_cd_not_found_shows_available() {
    let repo = TestRepo::new();

    repo.gwik(&["open", "feature-a"]);
    repo.gwik(&["open", "feature-b"]);

    let output = repo.gwik(&["cd", "nonexistent"]);

    assert!(
        !output.status.success(),
        "Should fail for non-existent worktree"
    );

    let err = stderr(&output);
    assert!(err.contains("not found"), "Error should indicate not found");
    assert!(
        err.contains("Available worktrees") || err.contains("feature-a"),
        "Should show available worktrees"
    );
}

/// Spec: Exact match required (no partial matching)
#[test]
fn test_cd_requires_exact_match() {
    let repo = TestRepo::new();

    repo.gwik(&["open", "feature-complete"]);

    // Partial match should fail
    let output = repo.gwik(&["cd", "feature"]);

    assert!(!output.status.success(), "Partial match should not work");

    let err = stderr(&output);
    assert!(err.contains("not found"), "Should indicate not found");
}

/// Test cd works from inside another worktree
#[test]
fn test_cd_from_worktree_to_worktree() {
    let repo = TestRepo::new();

    repo.gwik(&["open", "feature-src"]);
    repo.gwik(&["open", "feature-dst"]);

    let src_path = repo.worktree_path("feature-src");
    let output = repo.gwik_in(&src_path, &["cd", "feature-dst"]);

    assert!(output.status.success());

    let out = stdout(&output);
    assert!(
        out.contains("feature-dst"),
        "Should output cd to destination"
    );
}

/// Test cd with converted slash names
#[test]
fn test_cd_with_slash_converted_name() {
    let repo = TestRepo::new();

    // Create worktree with slash in branch name
    repo.gwik(&["open", "feature/auth"]);

    // cd uses directory name (hyphens)
    let output = repo.gwik(&["cd", "feature-auth"]);

    assert!(
        output.status.success(),
        "Should find worktree by directory name"
    );

    let out = stdout(&output);
    assert!(
        out.contains("feature-auth"),
        "cd should use hyphenated name"
    );
}

/// Test cd error message when no worktrees exist
#[test]
fn test_cd_no_worktrees() {
    let repo = TestRepo::new();

    let output = repo.gwik(&["cd", "ghost"]);

    assert!(!output.status.success());

    let err = stderr(&output);
    assert!(err.contains("not found"), "Should indicate not found");
}

/// Test cd outputs full absolute path
#[test]
fn test_cd_outputs_absolute_path() {
    let repo = TestRepo::new();

    repo.gwik(&["open", "feature-abs"]);

    let output = repo.gwik(&["cd", "feature-abs"]);

    assert!(output.status.success());

    let out = stdout(&output);
    // Path should be absolute (start with /)
    let path_part = out.strip_prefix("cd ").unwrap_or(&out).trim();
    assert!(
        path_part.starts_with('/'),
        "Path should be absolute: {}",
        path_part
    );
}

/// Test multiple cd commands in sequence (stateless)
#[test]
fn test_cd_is_stateless() {
    let repo = TestRepo::new();

    repo.gwik(&["open", "wt-1"]);
    repo.gwik(&["open", "wt-2"]);

    let output1 = repo.gwik(&["cd", "wt-1"]);
    let output2 = repo.gwik(&["cd", "wt-2"]);
    let output3 = repo.gwik(&["cd", "wt-1"]);

    assert!(output1.status.success());
    assert!(output2.status.success());
    assert!(output3.status.success());

    assert!(stdout(&output1).contains("wt-1"));
    assert!(stdout(&output2).contains("wt-2"));
    assert!(stdout(&output3).contains("wt-1"));
}
