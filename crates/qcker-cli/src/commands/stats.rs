use clap::Args;
use std::path::Path;

#[derive(Args)]
pub struct StatsArgs {
    pub container_id: Option<String>,

    #[arg(short, long)]
    pub all: bool,
}

pub fn execute(args: StatsArgs, data_dir: &Path, format: &str) -> anyhow::Result<()> {
    let containers_dir = data_dir.join("containers");

    if !containers_dir.exists() {
        if format == "json" {
            println!("[]");
        } else {
            println!("No containers found.");
        }
        return Ok(());
    }

    let mut stats_list = Vec::new();

    if let Some(id) = &args.container_id {
        let state_path = containers_dir.join(id).join("state.json");
        if state_path.exists() {
            let content = std::fs::read_to_string(&state_path)?;
            let state: serde_json::Value = serde_json::from_str(&content)?;
            let status = state["state"].as_str().unwrap_or("unknown");
            let pid = state["pid"].as_i64();

            let (cpu_usage, mem_usage) = if let Some(pid) = pid {
                get_process_stats(pid as i32)
            } else {
                (0, 0)
            };

            stats_list.push((id.clone(), status.to_string(), pid, cpu_usage, mem_usage));
        } else {
            return Err(anyhow::anyhow!("Container not found: {}", id));
        }
    } else if args.all {
        for entry in std::fs::read_dir(&containers_dir)? {
            let entry = entry?;
            let state_path = entry.path().join("state.json");
            if state_path.exists() {
                let content = std::fs::read_to_string(&state_path)?;
                let state: serde_json::Value = serde_json::from_str(&content)?;
                let id = state["id"].as_str().unwrap_or("unknown").to_string();
                let status = state["state"].as_str().unwrap_or("unknown").to_string();
                let pid = state["pid"].as_i64();

                let (cpu_usage, mem_usage) = if let Some(pid) = pid {
                    get_process_stats(pid as i32)
                } else {
                    (0, 0)
                };

                stats_list.push((id, status, pid, cpu_usage, mem_usage));
            }
        }
    } else {
        return Err(anyhow::anyhow!("Specify container ID or use --all"));
    }

    if format == "json" {
        let output: Vec<serde_json::Value> = stats_list
            .iter()
            .map(|(id, status, pid, cpu, mem)| {
                serde_json::json!({
                    "id": id,
                    "status": status,
                    "pid": pid,
                    "cpu_usage_ns": cpu,
                    "memory_usage_bytes": mem,
                })
            })
            .collect();
        println!("{}", serde_json::to_string_pretty(&output)?);
    } else {
        println!("{:<15} {:<12} {:<8} {:<15} {:<15}", "CONTAINER", "STATUS", "PID", "CPU (ns)", "MEMORY (B)");
        for (id, status, pid, cpu, mem) in &stats_list {
            println!(
                "{:<15} {:<12} {:<8} {:<15} {:<15}",
                id,
                status,
                pid.map_or("-".to_string(), |p| p.to_string()),
                cpu,
                mem
            );
        }
    }

    Ok(())
}

fn get_process_stats(pid: i32) -> (u64, u64) {
    let status_path = format!("/proc/{}/status", pid);

    let cpu_usage = 0u64;
    let mut mem_usage = 0u64;

    if let Ok(content) = std::fs::read_to_string(&status_path) {
        for line in content.lines() {
            if line.starts_with("VmRSS:") {
                if let Some(val) = line.split_whitespace().nth(1) {
                    if let Ok(kb) = val.parse::<u64>() {
                        mem_usage = kb * 1024;
                    }
                }
            }
        }
    }

    (cpu_usage, mem_usage)
}
