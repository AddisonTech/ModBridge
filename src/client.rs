use std::net::SocketAddr;
use tokio_modbus::client::{tcp, Context};
use tokio_modbus::prelude::*;

pub type BoxError = Box<dyn std::error::Error + Send + Sync>;

/// Parse "40001-40010" or "30001-30010" into (start, end).
/// 40001-49999 = holding registers (FC3), 30001-39999 = input registers (FC4).
pub fn parse_register_range(spec: &str) -> Result<(u16, u16), BoxError> {
    let parts: Vec<&str> = spec.splitn(2, '-').collect();
    if parts.len() != 2 {
        return Err(format!("invalid register range {spec:?}, expected format: 40001-40010").into());
    }
    let start: u16 = parts[0].parse()?;
    let end: u16 = parts[1].parse()?;
    if start > end {
        return Err("register range: start must be <= end".into());
    }
    let valid_fc4 = (30001..=39999).contains(&start) && (30001..=39999).contains(&end);
    let valid_fc3 = (40001..=49999).contains(&start) && (40001..=49999).contains(&end);
    if !valid_fc4 && !valid_fc3 {
        return Err(
            "register range must be 30001-39999 (input registers, FC4) or 40001-49999 (holding registers, FC3)".into(),
        );
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

    /// Read registers. start_register uses Modbus numbering (30001+ = FC4, 40001+ = FC3).
    /// Returns a Vec of (register_number, value) pairs.
    pub async fn poll(&mut self, start_register: u16, count: u16) -> Result<Vec<(u16, u16)>, BoxError> {
        let regs = if start_register >= 30001 && start_register <= 39999 {
            let address = start_register - 30001;
            match self.ctx.read_input_registers(address, count).await {
                Ok(Ok(v)) => v,
                Ok(Err(e)) => return Err(format!("Modbus exception: {e:?}").into()),
                Err(e) => return Err(e.into()),
            }
        } else {
            let address = start_register - 40001;
            match self.ctx.read_holding_registers(address, count).await {
                Ok(Ok(v)) => v,
                Ok(Err(e)) => return Err(format!("Modbus exception: {e:?}").into()),
                Err(e) => return Err(e.into()),
            }
        };
        Ok(regs
            .into_iter()
            .enumerate()
            .map(|(i, v)| (start_register + i as u16, v))
            .collect())
    }
}
