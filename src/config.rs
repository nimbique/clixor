use std::{fs, io, num::NonZeroU64};

use serde::{Deserialize, Serialize};

use crate::error::ConfigError;

const CONFIG_PATH: &str = "config.json";
const CPS_MIN: f64 = 1.0;
const CPS_MAX: f64 = 1_000.0;
const RAMP_RATIO_MIN: f64 = 0.01;
const RAMP_RATIO_MAX: f64 = 1.0;

#[derive(Serialize, Deserialize)]
#[serde(default)]
pub struct Config {
    pub lmb_cps: f64,
    pub rmb_cps: f64,
    pub lmb_toggle_key: String,
    pub rmb_toggle_key: String,
    pub ramp_duration_ms: NonZeroU64,
    pub ramp_start_ratio: f64,
    pub spin_threshold_us: u64,
    pub sleep_undershoot_us: u64,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            lmb_cps: 10.0,
            rmb_cps: 10.0,
            lmb_toggle_key: "F6".into(),
            rmb_toggle_key: "F7".into(),
            ramp_duration_ms: NonZeroU64::new(300).unwrap(),
            ramp_start_ratio: 0.20,
            spin_threshold_us: 1_500,
            sleep_undershoot_us: 600,
        }
    }
}

impl Config {
    fn validate(&self) -> Result<(), ConfigError> {
        for &cps in &[self.lmb_cps, self.rmb_cps] {
            if !(CPS_MIN..=CPS_MAX).contains(&cps) {
                return Err(ConfigError::InvalidCps { value: cps });
            }
        }
        if !(RAMP_RATIO_MIN..=RAMP_RATIO_MAX).contains(&self.ramp_start_ratio) {
            return Err(ConfigError::InvalidRampRatio { value: self.ramp_start_ratio });
        }
        if self.sleep_undershoot_us >= self.spin_threshold_us {
            return Err(ConfigError::InvalidTimingParams);
        }
        Ok(())
    }
}

pub fn load() -> Config {
    read_config().unwrap_or_else(|e| {
        if matches!(&e, ConfigError::Io(io_e) if io_e.kind() == io::ErrorKind::NotFound) {
            let default = Config::default();
            if let Ok(s) = serde_json::to_string_pretty(&default) {
                if fs::write(CONFIG_PATH, &s).is_ok() {
                    println!("  created config.json with default values");
                }
            }
            return default;
        }
        eprintln!("  config: {e} — using defaults");
        Config::default()
    })
}

fn read_config() -> Result<Config, ConfigError> {
    let text = fs::read_to_string(CONFIG_PATH)?;
    let cfg: Config = serde_json::from_str(&text)?;
    cfg.validate()?;
    Ok(cfg)
}
