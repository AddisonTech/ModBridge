use crate::client::BoxError;
use serde::Deserialize;
use std::path::Path;

#[derive(Deserialize, Debug, Clone, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum Behavior {
    Walk,
    Static,
    Counter,
    Sine,
}

impl Default for Behavior {
    fn default() -> Self {
        Behavior::Walk
    }
}

#[derive(Deserialize, Debug, Clone)]
pub struct RegisterConfig {
    /// Modbus address (e.g. 40001). Must be in the 40001-40100 range to map into the simulator bank.
    pub address: u16,
    /// Starting value.
    pub initial: u16,
    #[serde(default)]
    pub behavior: Behavior,
    /// Max random delta per tick (walk only).
    #[serde(default = "default_delta")]
    pub delta: u16,
    /// Increment per tick (counter only). Negative values count down.
    #[serde(default = "default_step")]
    pub step: i32,
    /// Minimum value (sine only).
    pub min: Option<u16>,
    /// Maximum value (sine only).
    pub max: Option<u16>,
    /// Period in ticks (sine only).
    #[serde(default = "default_period")]
    pub period: u32,
}

fn default_delta() -> u16 { 50 }
fn default_step() -> i32 { 1 }
fn default_period() -> u32 { 60 }

#[derive(Deserialize, Debug)]
pub struct SimConfig {
    pub register: Vec<RegisterConfig>,
}

pub fn load(path: &Path) -> Result<SimConfig, BoxError> {
    let text = std::fs::read_to_string(path)?;
    let config: SimConfig = toml::from_str(&text)?;
    Ok(config)
}
