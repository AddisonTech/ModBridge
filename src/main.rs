mod api;
mod client;
mod commands;
mod display;
mod logger;
mod simulator;

use clap::{Parser, Subcommand};
use client::parse_register_range;
use colored::Colorize;

#[derive(Parser)]
#[command(name = "modbridge", about = "Modbus TCP utility -- poll, serve, log, or simulate")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Live terminal display of register values
    Poll {
        /// Modbus device IP
        #[arg(long)]
        host: String,
        /// Modbus TCP port
        #[arg(long, default_value = "502")]
        port: u16,
        /// Register range, e.g. 40001-40010
        #[arg(long)]
        registers: String,
        /// Poll interval in seconds
        #[arg(long, default_value = "1.0")]
        interval: f64,
        /// Modbus unit/slave ID
        #[arg(long, default_value = "1")]
        unit_id: u8,
    },

    /// REST API server exposing live register values
    Serve {
        /// Modbus device IP
        #[arg(long)]
        host: String,
        /// Modbus TCP port
        #[arg(long, default_value = "502")]
        modbus_port: u16,
        /// Register range, e.g. 40001-40100
        #[arg(long)]
        registers: String,
        /// HTTP API port
        #[arg(long, default_value = "3000")]
        port: u16,
        /// Poll interval in seconds
        #[arg(long, default_value = "1.0")]
        interval: f64,
        /// Modbus unit/slave ID
        #[arg(long, default_value = "1")]
        unit_id: u8,
    },

    /// Log register values to a CSV file
    Log {
        /// Modbus device IP
        #[arg(long)]
        host: String,
        /// Modbus TCP port
        #[arg(long, default_value = "502")]
        port: u16,
        /// Register range, e.g. 40001-40010
        #[arg(long)]
        registers: String,
        /// Output CSV file path
        #[arg(long)]
        output: String,
        /// Poll interval in seconds
        #[arg(long, default_value = "1.0")]
        interval: f64,
        /// Modbus unit/slave ID
        #[arg(long, default_value = "1")]
        unit_id: u8,
    },

    /// Run a fake Modbus TCP server with random-walking register values
    Simulate {
        /// Bind address
        #[arg(long, default_value = "localhost")]
        host: String,
        /// Bind port
        #[arg(long, default_value = "502")]
        port: u16,
        /// Random-walk interval in seconds
        #[arg(long, default_value = "1.0")]
        interval: f64,
    },
}

#[tokio::main]
async fn main() {
    let cli = Cli::parse();
    let result = run(cli).await;
    if let Err(e) = result {
        eprintln!("\n  {}: {}", "error".red().bold(), e);
        std::process::exit(1);
    }
}

async fn run(cli: Cli) -> Result<(), client::BoxError> {
    match cli.command {
        Commands::Poll {
            host,
            port,
            registers,
            interval,
            unit_id,
        } => {
            let (start, end) = parse_register_range(&registers)?;
            let count = end - start + 1;
            commands::poll::run(&host, port, start, count, interval, unit_id).await
        }

        Commands::Serve {
            host,
            modbus_port,
            registers,
            port,
            interval,
            unit_id,
        } => {
            let (start, end) = parse_register_range(&registers)?;
            let count = end - start + 1;
            commands::serve::run(&host, modbus_port, start, count, interval, unit_id, port).await
        }

        Commands::Log {
            host,
            port,
            registers,
            output,
            interval,
            unit_id,
        } => {
            let (start, end) = parse_register_range(&registers)?;
            let count = end - start + 1;
            commands::log::run(&host, port, start, count, interval, unit_id, &output).await
        }

        Commands::Simulate {
            host,
            port,
            interval,
        } => commands::simulate::run(&host, port, interval).await,
    }
}
