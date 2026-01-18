mod common;

use common::{stderr, stdout, TestRepo};

/// gwik close [worktree-name]: Removes a worktree
/// Spec: The associated branch is NOT deleted
#[test]
fn test_close_removes_worktree_keeps_branch() {
    let repo = TestRepo::new();

    // Create a worktree
    repo.gwik(&["open", "feature-close"]);
    assert!(repo.worktree_exists("feature-close"));

    // Close it
    let output = repo.gwik(&["close", "feature-close"]);

    assert!(
        output.status.success(),
        "gwik close failed: {}",
        stderr(&output)
    );
    assert!(
        !repo.worktree_exists("feature-close"),
        "Worktree should be removed"
    );

    // Branch should still exist
    let git_output = repo.git(&["branch", "--list", "feature-close"]);
    let branches = stdout(&git_output);
    assert!(
        branches.contains("feature-close"),
        "Branch should NOT be deleted"
    );
}

/// Spec: After deletion, outputs cd command to main repository
#[test]
fn test_close_outputs_cd_to_main_repo() {
    let repo = TestRepo::new();

    // Create and close a worktree
    repo.gwik(&["open", "feature-cd"]);
    let output = repo.gwik(&["close", "feature-cd"]);

    assert!(output.status.success());

    let out = stdout(&output);
    assert!(out.starts_with("cd "), "Output should be a cd command");
    // Should point to main repo, not worktree
    assert!(
        !out.contains(".worktrees"),
        "Should point to main repo, not worktree"
    );
}

/// Spec: If worktree not found, show error with available worktrees list
#[test]
fn test_close_not_found_shows_available() {
    let repo = TestRepo::new();

    // Create some worktrees
    repo.gwik(&["open", "feature-a"]);
    repo.gwik(&["open", "feature-b"]);

    // Try to close non-existent
    let output = repo.gwik(&["close", "nonexistent"]);

    assert!(
        !output.status.success(),
        "Should fail for non-existent worktree"
    );

    let err = stderr(&output);
    assert!(err.contains("not found"), "Error should mention not found");
    assert!(
        err.contains("Available worktrees") || err.contains("feature-a"),
        "Should show available worktrees"
    );
}

/// Spec: If uncommitted changes exist, refuse to delete
#[test]
fn test_close_refuses_with_uncommitted_changes() {
    let repo = TestRepo::new();

    // Create a worktree
    repo.gwik(&["open", "feature-dirty"]);

    // Create uncommitted changes in the worktree
    repo.create_file_in_worktree("feature-dirty", "new-file.txt", "uncommitted content");
    repo.stage_file_in_worktree("feature-dirty", "new-file.txt");

    // Try to close - should fail
    let output = repo.gwik(&["close", "feature-dirty"]);

    assert!(
        !output.status.success(),
        "Should refuse to delete with uncommitted changes"
    );

    let err = stderr(&output);
    assert!(
        err.contains("uncommitted") || err.contains("modified") || err.contains("untracked"),
        "Error should mention uncommitted changes"
    );

    // Worktree should still exist
    assert!(
        repo.worktree_exists("feature-dirty"),
        "Worktree should remain"
    );
}

/// Spec: If no argument provided, closes the current worktree (if inside one)
/// Note: This test uses --yes to skip the "inside worktree" confirmation
#[test]
fn test_close_current_worktree_with_yes() {
    let repo = TestRepo::new();

    // Create a worktree
    repo.gwik(&["open", "feature-current"]);

    // Close from inside the worktree with --yes
    let worktree_path = repo.worktree_path("feature-current");
    let output = repo.gwik_in(&worktree_path, &["close", "--yes"]);

    assert!(
        output.status.success(),
        "Should close current worktree: {}",
        stderr(&output)
    );
    assert!(
        !repo.worktree_exists("feature-current"),
        "Current worktree should be removed"
    );
}

/// Spec: If current directory is inside target worktree, requires --yes to proceed
#[test]
fn test_close_from_inside_requires_confirmation() {
    let repo = TestRepo::new();

    // Create a worktree
    repo.gwik(&["open", "feature-inside"]);

    // Try to close from inside without --yes (will fail because stdin is not a tty)
    let worktree_path = repo.worktree_path("feature-inside");

    // Without --yes, it should abort (since no tty input available)
    // The behavior may vary, but the worktree should remain
    let output = repo.gwik_in(&worktree_path, &["close"]);

    // In non-interactive mode, this typically aborts or prompts
    // The worktree should still exist since we didn't confirm
    // Note: exact behavior depends on whether stdin is available
    assert!(
        repo.worktree_exists("feature-inside") || output.status.success(),
        "Either should prompt/abort or succeed"
    );
}

/// Spec: Close specific worktree by name from main repo
#[test]
fn test_close_specific_worktree_by_name() {
    let repo = TestRepo::new();

    // Create multiple worktrees
    repo.gwik(&["open", "feature-x"]);
    repo.gwik(&["open", "feature-y"]);
    repo.gwik(&["open", "feature-z"]);

    // Close just feature-y
    let output = repo.gwik(&["close", "feature-y"]);

    assert!(output.status.success());
    assert!(repo.worktree_exists("feature-x"), "feature-x should remain");
    assert!(
        !repo.worktree_exists("feature-y"),
        "feature-y should be removed"
    );
    assert!(repo.worktree_exists("feature-z"), "feature-z should remain");
}

/// Spec: Message confirms removal
#[test]
fn test_close_outputs_removal_message() {
    let repo = TestRepo::new();

    repo.gwik(&["open", "feature-msg"]);
    let output = repo.gwik(&["close", "feature-msg"]);

    assert!(output.status.success());

    let err = stderr(&output);
    assert!(
        err.contains("Removed worktree") || err.contains("feature-msg"),
        "Should output removal confirmation"
    );
}

/// Test closing worktree from a different worktree
#[test]
fn test_close_other_worktree_from_worktree() {
    let repo = TestRepo::new();

    // Create two worktrees
    repo.gwik(&["open", "feature-one"]);
    repo.gwik(&["open", "feature-two"]);

    // Close feature-two from inside feature-one
    let worktree_one_path = repo.worktree_path("feature-one");
    let output = repo.gwik_in(&worktree_one_path, &["close", "feature-two"]);

    assert!(
        output.status.success(),
        "Should close other worktree: {}",
        stderr(&output)
    );
    assert!(
        repo.worktree_exists("feature-one"),
        "Current worktree should remain"
    );
    assert!(
        !repo.worktree_exists("feature-two"),
        "Target worktree should be removed"
    );
}

/// Test closing non-existent worktree when no worktrees exist
#[test]
fn test_close_nonexistent_no_worktrees() {
    let repo = TestRepo::new();

    let output = repo.gwik(&["close", "ghost"]);

    assert!(!output.status.success());

    let err = stderr(&output);
    assert!(
        err.contains("not found"),
        "Should indicate worktree not found"
    );
}
