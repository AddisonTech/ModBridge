use crate::client::BoxError;
use crate::simulator::{
    config_walk_loop, make_registers, make_registers_with_config, random_walk_loop, run_server,
};
use colored::Colorize;
use std::path::PathBuf;

pub async fn run(host: &str, port: u16, interval: f64, config: Option<PathBuf>) -> Result<(), BoxError> {
    let (registers, desc) = if let Some(ref path) = config {
        let cfg = crate::sim_config::load(path)?;
        let n = cfg.register.len();
        let desc = format!("{n} configured registers from {}", path.display());
        let regs = make_registers_with_config(&cfg.register);
        let regs_walk = regs.clone();
        tokio::spawn(async move {
            config_walk_loop(regs_walk, cfg.register, interval).await;
        });
        (regs, desc)
    } else {
        let regs = make_registers();
        let regs_walk = regs.clone();
        tokio::spawn(async move {
            random_walk_loop(regs_walk, interval).await;
        });
        (regs, "100 registers (40001-40100) with random-walking values".to_string())
    };

    println!(
        "  {} simulator on {}:{}",
        "ModBridge".bold().cyan(),
        host,
        port
    );
    println!("  Serving {desc}");
    println!("  Press Ctrl+C to stop\n");

    run_server(host, port, registers).await
}
