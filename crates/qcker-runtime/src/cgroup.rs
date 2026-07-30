use std::fs;
use std::path::{Path, PathBuf};

use crate::spec::ResourcesConfig;
use qcker_common::error::{QckerError, Result};

const CGROUP_BASE: &str = "/sys/fs/cgroup";

pub fn create_cgroup(container_id: &str) -> Result<PathBuf> {
    let cgroup_path = Path::new(CGROUP_BASE).join("qcker").join(container_id);

    if !cgroup_path.exists() {
        fs::create_dir_all(&cgroup_path)
            .map_err(|e| QckerError::Cgroup(format!("Failed to create cgroup: {}", e)))?;
    }

    Ok(cgroup_path)
}

pub fn apply_resources(cgroup_path: &Path, resources: &ResourcesConfig) -> Result<()> {
    if let Some(ref memory) = resources.memory {
        if let Some(limit) = memory.limit {
            let path = cgroup_path.join("memory.max");
            fs::write(&path, limit.to_string())
                .map_err(|e| QckerError::Cgroup(format!("Failed to set memory.max: {}", e)))?;
        }
    }

    if let Some(ref cpu) = resources.cpu {
        if let (Some(quota), Some(period)) = (cpu.quota, cpu.period) {
            let path = cgroup_path.join("cpu.max");
            let value = format!("{} {}", quota, period);
            fs::write(&path, &value)
                .map_err(|e| QckerError::Cgroup(format!("Failed to set cpu.max: {}", e)))?;
        }
    }

    if let Some(ref pids) = resources.pids {
        let path = cgroup_path.join("pids.max");
        fs::write(&path, pids.limit.to_string())
            .map_err(|e| QckerError::Cgroup(format!("Failed to set pids.max: {}", e)))?;
    }

    Ok(())
}

pub fn add_process(cgroup_path: &Path, pid: i32) -> Result<()> {
    let procs_path = cgroup_path.join("cgroup.procs");
    fs::write(&procs_path, pid.to_string())
        .map_err(|e| QckerError::Cgroup(format!("Failed to add process to cgroup: {}", e)))?;
    Ok(())
}

pub fn remove_cgroup(cgroup_path: &Path) -> Result<()> {
    if cgroup_path.exists() {
        fs::remove_dir(cgroup_path)
            .map_err(|e| QckerError::Cgroup(format!("Failed to remove cgroup: {}", e)))?;
    }
    Ok(())
}

pub fn get_stats(cgroup_path: &Path) -> Result<CgroupStats> {
    let mut stats = CgroupStats::default();

    // Memory current
    let memory_current_path = cgroup_path.join("memory.current");
    if memory_current_path.exists() {
        if let Ok(content) = fs::read_to_string(&memory_current_path) {
            if let Ok(val) = content.trim().parse::<u64>() {
                stats.memory_current = val;
            }
        }
    }

    // Memory max
    let memory_max_path = cgroup_path.join("memory.max");
    if memory_max_path.exists() {
        if let Ok(content) = fs::read_to_string(&memory_max_path) {
            let trimmed = content.trim();
            if trimmed != "max" {
                if let Ok(val) = trimmed.parse::<u64>() {
                    stats.memory_limit = val;
                }
            }
        }
    }

    // PIDs current
    let pids_current_path = cgroup_path.join("pids.current");
    if pids_current_path.exists() {
        if let Ok(content) = fs::read_to_string(&pids_current_path) {
            if let Ok(val) = content.trim().parse::<i64>() {
                stats.pids_current = val;
            }
        }
    }

    // CPU stats from cpu.stat
    let cpu_stat_path = cgroup_path.join("cpu.stat");
    if cpu_stat_path.exists() {
        if let Ok(content) = fs::read_to_string(&cpu_stat_path) {
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

    // IO stats
    let io_stat_path = cgroup_path.join("io.stat");
    if io_stat_path.exists() {
        if let Ok(content) = fs::read_to_string(&io_stat_path) {
            for line in content.lines() {
                let parts: Vec<&str> = line.split_whitespace().collect();
                for part in &parts[1..] {
                    if let Some(val) = part.strip_prefix("rbytes=") {
                        if let Ok(v) = val.parse::<u64>() {
                            stats.io_read_bytes += v;
                        }
                    }
                    if let Some(val) = part.strip_prefix("wbytes=") {
                        if let Ok(v) = val.parse::<u64>() {
                            stats.io_write_bytes += v;
                        }
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

#[derive(Debug, Default)]
pub struct CgroupStats {
    pub memory_current: u64,
    pub memory_limit: u64,
    pub pids_current: i64,
    pub cpu_usage_usec: u64,
    pub cpu_user_usec: u64,
    pub cpu_system_usec: u64,
    pub io_read_bytes: u64,
    pub io_write_bytes: u64,
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
        assert_eq!(stats.cpu_usage_usec, 0);
    }
}
