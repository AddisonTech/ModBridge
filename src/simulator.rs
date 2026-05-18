/// Raw Modbus TCP server implementing function code 3 (Read Holding Registers).
/// Does not use the tokio-modbus server API -- implemented directly over tokio TCP
/// for version-independence.
use rand::{rngs::SmallRng, Rng, SeedableRng as _};
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

async fn handle_client(
    mut stream: tokio::net::TcpStream,
    registers: SharedRegisters,
) {
    let mut buf = [0u8; 256];
    loop {
        // Read MBAP header (7 bytes)
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

        // Read remaining bytes if the initial read was incomplete
        if n < 6 + length {
            continue;
        }

        let func_code = buf[7];
        if func_code == 0x03 && n >= 12 {
            let start = u16::from_be_bytes([buf[8], buf[9]]) as usize;
            let qty = u16::from_be_bytes([buf[10], buf[11]]) as usize;
            let qty = qty.min(NUM_REGISTERS.saturating_sub(start));

            let regs = registers.lock().await;
            let values: Vec<u16> = regs[start..start + qty].to_vec();
            drop(regs);

            let byte_count = (values.len() * 2) as u8;
            let pdu_len = (2 + values.len() * 2 + 1) as u16; // unit_id + func + byte_count + data
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
