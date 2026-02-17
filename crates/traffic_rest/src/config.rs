use std::{fmt, fs, path::Path};

use serde::Deserialize;

use crate::model::{KeyMode, TrafficProfile};

/// REST traffic generator configuration.
///
/// Loaded from a TOML file via `--config <path>`.
/// If no config is provided, defaults are used.
#[derive(Debug, Clone, Deserialize)]
pub struct TrafficRunConfig {
    /// Address to target with generated traffic, e.g. "http://127.0.0.1:3000/".
    pub target_url: String,

    /// Duration of the traffic run in seconds.
    pub duration_seconds: u64,

    /// Number of requests to send per second.
    pub requests_per_second: u64,

    /// Number of concurrent tasks sending requests.
    pub concurrency: usize,

    /// Header used for key extraction, e.g. "x-api-key".
    pub key_header: String,

    /// Key extraction policy.
    pub key_mode: KeyModeConfig,
}

/// Key extraction policy configuration.
///
/// The `mode` field determines which of the other fields are used:
/// - `key` is used when `mode = single_key`.
/// - `keys` is used when `mode = round_robin`.
#[derive(Debug, Clone, Deserialize)]
pub struct KeyModeConfig {
    /// Key extraction mode.
    pub mode: KeyModeKind,

    /// Key used when `mode = single_key`.
    #[serde(default)]
    pub key: Option<String>,

    /// Keys used when `mode = round_robin`.
    #[serde(default)]
    pub keys: Vec<String>,
}

/// Key extraction mode.
#[derive(Debug, Clone, Copy, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum KeyModeKind {
    Keyless,
    SingleKey,
    RoundRobin,
}

#[derive(Debug)]
pub enum ConfigError {
    Io(std::io::Error),
    ParseToml(toml::de::Error),
    InvalidValue(&'static str),
}

impl fmt::Display for ConfigError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ConfigError::Io(err) => write!(f, "failed to read config file: {err}"),
            ConfigError::ParseToml(err) => write!(f, "failed to parse TOML config: {err}"),
            ConfigError::InvalidValue(msg) => write!(f, "invalid traffic config: {msg}"),
        }
    }
}

impl std::error::Error for ConfigError {}

impl Default for TrafficRunConfig {
    fn default() -> Self {
        Self {
            target_url: "http://127.0.0.1:3000/".to_string(),
            duration_seconds: 5,
            requests_per_second: 60_000,
            concurrency: 16,
            key_header: "x-api-key".to_string(),
            key_mode: KeyModeConfig {
                mode: KeyModeKind::SingleKey,
                key: Some("user1".to_string()),
                keys: Vec::new(),
            },
        }
    }
}

impl TrafficRunConfig {
    pub fn load(path: Option<&Path>) -> Result<Self, ConfigError> {
        let config = match path {
            Some(path) => {
                let text = fs::read_to_string(path).map_err(ConfigError::Io)?;
                toml::from_str::<TrafficRunConfig>(&text).map_err(ConfigError::ParseToml)?
            }
            None => TrafficRunConfig::default(),
        };

        config.validate()?;
        Ok(config)
    }

    pub fn to_profile_and_mode(self) -> Result<(TrafficProfile, KeyMode), ConfigError> {
        self.validate()?;

        let profile = TrafficProfile {
            target_url: self.target_url,
            duration: std::time::Duration::from_secs(self.duration_seconds),
            requests_per_second: self.requests_per_second,
            concurrency: self.concurrency,
            key_header: self.key_header,
        };

        let key_mode = match self.key_mode.mode {
            KeyModeKind::Keyless => KeyMode::Keyless,
            KeyModeKind::SingleKey => KeyMode::SingleKey(self.key_mode.key.ok_or(
                ConfigError::InvalidValue("`key_mode.key` is required when mode=single_key"),
            )?),
            KeyModeKind::RoundRobin => KeyMode::RoundRobin(self.key_mode.keys),
        };

        Ok((profile, key_mode))
    }

    fn validate(&self) -> Result<(), ConfigError> {
        if self.target_url.trim().is_empty() {
            return Err(ConfigError::InvalidValue("`target_url` cannot be empty"));
        }
        if self.duration_seconds == 0 {
            return Err(ConfigError::InvalidValue("`duration_seconds` must be > 0"));
        }
        if self.concurrency == 0 {
            return Err(ConfigError::InvalidValue("`concurrency` must be > 0"));
        }
        if self.key_header.trim().is_empty() {
            return Err(ConfigError::InvalidValue("`key_header` cannot be empty"));
        }

        match self.key_mode.mode {
            KeyModeKind::Keyless => Ok(()),
            KeyModeKind::SingleKey => {
                let Some(key) = &self.key_mode.key else {
                    return Err(ConfigError::InvalidValue(
                        "`key_mode.key` is required when mode=single_key",
                    ));
                };

                if key.trim().is_empty() {
                    return Err(ConfigError::InvalidValue(
                        "`key_mode.key` cannot be empty when mode=single_key",
                    ));
                }
                Ok(())
            }
            KeyModeKind::RoundRobin => {
                if self.key_mode.keys.is_empty() {
                    return Err(ConfigError::InvalidValue(
                        "`key_mode.keys` must include at least one key when mode=round_robin",
                    ));
                }
                if self.key_mode.keys.iter().any(|k| k.trim().is_empty()) {
                    return Err(ConfigError::InvalidValue(
                        "`key_mode.keys` cannot include empty values",
                    ));
                }
                Ok(())
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{ConfigError, TrafficRunConfig};

    #[test]
    fn default_config_is_valid() {
        let cfg = TrafficRunConfig::default();
        assert!(cfg.validate().is_ok());
    }

    #[test]
    fn round_robin_requires_keys() {
        let text = r#"
target_url = "http://127.0.0.1:3000/"
duration_seconds = 5
requests_per_second = 1000
concurrency = 2
key_header = "x-api-key"

[key_mode]
mode = "round_robin"
"#;

        let parsed = toml::from_str::<TrafficRunConfig>(text).expect("parse");
        let err = parsed.validate().expect_err("validation should fail");
        assert!(matches!(err, ConfigError::InvalidValue(_)));
    }

    #[test]
    fn single_key_requires_key_value() {
        let text = r#"
target_url = "http://127.0.0.1:3000/"
duration_seconds = 5
requests_per_second = 1000
concurrency = 2
key_header = "x-api-key"

[key_mode]
mode = "single_key"
"#;

        let parsed = toml::from_str::<TrafficRunConfig>(text).expect("parse");
        let err = parsed.validate().expect_err("validation should fail");
        assert!(matches!(err, ConfigError::InvalidValue(_)));
    }
}
