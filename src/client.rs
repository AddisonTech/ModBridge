use std::net::SocketAddr;
use tokio_modbus::client::{tcp, Context};
use tokio_modbus::prelude::*;

pub type BoxError = Box<dyn std::error::Error + Send + Sync>;

/// Parse "40001-40010" into (40001, 40010).
pub fn parse_register_range(spec: &str) -> Result<(u16, u16), BoxError> {
    let parts: Vec<&str> = spec.splitn(2, '-').collect();
    if parts.len() != 2 {
        return Err(format!("invalid register range {spec:?}, expected format: 40001-40010").into());
    }
    let start: u16 = parts[0].parse()?;
    let end: u16 = parts[1].parse()?;
    if start < 40001 || start > end {
        return Err("register range must start at >= 40001 with start <= end".into());
    }
    Ok((start, end))
}

pub struct ModbusClient {
    ctx: Context,
}

impl ModbusClient {
    pub async fn connect(host: &str, port: u16, unit_id: u8) -> Result<Self, BoxError> {
        let addr: SocketAddr = format!("{host}:{port}").parse()?;
        let ctx = tcp::connect_slave(addr, Slave(unit_id)).await?;
        Ok(Self { ctx })
    }

    /// Read holding registers. `start_register` uses Modbus numbering (e.g. 40001).
    /// Returns a Vec of (register_number, value) pairs.
    pub async fn poll(&mut self, start_register: u16, count: u16) -> Result<Vec<(u16, u16)>, BoxError> {
        let address = start_register - 40001;
        let regs = match self.ctx.read_holding_registers(address, count).await {
            Ok(Ok(v)) => v,
            Ok(Err(e)) => return Err(format!("Modbus exception: {e:?}").into()),
            Err(e) => return Err(e.into()),
        };
        Ok(regs
            .into_iter()
            .enumerate()
            .map(|(i, v)| (start_register + i as u16, v))
            .collect())
    }
}
