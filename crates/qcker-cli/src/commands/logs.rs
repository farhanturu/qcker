use clap::Args;
use std::path::Path;

use crate::output;

#[derive(Args)]
pub struct LogsArgs {
    pub container_id: String,

    #[arg(short, long)]
    pub follow: bool,

    #[arg(short, long)]
    pub tail: Option<usize>,

    #[arg(long)]
    pub since: Option<String>,

    #[arg(short, long)]
    pub timestamps: bool,
}

pub fn execute(args: LogsArgs, data_dir: &Path, format: &str) -> anyhow::Result<()> {
    let log_path = data_dir
        .join("containers")
        .join(&args.container_id)
        .join("container.log");

    if !log_path.exists() {
        if format == "json" {
            let output = serde_json::json!({
                "container": args.container_id,
                "logs": [],
            });
            println!("{}", serde_json::to_string_pretty(&output)?);
        } else {
            println!("No logs available for container {}", args.container_id);
        }
        return Ok(());
    }

    let content = std::fs::read_to_string(&log_path)?;
    let lines: Vec<&str> = content.lines().collect();

    let display_lines = if let Some(tail) = args.tail {
        let start = if lines.len() > tail { lines.len() - tail } else { 0 };
        &lines[start..]
    } else {
        &lines
    };

    if format == "json" {
        let log_entries: Vec<serde_json::Value> = display_lines
            .iter()
            .map(|line| {
                serde_json::json!({
                    "log": line,
                })
            })
            .collect();
        let output = serde_json::json!({
            "container": args.container_id,
            "logs": log_entries,
        });
        println!("{}", serde_json::to_string_pretty(&output)?);
    } else {
        for line in display_lines {
            if args.timestamps {
                // Add timestamp prefix
                println!("[{}] {}", chrono::Local::now().format("%Y-%m-%d %H:%M:%S"), line);
            } else {
                println!("{}", line);
            }
        }
    }

    Ok(())
}
