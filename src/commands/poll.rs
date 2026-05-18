use crate::client::{BoxError, ModbusClient};
use crate::display::print_table;
use colored::Colorize;
use tokio::time::{sleep, Duration};

pub async fn run(
    host: &str,
    port: u16,
    start: u16,
    count: u16,
    interval: f64,
    unit_id: u8,
) -> Result<(), BoxError> {
    let duration = Duration::from_secs_f64(interval);
    loop {
        match ModbusClient::connect(host, port, unit_id).await {
            Ok(mut client) => loop {
                match client.poll(start, count).await {
                    Ok(values) => {
                        print!("\x1B[2J\x1B[H");
                        print_table(&values);
                    }
                    Err(e) => {
                        eprintln!("  poll error: {e} -- reconnecting");
                        break;
                    }
                }
                sleep(duration).await;
            },
            Err(e) => {
                eprintln!("  {}: {e} -- retrying in 2s", "connection failed".yellow());
                sleep(Duration::from_secs(2)).await;
            }
        }
    }
}
