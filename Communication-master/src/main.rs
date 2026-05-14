mod crypto;
mod network;
mod drone;
mod device;
pub const MAX_DEVICES: usize = 10;
use clap::{Parser, Subcommand};
use colored::*;
use anyhow::Result;

#[derive(Parser)]
#[command(name = "drone_mesh")]
#[command(about = "Secure Drone Mesh Communication System", long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Run as central drone (Raspberry Pi)
    Drone {
        #[arg(short, long, default_value = "0.0.0.0:8888")]
        addr: String,
    },
    /// Run as client device (laptop, phone, tablet)
    Device {
        #[arg(short, long)]
        id: String,
        #[arg(short, long)]
        name: String,
        #[arg(short, long, default_value = "192.168.1.100:8888")]
        drone_addr: String,
    },
}

#[tokio::main]
async fn main() -> Result<()> {
    env_logger::init();
    
    let cli = Cli::parse();
    
    println!("{}", "=".repeat(60).bright_cyan());
    println!("{}", "🔒 DRONE MESH SECURE COMMUNICATION SYSTEM".bold().bright_cyan());
    println!("{}", format!("v{}", env!("CARGO_PKG_VERSION")).dimmed());
    println!("{}", "=".repeat(60).bright_cyan());
    
    match cli.command {
        Commands::Drone { addr } => {
            let mut drone = drone::CentralDrone::new();
            drone.initialize().await?;
            drone.start(&addr).await?;
        }
        Commands::Device { id, name, drone_addr } => {
            let mut device = device::ClientDevice::new(&id, &name);
            device.initialize().await?;
            device.connect_to_drone(&drone_addr).await?;
        }
    }
    
    Ok(())
}
