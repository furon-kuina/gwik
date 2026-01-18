use anyhow::{bail, Result};
use clap::CommandFactory;
use clap_complete::{generate, Shell};

use crate::cli::Cli;

pub fn generate_shell_integration(shell: &str) -> Result<String> {
    match shell {
        "bash" => Ok(generate_bash()),
        "zsh" => Ok(generate_zsh()),
        _ => bail!("Unsupported shell: {}. Use 'bash' or 'zsh'", shell),
    }
}

fn generate_bash() -> String {
    let mut completion = Vec::new();
    generate(Shell::Bash, &mut Cli::command(), "gwik", &mut completion);
    let completion_str = String::from_utf8_lossy(&completion);

    format!(
        r#"# gwik shell integration for bash

# Shell wrapper function
gwik() {{
    local output
    local exit_code

    # Capture the output
    output=$(command gwik "$@")
    exit_code=$?

    if [ $exit_code -eq 0 ]; then
        # Check if output starts with "cd "
        if [[ "$output" == cd\ * ]]; then
            eval "$output"
        elif [ -n "$output" ]; then
            echo "$output"
        fi
    else
        if [ -n "$output" ]; then
            echo "$output"
        fi
        return $exit_code
    fi
}}

# Completions
{completion_str}
"#
    )
}

fn generate_zsh() -> String {
    let mut completion = Vec::new();
    generate(Shell::Zsh, &mut Cli::command(), "gwik", &mut completion);
    let completion_str = String::from_utf8_lossy(&completion);

    format!(
        r#"# gwik shell integration for zsh

# Shell wrapper function
gwik() {{
    local output
    local exit_code

    # Capture the output
    output=$(command gwik "$@")
    exit_code=$?

    if [[ $exit_code -eq 0 ]]; then
        # Check if output starts with "cd "
        if [[ "$output" == cd\ * ]]; then
            eval "$output"
        elif [[ -n "$output" ]]; then
            echo "$output"
        fi
    else
        if [[ -n "$output" ]]; then
            echo "$output"
        fi
        return $exit_code
    fi
}}

# Completions
{completion_str}
"#
    )
}
