use anyhow::Result;

use crate::shell::generate_shell_integration;

pub fn run(shell: &str) -> Result<()> {
    let integration = generate_shell_integration(shell)?;
    println!("{}", integration);
    Ok(())
}
