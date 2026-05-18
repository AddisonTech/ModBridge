use crate::client::{BoxError, ModbusClient};
use crate::logger::write_row;
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
    let mut client = ModbusClient::connect(host, port, unit_id).await?;
    let duration = Duration::from_secs_f64(interval);
    let mut rows: u64 = 0;

    println!("  Logging {count} registers to {output}");
    println!("  Press Ctrl+C to stop\n");

    loop {
        match client.poll(start, count).await {
            Ok(values) => {
                write_row(output, &values)?;
                rows += 1;
                print!("\r  Rows written: {rows}");
                use std::io::Write;
                std::io::stdout().flush().ok();
            }
            Err(e) => {
                eprintln!("\n  poll error: {e}");
            }
        }
        sleep(duration).await;
    }
}
