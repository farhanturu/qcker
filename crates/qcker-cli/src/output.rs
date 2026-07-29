/// Output container state
pub fn print_container_state(id: &str, state: &str, pid: Option<i32>, format: &str) {
    match format {
        "json" => {
            let output = serde_json::json!({
                "id": id,
                "status": state,
                "pid": pid,
            });
            println!("{}", serde_json::to_string_pretty(&output).unwrap());
        }
        _ => {
            println!("Container: {}", id);
            println!("Status:    {}", state);
            if let Some(pid) = pid {
                println!("PID:       {}", pid);
            }
        }
    }
}

/// Output container list
pub fn print_container_list(containers: &[(String, String, Option<i32>)], format: &str) {
    match format {
        "json" => {
            let output: Vec<serde_json::Value> = containers
                .iter()
                .map(|(id, state, pid)| {
                    serde_json::json!({
                        "id": id,
                        "status": state,
                        "pid": pid,
                    })
                })
                .collect();
            println!("{}", serde_json::to_string_pretty(&output).unwrap());
        }
        _ => {
            println!("{:<15} {:<15} {:<10}", "CONTAINER ID", "STATUS", "PID");
            for (id, state, pid) in containers {
                println!(
                    "{:<15} {:<15} {:<10}",
                    id,
                    state,
                    pid.map_or("-".to_string(), |p| p.to_string())
                );
            }
        }
    }
}

/// Print success message
pub fn print_success(message: &str) {
    println!("{}", message);
}

/// Print error message
pub fn print_error(message: &str) {
    eprintln!("Error: {}", message);
}
