use thiserror::Error;

#[derive(Debug, Error)]
pub enum ClixorError {
    #[error("failed to install hook: {0}")]
    HookInstall(String),
    #[error("config error: {0}")]
    Config(#[from] ConfigError),
}

#[derive(Debug, Error)]
pub enum ConfigError {
    #[error("file read error: {0}")]
    Io(#[from] std::io::Error),
    #[error("JSON parse error: {0}")]
    Json(#[from] serde_json::Error),
    #[error("lmb_cps / rmb_cps must be in [1..=1000], got {value}")]
    InvalidCps { value: f64 },
    #[error("ramp_start_ratio must be in [0.01..=1.0], got {value}")]
    InvalidRampRatio { value: f64 },
    #[error("unknown toggle key '{0}'")]
    UnknownKey(String),
    #[error("sleep_undershoot_us must be less than spin_threshold_us")]
    InvalidTimingParams,
}
