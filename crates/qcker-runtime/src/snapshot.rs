use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

use qcker_common::error::{QckerError, Result};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SnapshotInfo {
    pub id: String,
    pub container_id: String,
    pub created_at: String,
    pub path: PathBuf,
    pub pid: Option<i32>,
    pub image: String,
}

#[derive(Debug, Clone)]
pub struct CheckpointOptions {
    pub leave_running: bool,
    pub track_pid: bool,
    pub external: bool,
    pub shell_job: bool,
    pub images_dir: PathBuf,
    pub work_dir: PathBuf,
}

impl Default for CheckpointOptions {
    fn default() -> Self {
        Self {
            leave_running: false,
            track_pid: true,
            external: false,
            shell_job: true,
            images_dir: PathBuf::from("/tmp"),
            work_dir: PathBuf::from("/tmp"),
        }
    }
}

pub fn is_criu_available() -> bool {
    Command::new("criu")
        .arg("--version")
        .output()
        .map(|out| out.status.success())
        .unwrap_or(false)
}

pub fn check_criu_requirements() -> Result<Vec<String>> {
    let mut issues = Vec::new();

    if !is_criu_available() {
        issues.push("CRIU binary not found. Install with: apt-get install criu".to_string());
        return Ok(issues);
    }

    let version_output = Command::new("criu")
        .arg("--version")
        .output()
        .ok();

    if let Some(out) = version_output {
        let version = String::from_utf8_lossy(&out.stdout);
        if version.contains("3.0") || version.contains("3.1") || version.contains("3.2")
            || version.contains("4.") || version.contains("5.") {
            tracing::info!("CRIU version: {}", version.trim());
        } else {
            issues.push(format!("Unusual CRIU version: {}", version.trim()));
        }
    }

    let check_output = Command::new("criu")
        .arg("check")
        .output()
        .ok();

    if let Some(out) = check_output {
        let check_result = String::from_utf8_lossy(&out.stdout);
        let check_stderr = String::from_utf8_lossy(&out.stderr);

        if check_result.contains("FAILED") || check_stderr.contains("FAILED") {
            issues.push("CRIU requirements check failed. See output above.".to_string());
        }
    }

    Ok(issues)
}

pub fn checkpoint_container(
    container_id: &str,
    data_dir: &Path,
    options: &CheckpointOptions,
) -> Result<SnapshotInfo> {
    let container_dir = data_dir.join("containers").join(container_id);
    let state_path = container_dir.join("state.json");

    if !state_path.exists() {
        return Err(QckerError::process(format!(
            "Container {} not found",
            container_id
        )));
    }

    let state_content = fs::read_to_string(&state_path)?;
    let state: serde_json::Value = serde_json::from_str(&state_content)?;
    let pid = state["pid"].as_i64().unwrap_or(-1) as i32;

    if pid <= 0 {
        return Err(QckerError::process(format!(
            "Container {} is not running (no PID)",
            container_id
        )));
    }

    let snapshot_name = format!("snapshot-{}-{}", container_id, chrono::Utc::now().format("%Y%m%d-%H%M%S"));
    let snapshot_dir = data_dir.join("snapshots").join(&snapshot_name);

    fs::create_dir_all(&snapshot_dir)
        .map_err(|e| QckerError::internal(format!("Failed to create snapshot dir: {}", e)))?;

    let mut cmd = Command::new("criu");
    cmd.arg("dump")
        .arg("-t")
        .arg(pid.to_string())
        .arg("--images-dir")
        .arg(&snapshot_dir)
        .arg("--work-dir")
        .arg(&options.work_dir)
        .arg("-vv")
        .arg("--shell-job");

    if options.leave_running {
        cmd.arg("-l");
    }

    if options.external {
        cmd.arg("-e");
    }

    tracing::info!(
        "Checkpointing container {} (PID {}) to {}",
        container_id,
        pid,
        snapshot_dir.display()
    );

    let output = cmd.output()?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        let stdout = String::from_utf8_lossy(&output.stdout);
        tracing::error!("Checkpoint failed: {}\n{}", stderr, stdout);
        return Err(QckerError::process(format!(
            "Checkpoint failed: {}",
            stderr.lines().next().unwrap_or("unknown error")
        )));
    }

    let container_name = state["id"].as_str().unwrap_or(container_id).to_string();
    let image = state.dig_str(&["config", "hostname"])
        .unwrap_or_else(|| "unknown".to_string());

    let snapshot_info = SnapshotInfo {
        id: snapshot_name.clone(),
        container_id: container_id.to_string(),
        created_at: chrono::Utc::now().to_rfc3339(),
        path: snapshot_dir.clone(),
        pid: Some(pid),
        image,
    };

    let info_path = snapshot_dir.join("snapshot.info");
    let info_json = serde_json::to_string_pretty(&snapshot_info)?;
    fs::write(&info_path, info_json)?;

    Ok(snapshot_info)
}

