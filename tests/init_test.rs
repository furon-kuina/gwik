mod common;

use common::{stderr, stdout, TestRepo};

/// gwik init <shell>: Outputs shell integration code
/// Spec: Supported shells: bash, zsh
#[test]
fn test_init_bash() {
    let repo = TestRepo::new();

    let output = repo.gwik(&["init", "bash"]);

    assert!(
        output.status.success(),
        "gwik init bash failed: {}",
        stderr(&output)
    );

    let out = stdout(&output);
    assert!(out.contains("gwik()"), "Should contain shell function");
    assert!(out.contains("bash"), "Should mention bash");
}

#[test]
fn test_init_zsh() {
    let repo = TestRepo::new();

    let output = repo.gwik(&["init", "zsh"]);

    assert!(
        output.status.success(),
        "gwik init zsh failed: {}",
        stderr(&output)
    );

    let out = stdout(&output);
    assert!(out.contains("gwik()"), "Should contain shell function");
    assert!(out.contains("zsh"), "Should mention zsh");
}

/// Spec: Unsupported shell returns error
#[test]
fn test_init_unsupported_shell() {
    let repo = TestRepo::new();

    let output = repo.gwik(&["init", "fish"]);

    assert!(!output.status.success(), "Unsupported shell should fail");

    let err = stderr(&output);
    assert!(
        err.contains("Unsupported shell") || err.contains("fish"),
        "Error should mention unsupported shell"
    );
}

/// Spec: Shell wrapper enables automatic cd after gwik open/close/cd
#[test]
fn test_init_bash_has_cd_handling() {
    let repo = TestRepo::new();

    let output = repo.gwik(&["init", "bash"]);

    assert!(output.status.success());

    let out = stdout(&output);
    // The wrapper should detect "cd " output and eval it
    assert!(out.contains("eval"), "Should use eval for cd commands");
    assert!(
        out.contains("cd ") || out.contains("cd\\"),
        "Should detect cd commands"
    );
}

#[test]
fn test_init_zsh_has_cd_handling() {
    let repo = TestRepo::new();

    let output = repo.gwik(&["init", "zsh"]);

    assert!(output.status.success());

    let out = stdout(&output);
    assert!(out.contains("eval"), "Should use eval for cd commands");
    assert!(
        out.contains("cd ") || out.contains("cd\\"),
        "Should detect cd commands"
    );
}

/// Spec: Generates shell completion scripts
#[test]
fn test_init_bash_includes_completions() {
    let repo = TestRepo::new();

    let output = repo.gwik(&["init", "bash"]);

    assert!(output.status.success());

    let out = stdout(&output);
    // Should contain completion code for bash
    assert!(
        out.contains("complete") || out.contains("COMPREPLY") || out.contains("_gwik"),
        "Should include bash completion code"
    );
}

#[test]
fn test_init_zsh_includes_completions() {
    let repo = TestRepo::new();

    let output = repo.gwik(&["init", "zsh"]);

    assert!(output.status.success());

    let out = stdout(&output);
    // Should contain completion code for zsh
    assert!(
        out.contains("compdef") || out.contains("#compdef") || out.contains("_gwik"),
        "Should include zsh completion code"
    );
}

/// Test that output can be used with eval
#[test]
fn test_init_output_is_valid_shell_code() {
    let repo = TestRepo::new();

    let output = repo.gwik(&["init", "bash"]);

    assert!(output.status.success());

    let out = stdout(&output);

    // Basic syntax check - should have balanced braces for function
    let open_braces = out.matches('{').count();
    let close_braces = out.matches('}').count();
    assert_eq!(open_braces, close_braces, "Should have balanced braces");
}

/// Test init with various invalid inputs
#[test]
fn test_init_invalid_shells() {
    let repo = TestRepo::new();

    for shell in &["powershell", "cmd", "nu", "elvish", "tcsh"] {
        let output = repo.gwik(&["init", shell]);
        assert!(
            !output.status.success(),
            "{} should not be supported",
            shell
        );
    }
}

/// Spec: Shell function passes through to gwik binary
#[test]
fn test_init_wrapper_calls_gwik_binary() {
    let repo = TestRepo::new();

    let output = repo.gwik(&["init", "bash"]);

    assert!(output.status.success());

    let out = stdout(&output);
    // The wrapper should call "command gwik" to invoke the binary
    assert!(
        out.contains("command gwik") || out.contains("gwik \"$@\""),
        "Wrapper should invoke gwik binary"
    );
}
