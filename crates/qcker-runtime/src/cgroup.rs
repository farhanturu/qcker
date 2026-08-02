use std::path::{Path, PathBuf};

use cgroups_rs::cgroup::Cgroup;
use cgroups_rs::hierarchies;

use qcker_common::error::{QckerError, Result};

const CGROUP_BASE: &str = "/sys/fs/cgroup";

pub fn create_cgroup(container_id: &str) -> Result<PathBuf> {
    let cgroup_path = Path::new(CGROUP_BASE).join("qcker").join(container_id);

    if !cgroup_path.exists() {
        std::fs::create_dir_all(&cgroup_path)
            .map_err(|e| QckerError::cgroup(format!("Failed to create cgroup: {}", e)))?;
    }

    Ok(cgroup_path)
}

pub fn apply_resources(cgroup_path: &Path, memory_limit: Option<i64>, cpu_quota: Option<i64>, pids_limit: Option<i64>) -> Result<()> {
    if let Some(mem_limit) = memory_limit {
        let path = cgroup_path.join("memory.max");
        std::fs::write(&path, mem_limit.to_string())
            .map_err(|e| QckerError::cgroup(format!("Failed to set memory.max: {}", e)))?;
    }

    if let Some(quota) = cpu_quota {
        let path = cgroup_path.join("cpu.max");
        let period = 100000i64;
        std::fs::write(&path, format!("{} {}", quota, period))
            .map_err(|e| QckerError::cgroup(format!("Failed to set cpu.max: {}", e)))?;
    }

    if let Some(pids) = pids_limit {
        let path = cgroup_path.join("pids.max");
        std::fs::write(&path, pids.to_string())
            .map_err(|e| QckerError::cgroup(format!("Failed to set pids.max: {}", e)))?;
    }

    Ok(())
}

pub fn add_process(cgroup_path: &Path, pid: i32) -> Result<()> {
    let procs_path = cgroup_path.join("cgroup.procs");
    std::fs::write(&procs_path, pid.to_string())
        .map_err(|e| QckerError::cgroup(format!("Failed to add process: {}", e)))?;
    Ok(())
}

pub fn remove_cgroup(cgroup_path: &Path) -> Result<()> {
    if cgroup_path.exists() {
        std::fs::remove_dir(cgroup_path)
            .map_err(|e| QckerError::cgroup(format!("Failed to remove cgroup: {}", e)))?;
    }
    Ok(())
}

pub fn get_stats(cgroup_path: &Path) -> Result<CgroupStats> {
    let mut stats = CgroupStats::default();

    let memory_current_path = cgroup_path.join("memory.current");
    if memory_current_path.exists() {
        if let Ok(content) = std::fs::read_to_string(&memory_current_path) {
            if let Ok(val) = content.trim().parse::<u64>() {
                stats.memory_current = val;
            }
        }
    }

    let memory_max_path = cgroup_path.join("memory.max");
    if memory_max_path.exists() {
        if let Ok(content) = std::fs::read_to_string(&memory_max_path) {
            let trimmed = content.trim();
            if trimmed != "max" {
                if let Ok(val) = trimmed.parse::<u64>() {
                    stats.memory_limit = val;
                }
            }
        }
    }

    let pids_current_path = cgroup_path.join("pids.current");
    if pids_current_path.exists() {
        if let Ok(content) = std::fs::read_to_string(&pids_current_path) {
            if let Ok(val) = content.trim().parse::<i64>() {
                stats.pids_current = val;
            }
        }
    }

    let cpu_stat_path = cgroup_path.join("cpu.stat");
    if cpu_stat_path.exists() {
        if let Ok(content) = std::fs::read_to_string(&cpu_stat_path) {
            for line in content.lines() {
                let parts: Vec<&str> = line.splitn(2, ' ').collect();
                if parts.len() == 2 {
                    match parts[0] {
                        "usage_usec" => {
                            if let Ok(val) = parts[1].parse::<u64>() {
                                stats.cpu_usage_usec = val;
                            }
                        }
                        "user_usec" => {
                            if let Ok(val) = parts[1].parse::<u64>() {
                                stats.cpu_user_usec = val;
                            }
                        }
                        "system_usec" => {
                            if let Ok(val) = parts[1].parse::<u64>() {
                                stats.cpu_system_usec = val;
                            }
                        }
                        _ => {}
                    }
                }
            }
        }
    }

    Ok(stats)
}

pub fn cgroups_v2_available() -> bool {
    let cgroup_path = Path::new(CGROUP_BASE);
    cgroup_path.exists() && cgroup_path.join("cgroup.controllers").exists()
}

pub fn load_cgroup(container_id: &str) -> Result<Cgroup> {
    let hierarchy = hierarchies::auto();
    let path = format!("qcker/{}", container_id);
    Ok(Cgroup::load(hierarchy, &path))
}

#[derive(Debug, Default)]
pub struct CgroupStats {
    pub memory_current: u64,
    pub memory_limit: u64,
    pub pids_current: i64,
    pub cpu_usage_usec: u64,
    pub cpu_user_usec: u64,
    pub cpu_system_usec: u64,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cgroups_v2_available() {
        let available = cgroups_v2_available();
        println!("Cgroups v2 available: {}", available);
    }

    #[test]
    fn test_cgroup_stats_default() {
        let stats = CgroupStats::default();
        assert_eq!(stats.memory_current, 0);
        assert_eq!(stats.memory_limit, 0);
        assert_eq!(stats.pids_current, 0);
    }
}
