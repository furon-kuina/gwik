mod common;

use common::{stdout, TestRepo};

/// gwik list: Lists all worktrees in the current repository
/// Spec: Outputs only paths, one per line
#[test]
fn test_list_outputs_paths() {
    let repo = TestRepo::new();

    // Create some worktrees
    repo.gwik(&["open", "feature-a"]);
    repo.gwik(&["open", "feature-b"]);

    let output = repo.gwik(&["list"]);

    assert!(output.status.success());

    let out = stdout(&output);
    let lines: Vec<&str> = out.lines().collect();

    assert_eq!(lines.len(), 2, "Should list 2 worktrees");
    assert!(
        lines.iter().any(|l| l.contains("feature-a")),
        "Should contain feature-a"
    );
    assert!(
        lines.iter().any(|l| l.contains("feature-b")),
        "Should contain feature-b"
    );
}

/// Spec: Does not include the main worktree
#[test]
fn test_list_excludes_main_worktree() {
    let repo = TestRepo::new();

    // Create a worktree
    repo.gwik(&["open", "feature-x"]);

    let output = repo.gwik(&["list"]);

    assert!(output.status.success());

    let out = stdout(&output);
    let lines: Vec<&str> = out.lines().collect();

    // Should only have one line (the worktree), not the main repo
    assert_eq!(lines.len(), 1, "Should only list worktree, not main");
    assert!(lines[0].contains("feature-x"), "Should list the worktree");
    assert!(
        lines[0].contains(".worktrees"),
        "Path should be in .worktrees"
    );
}

/// Test list with no worktrees returns empty output
#[test]
fn test_list_empty_when_no_worktrees() {
    let repo = TestRepo::new();

    let output = repo.gwik(&["list"]);

    assert!(output.status.success());

    let out = stdout(&output);
    assert!(
        out.trim().is_empty(),
        "Should output nothing when no worktrees"
    );
}

/// Spec: Output is suitable for piping to fzf/peco
#[test]
fn test_list_output_format_for_piping() {
    let repo = TestRepo::new();

    repo.gwik(&["open", "feature-1"]);
    repo.gwik(&["open", "feature-2"]);
    repo.gwik(&["open", "feature-3"]);

    let output = repo.gwik(&["list"]);

    assert!(output.status.success());

    let out = stdout(&output);

    // Each line should be a full path
    for line in out.lines() {
        assert!(
            line.starts_with('/'),
            "Each line should be an absolute path"
        );
        assert!(!line.contains('\t'), "No tabs in output");
        assert!(line.contains(".worktrees"), "Should be worktree path");
    }
}

/// Test list works from inside a worktree
#[test]
fn test_list_from_worktree() {
    let repo = TestRepo::new();

    repo.gwik(&["open", "feature-inside"]);
    repo.gwik(&["open", "feature-other"]);

    // Run list from inside a worktree
    let worktree_path = repo.worktree_path("feature-inside");
    let output = repo.gwik_in(&worktree_path, &["list"]);

    assert!(output.status.success());

    let out = stdout(&output);
    let lines: Vec<&str> = out.lines().collect();

    // Should list both worktrees
    assert_eq!(
        lines.len(),
        2,
        "Should list all worktrees from inside a worktree"
    );
}

/// Test that list shows all worktrees including ones with slashes in branch names
#[test]
fn test_list_shows_converted_names() {
    let repo = TestRepo::new();

    repo.gwik(&["open", "feature/auth/login"]);
    repo.gwik(&["open", "bugfix-simple"]);

    let output = repo.gwik(&["list"]);

    assert!(output.status.success());

    let out = stdout(&output);
    assert!(
        out.contains("feature-auth-login"),
        "Should show converted directory name"
    );
    assert!(out.contains("bugfix-simple"), "Should show simple name");
}
