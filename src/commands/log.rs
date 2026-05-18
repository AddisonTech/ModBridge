use crate::client::{BoxError, ModbusClient};
use crate::logger::write_row;
use colored::Colorize;
use tokio::time::{sleep, Duration};

pub async fn run(
    host: &str,
    port: u16,
    start: u16,
    count: u16,
    interval: f64,
    unit_id: u8,
    output: &str,
) -> Result<(), BoxError> {
    let duration = Duration::from_secs_f64(interval);
    let mut rows: u64 = 0;

    println!("  Logging {count} registers to {output}");
    println!("  Press Ctrl+C to stop\n");

    loop {
        match ModbusClient::connect(host, port, unit_id).await {
            Ok(mut client) => loop {
                match client.poll(start, count).await {
                    Ok(values) => {
                        write_row(output, &values)?;
                        rows += 1;
                        print!("\r  Rows written: {rows}");
                        use std::io::Write;
                        std::io::stdout().flush().ok();
                    }
                    Err(e) => {
                        eprintln!("\n  poll error: {e} -- reconnecting");
                        break;
                    }
                }
                sleep(duration).await;
            },
            Err(e) => {
                eprintln!("\n  {}: {e} -- retrying in 2s", "connection failed".yellow());
                sleep(Duration::from_secs(2)).await;
            }
        }
    }
}
