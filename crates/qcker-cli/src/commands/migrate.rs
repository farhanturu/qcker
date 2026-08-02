use clap::Args;
use std::path::Path;
use std::process::Command;

#[derive(Args)]
pub struct MigrateArgs {
    pub container_id: String,

    #[arg(long, help = "New container name in Qcker")]
    pub name: Option<String>,

    #[arg(long, help = "Output format: text or json")]
    pub format: Option<String>,
}

pub fn execute(args: MigrateArgs, _data_dir: &Path, format: &str) -> anyhow::Result<()> {
    let new_name = args.name.clone().unwrap_or_else(|| format!("migrated-{}", args.container_id));
    let output_format = args.format.clone().unwrap_or_else(|| format.to_string());

    println!("Starting migration from Docker to Qcker...");
    println!("Source container: {}", args.container_id);
    println!("Target container: {}", new_name);
    println!("");

    if Command::new("docker")
        .arg("ps")
        .output()
        .map(|o| !o.status.success())
        .unwrap_or(true)
    {
        eprintln!("Error: Docker is not installed or not running");
        std::process::exit(1);
    }

    println!("Step 1: Inspecting Docker container...");
    let inspect_output = Command::new("docker")
        .args(&["inspect", &args.container_id])
        .output()?;

    if !inspect_output.status.success() {
        eprintln!("Error: Failed to inspect container '{}'", args.container_id);
        std::process::exit(1);
    }

    let inspect_json = String::from_utf8_lossy(&inspect_output.stdout);
    println!("✓ Container inspected successfully");
    println!("");

    println!("Step 2: Extracting container details...");
    let details = extract_docker_details(&inspect_json);

    println!("  Image: {}", details.image);
    println!("  Command: {}", details.command);
    println!("  Environment variables: {} items", details.env_vars.len());
    println!("  Mounts: {} items", details.mounts.len());
    println!("  Port mappings: {} items", details.ports.len());
    println!("  Hostname: {}", details.hostname);
    println!("");

    println!("Step 3: Generating Qcker run command...");
    let mut qcker_cmd = vec!["run".to_string()];

    if let Some(name) = &args.name {
        qcker_cmd.push("--name".to_string());
        qcker_cmd.push(name.clone());
    }

    for env in &details.env_vars {
        qcker_cmd.push("-e".to_string());
        qcker_cmd.push(env.clone());
    }

    for mount in &details.mounts {
        qcker_cmd.push("-v".to_string());
        qcker_cmd.push(mount.clone());
    }

    for port in &details.ports {
        qcker_cmd.push("-p".to_string());
        qcker_cmd.push(port.clone());
    }

    if !details.hostname.is_empty() {
        qcker_cmd.push("--hostname".to_string());
        qcker_cmd.push(details.hostname.clone());
    }

    qcker_cmd.push(details.image.clone());

    if !details.command.is_empty() {
        qcker_cmd.extend(details.command.split_whitespace().map(|s| s.to_string()));
    } else {
        qcker_cmd.push("/bin/sh".to_string());
    }

    println!("Generated command:");
    println!("  qcker {}", qcker_cmd.join(" "));
    println!("");

    println!("Step 4: Migration instructions...");
    println!("");
    println!("To complete the migration:");
    println!("1. Create rootfs from image:");
    println!("   qcker pull {}", details.image);
    println!("");
    println!("2. Run with generated command above");
    println!("");
    println!("3. Verify migration:");
    println!("   qcker ps");
    println!("   qcker logs {}", new_name);
    println!("");

    if output_format == "json" {
        let instructions = vec![
            format!("Run: qcker pull {}", details.image),
            format!("Run: qcker {}", qcker_cmd[1..].join(" ")),
            "Verify: qcker ps".to_string(),
            format!("Check logs: qcker logs {}", new_name),
        ];
        let output = serde_json::json!({
            "status": "success",
            "source_container": args.container_id,
            "target_container": new_name,
            "image": details.image,
            "command": details.command,
            "environment_variables": details.env_vars,
            "mounts": details.mounts,
            "ports": details.ports,
            "hostname": details.hostname,
            "qcker_command": qcker_cmd.join(" "),
            "instructions": instructions,
        });
        println!("{}", serde_json::to_string_pretty(&output)?);
    }

    println!("Migration analysis complete!");
    Ok(())
}

#[derive(Debug)]
struct DockerDetails {
    image: String,
    command: String,
    env_vars: Vec<String>,
    mounts: Vec<String>,
    ports: Vec<String>,
    hostname: String,
}

fn extract_docker_details(json: &str) -> DockerDetails {
    use std::io::Write;

    let python_script = r#"
import json
import sys

data = json.load(sys.stdin)
if not data:
    sys.exit(0)

d = data[0]
config = d.get('Config', {})
host_config = d.get('HostConfig', {})

print("IMAGE:" + config.get('Image', ''))

cmd = config.get('Cmd', ['/bin/sh'])
if cmd and cmd[0] == '':
    cmd = ['/bin/sh']
print("CMD:" + ' '.join(cmd))

print("HOSTNAME:" + config.get('Hostname', ''))

for env in config.get('Env', []):
    print("ENV:" + env)

for m in d.get('Mounts', []):
    ro = 'ro' if m.get('ReadOnly', False) else 'rw'
    dest = m.get('Destination', '')
    source = m.get('HostPath', m.get('Source', ''))
    print("MOUNT:" + dest + ":" + source + ":" + ro)

for host_port, bindings in host_config.get('PortBindings', {}).items():
    if bindings and len(bindings) > 0:
        container_port = host_port.split('/')[0]
        print("PORT:" + bindings[0].get('HostPort', '') + ":" + container_port)
"#;

    let mut details = DockerDetails {
        image: String::new(),
        command: String::new(),
        env_vars: Vec::new(),
        mounts: Vec::new(),
        ports: Vec::new(),
        hostname: String::new(),
    };

    let mut child = match Command::new("python3")
        .arg("-c")
        .arg(python_script)
        .stdin(std::process::Stdio::piped())
        .stdout(std::process::Stdio::piped())
        .spawn()
    {
        Ok(c) => c,
        Err(_) => return details,
    };

    if let Some(mut stdin) = child.stdin.take() {
        let _ = stdin.write_all(json.as_bytes());
    }

    if let Ok(output) = child.wait_with_output() {
        let result = String::from_utf8_lossy(&output.stdout);
        for line in result.lines() {
            if line.starts_with("IMAGE:") {
                details.image = line[6..].to_string();
            } else if line.starts_with("CMD:") {
                details.command = line[4..].to_string();
            } else if line.starts_with("HOSTNAME:") {
                details.hostname = line[9..].to_string();
            } else if line.starts_with("ENV:") {
                details.env_vars.push(line[4..].to_string());
            } else if line.starts_with("MOUNT:") {
                details.mounts.push(line[6..].to_string());
            } else if line.starts_with("PORT:") {
                details.ports.push(line[5..].to_string());
            }
        }
    }

    details
}
