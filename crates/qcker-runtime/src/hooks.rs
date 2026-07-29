use std::path::PathBuf;
use std::process::Command;

use crate::spec::Container;
use qcker_common::error::{QckerError, Result};

/// OCI lifecycle hook
#[derive(Debug, Clone)]
pub struct Hook {
    pub path: PathBuf,
    pub args: Vec<String>,
    pub env: Vec<String>,
    pub timeout: Option<u32>,
}

/// Hook type
#[derive(Debug, Clone)]
pub enum HookType {
    Prestart,
    CreateRuntime,
    CreateContainer,
    StartContainer,
    Poststart,
    Poststop,
}

/// Execute a hook
pub fn execute_hook(hook: &Hook, container: &Container) -> Result<()> {
    let mut cmd = Command::new(&hook.path);

    // Set arguments
    cmd.args(&hook.args);

    // Set environment variables
    for env in &hook.env {
        let parts: Vec<&str> = env.splitn(2, '=').collect();
        if parts.len() == 2 {
            cmd.env(parts[0], parts[1]);
        }
    }

    // Set container state as environment variable
    cmd.env("container_id", &container.id);
    cmd.env("container_pid", container.pid.unwrap_or(0).to_string());

    // Execute hook
    let output = cmd
        .output()
        .map_err(|e| QckerError::Process(format!("Failed to execute hook: {}", e)))?;

    if !output.status.success() {
        return Err(QckerError::Process(format!(
            "Hook failed with exit code: {}",
            output.status.code().unwrap_or(-1)
        )));
    }

    Ok(())
}

/// Execute prestart hooks
pub fn execute_prestart_hooks(container: &Container, hooks: &[Hook]) -> Result<()> {
    for hook in hooks {
        execute_hook(hook, container)?;
    }
    Ok(())
}

/// Execute poststart hooks
pub fn execute_poststart_hooks(container: &Container, hooks: &[Hook]) -> Result<()> {
    for hook in hooks {
        execute_hook(hook, container)?;
    }
    Ok(())
}

/// Execute poststop hooks
pub fn execute_poststop_hooks(container: &Container, hooks: &[Hook]) -> Result<()> {
    for hook in hooks {
        execute_hook(hook, container)?;
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_hook_creation() {
        let hook = Hook {
            path: PathBuf::from("/usr/bin/echo"),
            args: vec!["hello".to_string()],
            env: vec![],
            timeout: Some(5),
        };
        assert_eq!(hook.path, PathBuf::from("/usr/bin/echo"));
    }
}
