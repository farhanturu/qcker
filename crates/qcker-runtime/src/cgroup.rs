use std::fs;
use std::path::{Path, PathBuf};

use crate::spec::ResourcesConfig;
use qcker_common::error::{QckerError, Result};

/// Cgroup v2 base path
const CGROUP_BASE: &str = "/sys/fs/cgroup";

/// Create a cgroup for a container
pub fn create_cgroup(container_id: &str) -> Result<PathBuf> {
    let cgroup_path = Path::new(CGROUP_BASE).join("qcker").join(container_id);

    if !cgroup_path.exists() {
        fs::create_dir_all(&cgroup_path)
            .map_err(|e| QckerError::Cgroup(format!("Failed to create cgroup: {}", e)))?;
    }

    Ok(cgroup_path)
}

/// Apply resource limits to cgroup
pub fn apply_resources(cgroup_path: &Path, resources: &ResourcesConfig) -> Result<()> {
    // Apply memory limits
    if let Some(ref memory) = resources.memory {
        if let Some(limit) = memory.limit {
            let path = cgroup_path.join("memory.max");
            fs::write(&path, limit.to_string())
                .map_err(|e| QckerError::Cgroup(format!("Failed to set memory.max: {}", e)))?;
        }
    }

    // Apply CPU limits
    if let Some(ref cpu) = resources.cpu {
        if let (Some(quota), Some(period)) = (cpu.quota, cpu.period) {
            let path = cgroup_path.join("cpu.max");
            let value = format!("{} {}", quota, period);
            fs::write(&path, &value)
                .map_err(|e| QckerError::Cgroup(format!("Failed to set cpu.max: {}", e)))?;
        }
    }

    // Apply PID limits
    if let Some(ref pids) = resources.pids {
        let path = cgroup_path.join("pids.max");
        fs::write(&path, pids.limit.to_string())
            .map_err(|e| QckerError::Cgroup(format!("Failed to set pids.max: {}", e)))?;
    }

    Ok(())
}

/// Add a process to a cgroup
pub fn add_process(cgroup_path: &Path, pid: i32) -> Result<()> {
    let procs_path = cgroup_path.join("cgroup.procs");
    fs::write(&procs_path, pid.to_string())
        .map_err(|e| QckerError::Cgroup(format!("Failed to add process to cgroup: {}", e)))?;
    Ok(())
}

/// Remove a cgroup
pub fn remove_cgroup(cgroup_path: &Path) -> Result<()> {
    if cgroup_path.exists() {
        fs::remove_dir(cgroup_path)
            .map_err(|e| QckerError::Cgroup(format!("Failed to remove cgroup: {}", e)))?;
    }
    Ok(())
}

/// Get cgroup stats
pub fn get_stats(cgroup_path: &Path) -> Result<CgroupStats> {
    let mut stats = CgroupStats::default();

    // Read memory current
    let memory_current_path = cgroup_path.join("memory.current");
    if memory_current_path.exists() {
        if let Ok(content) = fs::read_to_string(&memory_current_path) {
            if let Ok(val) = content.trim().parse::<u64>() {
                stats.memory_current = val;
            }
        }
    }

    // Read pids current
    let pids_current_path = cgroup_path.join("pids.current");
    if pids_current_path.exists() {
        if let Ok(content) = fs::read_to_string(&pids_current_path) {
            if let Ok(val) = content.trim().parse::<i64>() {
                stats.pids_current = val;
            }
        }
    }

    Ok(stats)
}

/// Check if cgroups v2 is available
pub fn cgroups_v2_available() -> bool {
    let cgroup_path = Path::new(CGROUP_BASE);
    cgroup_path.exists() && cgroup_path.join("cgroup.controllers").exists()
}

/// Cgroup statistics
#[derive(Debug, Default)]
pub struct CgroupStats {
    pub memory_current: u64,
    pub pids_current: i64,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cgroups_v2_available() {
        // This test will pass on systems with cgroups v2
        // On systems without, it will return false
        let available = cgroups_v2_available();
        println!("Cgroups v2 available: {}", available);
    }
}
