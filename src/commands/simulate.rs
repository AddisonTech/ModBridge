use crate::client::BoxError;
use crate::simulator::{make_registers, random_walk_loop, run_server};
use colored::Colorize;

pub async fn run(host: &str, port: u16, interval: f64) -> Result<(), BoxError> {
    let registers = make_registers();

    let regs_walk = registers.clone();
    tokio::spawn(async move {
        random_walk_loop(regs_walk, interval).await;
    });

    println!(
        "  {} simulator on {}:{}",
        "ModBridge".bold().cyan(),
        host,
        port
    );
    println!("  Serving 100 registers (40001-40100) with random-walking values");
    println!("  Press Ctrl+C to stop\n");

    run_server(host, port, registers).await
}
