use crate::client::{BoxError, ModbusClient};
use crate::display::print_table;
use tokio::time::{sleep, Duration};

pub async fn run(
    host: &str,
    port: u16,
    start: u16,
    count: u16,
    interval: f64,
    unit_id: u8,
) -> Result<(), BoxError> {
    let mut client = ModbusClient::connect(host, port, unit_id).await?;
    let duration = Duration::from_secs_f64(interval);
    loop {
        match client.poll(start, count).await {
            Ok(values) => {
                // Clear terminal and reprint table
                print!("\x1B[2J\x1B[H");
                print_table(&values);
            }
            Err(e) => {
                eprintln!("  poll error: {e}");
            }
        }
        sleep(duration).await;
    }
}
