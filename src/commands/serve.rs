use crate::api::{build_router, RegisterState, SharedState};
use crate::client::{BoxError, ModbusClient};
use chrono::Utc;
use colored::Colorize;
use std::{collections::BTreeMap, sync::Arc};
use tokio::{
    sync::RwLock,
    time::{sleep, Duration},
};

pub async fn run(
    host: &str,
    modbus_port: u16,
    start: u16,
    count: u16,
    interval: f64,
    unit_id: u8,
    api_port: u16,
) -> Result<(), BoxError> {
    let state: SharedState = Arc::new(RwLock::new(RegisterState::default()));
    let state_bg = Arc::clone(&state);
    let host_owned = host.to_string();

    tokio::spawn(async move {
        let duration = Duration::from_secs_f64(interval);
        loop {
            match ModbusClient::connect(&host_owned, modbus_port, unit_id).await {
                Ok(mut client) => loop {
                    match client.poll(start, count).await {
                        Ok(values) => {
                            let mut s = state_bg.write().await;
                            s.registers = values
                                .iter()
                                .map(|(k, v)| (k.to_string(), *v))
                                .collect::<BTreeMap<_, _>>();
                            s.last_updated = Some(Utc::now().to_rfc3339());
                            s.error = None;
                        }
                        Err(e) => {
                            let mut s = state_bg.write().await;
                            s.error = Some(e.to_string());
                            break; // reconnect
                        }
                    }
                    sleep(duration).await;
                },
                Err(e) => {
                    let mut s = state_bg.write().await;
                    s.error = Some(format!("connection failed: {e}"));
                    drop(s);
                    sleep(duration).await;
                }
            }
        }
    });

    let app = build_router(state);
    let bind = format!("0.0.0.0:{api_port}");
    println!(
        "  {} listening on {}",
        "ModBridge".bold().cyan(),
        format!("http://{bind}").underline()
    );
    println!("  Polling {host}:{modbus_port} every {interval}s");
    println!("  Press Ctrl+C to stop\n");

    let listener = tokio::net::TcpListener::bind(&bind).await?;
    axum::serve(listener, app).await?;
    Ok(())
}
