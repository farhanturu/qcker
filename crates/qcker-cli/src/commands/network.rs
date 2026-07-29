use clap::{Args, Subcommand};
use std::path::Path;

use qcker_engine::network::manager::NetworkManager;
use qcker_engine::network::types::NetworkConfig;

/// Manage networks
#[derive(Args)]
pub struct NetworkArgs {
    #[command(subcommand)]
    command: NetworkCommand,
}

#[derive(Subcommand)]
pub enum NetworkCommand {
    /// Create a network
    Create(CreateNetworkArgs),
    /// Remove a network
    Rm(RmNetworkArgs),
    /// List networks
    Ls,
    /// Inspect a network
    Inspect(InspectNetworkArgs),
}

/// Create a network
#[derive(Args)]
pub struct CreateNetworkArgs {
    /// Network name
    name: String,

    /// Network driver
    #[arg(short, long, default_value = "bridge")]
    driver: String,

    /// Subnet (e.g., 172.20.0.0/16)
    #[arg(long)]
    subnet: Option<String>,
}

/// Remove a network
#[derive(Args)]
pub struct RmNetworkArgs {
    /// Network name or ID
    name: String,
}

/// Inspect a network
#[derive(Args)]
pub struct InspectNetworkArgs {
    /// Network name or ID
    name: String,
}

pub fn execute(args: NetworkArgs, data_dir: &Path, format: &str) -> anyhow::Result<()> {
    match args.command {
        NetworkCommand::Create(args) => create_network(args, data_dir, format),
        NetworkCommand::Rm(args) => remove_network(args, data_dir, format),
        NetworkCommand::Ls => list_networks(data_dir, format),
        NetworkCommand::Inspect(args) => inspect_network(args, data_dir, format),
    }
}

fn create_network(args: CreateNetworkArgs, data_dir: &Path, format: &str) -> anyhow::Result<()> {
    let mut manager = NetworkManager::new(data_dir.to_path_buf());
    manager.init()?;

    let config = match args.driver.as_str() {
        "bridge" => {
            let subnet = args.subnet.unwrap_or_else(|| "172.20.0.0/16".to_string());
            NetworkConfig::new_bridge(&args.name, &subnet)
        }
        "host" => NetworkConfig::new_host(),
        "none" => NetworkConfig::new_none(),
        _ => return Err(anyhow::anyhow!("Unknown driver: {}", args.driver)),
    };

    let network_id = config.id.clone();
    manager.create_network(config)?;

    if format == "json" {
        let output = serde_json::json!({
            "id": network_id,
            "name": args.name,
        });
        println!("{}", serde_json::to_string_pretty(&output)?);
    } else {
        println!("Network {} created ({})", args.name, &network_id[..12]);
    }

    Ok(())
}

fn remove_network(args: RmNetworkArgs, data_dir: &Path, format: &str) -> anyhow::Result<()> {
    let mut manager = NetworkManager::new(data_dir.to_path_buf());
    manager.init()?;

    manager.remove_network(&args.name)?;

    if format == "json" {
        let output = serde_json::json!({
            "name": args.name,
            "removed": true,
        });
        println!("{}", serde_json::to_string_pretty(&output)?);
    } else {
        println!("Network {} removed", args.name);
    }

    Ok(())
}

fn list_networks(data_dir: &Path, format: &str) -> anyhow::Result<()> {
    let mut manager = NetworkManager::new(data_dir.to_path_buf());
    manager.init()?;

    let networks = manager.list_networks();

    if format == "json" {
        let output: Vec<serde_json::Value> = networks
            .iter()
            .map(|n| {
                serde_json::json!({
                    "id": n.id,
                    "name": n.name,
                    "driver": format!("{:?}", n.driver).to_lowercase(),
                    "subnet": n.subnet,
                    "gateway": n.gateway,
                })
            })
            .collect();
        println!("{}", serde_json::to_string_pretty(&output)?);
    } else {
        println!("{:<15} {:<15} {:<15} {:<20}", "NETWORK ID", "NAME", "DRIVER", "SUBNET");
        for n in &networks {
            println!(
                "{:<15} {:<15} {:<15} {:<20}",
                &n.id[..12],
                n.name,
                format!("{:?}", n.driver).to_lowercase(),
                n.subnet.as_deref().unwrap_or("-")
            );
        }
    }

    Ok(())
}

fn inspect_network(args: InspectNetworkArgs, data_dir: &Path, format: &str) -> anyhow::Result<()> {
    let mut manager = NetworkManager::new(data_dir.to_path_buf());
    manager.init()?;

    let config = manager.get_network(&args.name)?;

    if format == "json" {
        println!("{}", serde_json::to_string_pretty(config)?);
    } else {
        println!("Network: {}", config.name);
        println!("ID:      {}", config.id);
        println!("Driver:  {:?}", config.driver);
        if let Some(ref subnet) = config.subnet {
            println!("Subnet:  {}", subnet);
        }
        if let Some(ref gateway) = config.gateway {
            println!("Gateway: {}", gateway);
        }
    }

    Ok(())
}
