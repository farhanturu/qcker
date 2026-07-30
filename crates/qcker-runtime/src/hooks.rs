use std::path::PathBuf;
use std::process::Command;

use crate::spec::Container;
use qcker_common::error::{QckerError, Result};

#[derive(Debug, Clone)]
pub struct Hook {
    pub path: PathBuf,
    pub args: Vec<String>,
    pub env: Vec<String>,
    pub timeout: Option<u32>,
}

#[derive(Debug, Clone)]
pub enum HookType {
    Prestart,
    CreateRuntime,
    CreateContainer,
    StartContainer,
    Poststart,
    Poststop,
}

pub fn execute_hook(hook: &Hook, container: &Container) -> Result<()> {
    let mut cmd = Command::new(&hook.path);

    cmd.args(&hook.args);

    for env in &hook.env {
        let parts: Vec<&str> = env.splitn(2, '=').collect();
        if parts.len() == 2 {
            cmd.env(parts[0], parts[1]);
        }
    }

    cmd.env("container_id", &container.id);
    cmd.env("container_pid", container.pid.unwrap_or(0).to_string());

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

pub fn execute_prestart_hooks(container: &Container, hooks: &[Hook]) -> Result<()> {
    for hook in hooks {
        execute_hook(hook, container)?;
    }
    Ok(())
}

pub fn execute_poststart_hooks(container: &Container, hooks: &[Hook]) -> Result<()> {
    for hook in hooks {
        execute_hook(hook, container)?;
    }
    Ok(())
}

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