pub fn restore_snapshot(
    snapshot_path: &Path,
    new_container_id: Option<&str>,
    data_dir: &Path,
) -> Result<String> {
    if !snapshot_path.exists() {
        return Err(QckerError::process(format!(
            "Snapshot not found: {}",
            snapshot_path.display()
        )));
    }

    let info_path = snapshot_path.join("snapshot.info");
    if !info_path.exists() {
        return Err(QckerError::process("Invalid snapshot: missing snapshot.info".to_string()));
    }

    let info_content = fs::read_to_string(&info_path)?;
    let snapshot_info: SnapshotInfo = serde_json::from_str(&info_content)?;

    let container_id = new_container_id
        .unwrap_or(&format!("restored-{}", snapshot_info.id))
        .to_string();

    let container_dir = data_dir.join("containers").join(&container_id);
    fs::create_dir_all(&container_dir)?;

    let mut cmd = Command::new("criu");
    cmd.arg("restore")
        .arg("--images-dir")
        .arg(&snapshot_info.path)
        .arg("--work-dir")
        .arg(snapshot_path)
        .arg("-vv")
        .arg("--shell-job");

    tracing::info!(
        "Restoring snapshot {} to container {}",
        snapshot_info.id,
        container_id
    );

    let output = cmd.output()?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        tracing::error!("Restore failed: {}", stderr);
        return Err(QckerError::process(format!(
            "Restore failed: {}",
            stderr.lines().next().unwrap_or("unknown error")
        )));
    }

    let new_state = serde_json::json!({
        "id": container_id,
        "state": "Running",
        "bundle": container_dir,
        "pid": null,
        "rootfs": snapshot_info.path.join("rootfs"),
        "created_at": chrono::Utc::now().to_rfc3339(),
        "snapshot_source": snapshot_info.id,
        "config": {
            "oci_version": "1.0.0",
            "root": {
                "path": ".",
                "readonly": false
            },
            "process": {
                "terminal": false,
                "user": {"uid": 0, "gid": 0},
                "args": ["/bin/sh"],
                "env": [],
                "cwd": "/",
                "capabilities": null,
                "rlimits": [],
                "no_new_privileges": true
            },
            "hostname": Some(snapshot_info.image),
            "mounts": [],
            "linux": {
                "namespaces": [],
                "resources": null,
                "uid_mappings": [],
                "gid_mappings": [],
                "seccomp": null
            }
        }
    });

    let state_json = serde_json::to_string_pretty(&new_state)?;
    fs::write(container_dir.join("state.json"), state_json)?;

    Ok(container_id)
}

pub fn list_snapshots(data_dir: &Path) -> Result<Vec<SnapshotInfo>> {
    let snapshots_dir = data_dir.join("snapshots");
    if !snapshots_dir.exists() {
        return Ok(Vec::new());
    }

    let mut snapshots = Vec::new();
    for entry in fs::read_dir(&snapshots_dir)? {
        let entry = entry?;
        let snapshot_info_path = entry.path().join("snapshot.info");
        if snapshot_info_path.exists() {
            let content = fs::read_to_string(&snapshot_info_path)?;
            let info: SnapshotInfo = serde_json::from_str(&content)?;
            snapshots.push(info);
        }
    }

    snapshots.sort_by(|a, b| b.created_at.cmp(&a.created_at));
    Ok(snapshots)
}

pub fn delete_snapshot(snapshot_id: &str, data_dir: &Path) -> Result<()> {
    let snapshot_dir = data_dir.join("snapshots").join(snapshot_id);
    if !snapshot_dir.exists() {
        return Err(QckerError::process(format!(
            "Snapshot not found: {}",
            snapshot_id
        )));
    }

    fs::remove_dir_all(&snapshot_dir)?;
    tracing::info!("Deleted snapshot: {}", snapshot_id);
    Ok(())
}

trait JsonValueExt {
    fn dig_str(&self, path: &[&str]) -> Option<String>;
}

impl JsonValueExt for serde_json::Value {
    fn dig_str(&self, path: &[&str]) -> Option<String> {
        let mut current = self;
        for key in path {
            current = current.get(*key)?;
        }
        current.as_str().map(|s| s.to_string())
    }
}

