/// Raw Modbus TCP server implementing function codes 3 and 4 (Read Holding/Input Registers).
/// Does not use the tokio-modbus server API -- implemented directly over tokio TCP
/// for version-independence.
use crate::sim_config::{Behavior, RegisterConfig};
use rand::{rngs::SmallRng, Rng, SeedableRng as _};
use std::collections::HashMap;
use std::f64::consts::PI;
use std::sync::Arc;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpListener;
use tokio::sync::Mutex;
use tokio::time::{sleep, Duration};

pub const NUM_REGISTERS: usize = 100;

pub type SharedRegisters = Arc<Mutex<Vec<u16>>>;

pub fn make_registers() -> SharedRegisters {
    let mut rng = SmallRng::from_entropy();
    let regs: Vec<u16> = (0..NUM_REGISTERS).map(|_| rng.gen_range(1000..5000)).collect();
    Arc::new(Mutex::new(regs))
}

/// Like make_registers but seeds config-specified addresses at their initial values.
pub fn make_registers_with_config(configs: &[RegisterConfig]) -> SharedRegisters {
    let mut rng = SmallRng::from_entropy();
    let mut regs: Vec<u16> = (0..NUM_REGISTERS).map(|_| rng.gen_range(1000..5000)).collect();
    for cfg in configs {
        if let Some(idx) = (cfg.address as usize).checked_sub(40001) {
            if idx < NUM_REGISTERS {
                regs[idx] = cfg.initial;
            }
        }
    }
    Arc::new(Mutex::new(regs))
}

async fn handle_client(mut stream: tokio::net::TcpStream, registers: SharedRegisters) {
    let mut buf = [0u8; 256];
    loop {
        let n = match stream.read(&mut buf).await {
            Ok(0) | Err(_) => break,
            Ok(n) => n,
        };
        if n < 8 {
            continue;
        }

        let trans_id = [buf[0], buf[1]];
        let proto_id = [buf[2], buf[3]];
        let length = u16::from_be_bytes([buf[4], buf[5]]) as usize;
        let unit_id = buf[6];

        if n < 6 + length {
            continue;
        }

        let func_code = buf[7];
        // FC3 = Read Holding Registers, FC4 = Read Input Registers -- same PDU shape.
        // Both are served from the same internal bank for simulator purposes.
        if (func_code == 0x03 || func_code == 0x04) && n >= 12 {
            let start = u16::from_be_bytes([buf[8], buf[9]]) as usize;
            let qty = u16::from_be_bytes([buf[10], buf[11]]) as usize;
            let qty = qty.min(NUM_REGISTERS.saturating_sub(start));

            let regs = registers.lock().await;
            let values: Vec<u16> = regs[start..start + qty].to_vec();
            drop(regs);

            let byte_count = (values.len() * 2) as u8;
            let pdu_len = (2 + values.len() * 2 + 1) as u16;
            let mut response = Vec::with_capacity(6 + pdu_len as usize);
            response.extend_from_slice(&trans_id);
            response.extend_from_slice(&proto_id);
            response.extend_from_slice(&pdu_len.to_be_bytes());
            response.push(unit_id);
            response.push(func_code);
            response.push(byte_count);
            for v in &values {
                response.extend_from_slice(&v.to_be_bytes());
            }
            let _ = stream.write_all(&response).await;
        }
    }
}

pub async fn random_walk_loop(registers: SharedRegisters, interval_secs: f64) {
    let duration = Duration::from_secs_f64(interval_secs);
    let mut rng = SmallRng::from_entropy();
    loop {
        sleep(duration).await;
        let mut regs = registers.lock().await;
        for v in regs.iter_mut() {
            let delta: i32 = rng.gen_range(-50..=50);
            *v = (*v as i32 + delta).clamp(0, 65535) as u16;
        }
    }
}

/// Walk loop that applies per-register behaviors from a config file.
/// Registers not covered by the config continue to random-walk.
pub async fn config_walk_loop(
    registers: SharedRegisters,
    configs: Vec<RegisterConfig>,
    interval_secs: f64,
) {
    let duration = Duration::from_secs_f64(interval_secs);
    let mut rng = SmallRng::from_entropy();
    let mut tick: u64 = 0;

    let config_map: HashMap<usize, RegisterConfig> = configs
        .into_iter()
        .filter_map(|cfg| {
            let idx = (cfg.address as usize).checked_sub(40001)?;
            if idx < NUM_REGISTERS { Some((idx, cfg)) } else { None }
        })
        .collect();

    loop {
        sleep(duration).await;
        tick = tick.wrapping_add(1);
        let mut regs = registers.lock().await;
        for (i, v) in regs.iter_mut().enumerate() {
            if let Some(cfg) = config_map.get(&i) {
                match cfg.behavior {
                    Behavior::Static => {}
                    Behavior::Walk => {
                        let d = cfg.delta as i32;
                        let delta: i32 = rng.gen_range(-d..=d);
                        *v = (*v as i32 + delta).clamp(0, 65535) as u16;
                    }
                    Behavior::Counter => {
                        let new_val = *v as i64 + cfg.step as i64;
                        *v = new_val.rem_euclid(65536) as u16;
                    }
                    Behavior::Sine => {
                        let lo = cfg.min.unwrap_or(0) as f64;
                        let hi = cfg.max.unwrap_or(65535) as f64;
                        let phase = (tick as f64) * 2.0 * PI / cfg.period as f64;
                        *v = (lo + (hi - lo) * (phase.sin() + 1.0) / 2.0).round() as u16;
                    }
                }
            } else {
                let delta: i32 = rng.gen_range(-50..=50);
                *v = (*v as i32 + delta).clamp(0, 65535) as u16;
            }
        }
    }
}

pub async fn run_server(
    host: &str,
    port: u16,
    registers: SharedRegisters,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let listener = TcpListener::bind(format!("{host}:{port}")).await?;
    loop {
        let (stream, _) = listener.accept().await?;
        let regs = Arc::clone(&registers);
        tokio::spawn(async move {
            handle_client(stream, regs).await;
        });
    }
}
