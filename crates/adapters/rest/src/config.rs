use std::{
    fmt, fs,
    net::SocketAddr,
    path::{Path, PathBuf},
    time::Duration,
};

use clap::Parser;
use serde::Deserialize;

#[derive(Debug, Parser)]
#[command(
    name = "rest_server",
    version,
    about = "REST server for rate limiting experiments."
)]
struct Args {
    /// Optional path to a TOML config file.
    #[arg(long)]
    config: Option<PathBuf>,
}

fn read_to_string(path: &Path) -> std::io::Result<String> {
    fs::read_to_string(path)
}

/// REST server configuration.
///
/// Loaded from a TOML file via `--config <path>`.
/// If no config is provided, defaults are used.
#[derive(Debug, Clone, Deserialize)]
pub struct RestServerConfig {
    /// Address to bind the server to, e.g. "127.0.0.1:3000".
    pub bind_address: String,

    /// Key extraction policy.
    pub key_mode: KeyMode,

    /// Header used for key extraction, e.g. "x-api-key".
    pub key_header: String,

    /// Fallback key used when `key_mode = HeaderOrAnonymous` and header is missing.
    pub anonymous_key: String,

    /// Instrumentation level, mapped to `rate_limit::models::InstrumantationLevel`.
    pub instrumentation: InstrumentationMode,

    /// Limits used by the in-memory limiter variant.
    #[serde(default)]
    pub limits: Option<LimitsConfig>,

    /// Limits used by the distributed limiter variant.
    #[serde(default)]
    pub distributed: Option<DistributedConfig>,

    /// Hybrid limiter options.
    #[serde(default)]
    pub hybrid: Option<HybridConfig>,

    /// Optional observability options.
    #[serde(default)]
    pub observability: Option<ObservabilityConfig>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct LimitsConfig {
    pub global_per_second: u32,
    pub per_key_per_second: u32,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum KeyMode {
    /// Prefer `key_header`, fall back to `anonymous_key`.
    HeaderOrAnonymous,

    /// Require `key_header`; if missing, treat as anonymous_key (still produces a key).
    HeaderOnly,

    /// Ignore header and always use `anonymous_key`.
    AnonymousOnly,
}

#[derive(Debug, Clone, Copy, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum InstrumentationMode {
    Off,
    Basic,
    Full,
}

#[derive(Debug, Clone, Deserialize)]
pub struct DistributedConfig {
    pub namespace: String,
    pub window_ms: u64,
    pub max: u32,
}

#[derive(Debug, Clone, Deserialize)]
pub struct HybridConfig {
    #[serde(default)]
    pub failure_policy: Option<HybridFailurePolicyMode>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct ObservabilityConfig {
    #[serde(default)]
    pub enabled: bool,
}

#[derive(Debug, Clone, Copy, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum HybridFailurePolicyMode {
    FailOpen,
    FailClosed,
}

#[derive(Debug)]
pub enum ConfigError {
    MissingInMemoryLimits,
    MissingDistributedConfig,
    InvalidLimitValue { name: &'static str, value: u32 },
}

impl fmt::Display for ConfigError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ConfigError::MissingInMemoryLimits => {
                write!(f, "missing [limits] configuration for in_memory_limiter")
            }
            ConfigError::MissingDistributedConfig => {
                write!(f, "missing [distributed] configuration for this limiter")
            }
            ConfigError::InvalidLimitValue { name, value } => {
                write!(f, "limit `{}` must be non-zero (found {})", name, value)
            }
        }
    }
}

impl std::error::Error for ConfigError {}

impl Default for RestServerConfig {
    fn default() -> Self {
        Self {
            bind_address: "127.0.0.1:3000".to_string(),
            key_mode: KeyMode::HeaderOrAnonymous,
            key_header: "x-api-key".to_string(),
            anonymous_key: "anonymous".to_string(),
            instrumentation: InstrumentationMode::Off,
            limits: Some(LimitsConfig {
                global_per_second: 1_000,
                per_key_per_second: 1_000,
            }),
            distributed: None,
            hybrid: None,
            observability: None,
        }
    }
}

impl RestServerConfig {
    /// Parse config from CLI/file with defaults.
    ///
    /// - If `--config` is provided, TOML is loaded and parsed.
    /// - If not, defaults are used.
    ///
    /// Panics with a clear message on malformed config.
    pub fn load() -> Self {
        let args = Args::parse();
        match args.config {
            Some(path) => {
                log::debug!("Loading REST server config from: {:?}", path);
                let text = read_to_string(&path)
                    .unwrap_or_else(|err| panic!("failed to read config {path:?}: {err}"));
                toml::from_str::<RestServerConfig>(&text)
                    .unwrap_or_else(|err| panic!("failed to parse TOML config {path:?}: {err}"))
            }
            None => {
                log::debug!("No config file provided, using defaults.");
                RestServerConfig::default()
            }
        }
    }

    /// Convenience: parse bind_address into SocketAddress.
    pub fn bind_socket_addr(&self) -> SocketAddr {
        self.bind_address
            .parse()
            .unwrap_or_else(|err| panic!("invalid bind_address {:?}: {err}", self.bind_address))
    }

    /// Convenience: map to `rate_limit::models::InstrumentationLevel`.
    pub fn instrumentation_level(&self) -> rate_limit::models::InstrumentationLevel {
        match self.instrumentation {
            InstrumentationMode::Off => rate_limit::models::InstrumentationLevel::Off,
            InstrumentationMode::Basic => rate_limit::models::InstrumentationLevel::Basic,
            InstrumentationMode::Full => rate_limit::models::InstrumentationLevel::Full,
        }
    }

    /// Convenience: distributed window duration.
    pub fn distributed_window(&self) -> Option<Duration> {
        let Some(d) = &self.distributed else {
            return None;
        };
        Some(Duration::from_millis(d.window_ms))
    }

    pub fn require_limits(&self) -> Result<&LimitsConfig, ConfigError> {
        self.limits
            .as_ref()
            .ok_or(ConfigError::MissingInMemoryLimits)
    }

    pub fn require_distributed(&self) -> Result<&DistributedConfig, ConfigError> {
        self.distributed
            .as_ref()
            .ok_or(ConfigError::MissingDistributedConfig)
    }

    pub fn validate_for_enabled_feature(&self) -> Result<(), ConfigError> {
        #[cfg(feature = "in_memory_limiter")]
        {
            self.validate_inmemory_limits()?;
        }
        #[cfg(feature = "distributed_limiter")]
        {
            self.validate_distributed_limits()?;
        }
        #[cfg(feature = "hybrid_limiter")]
        {
            self.validate_inmemory_limits()?;
            self.validate_distributed_limits()?;
        }
        Ok(())
    }

    pub fn observability_enabled(&self) -> bool {
        self.observability
            .as_ref()
            .map(|o| o.enabled)
            .unwrap_or(false)
    }

    #[cfg(any(feature = "in_memory_limiter", feature = "hybrid_limiter"))]
    fn validate_inmemory_limits(&self) -> Result<(), ConfigError> {
        let limits = self.require_limits()?;
        if limits.global_per_second == 0 {
            return Err(ConfigError::InvalidLimitValue {
                name: "global_per_second",
                value: limits.global_per_second,
            });
        }
        if limits.per_key_per_second == 0 {
            return Err(ConfigError::InvalidLimitValue {
                name: "per_key_per_second",
                value: limits.per_key_per_second,
            });
        }
        Ok(())
    }

    #[cfg(any(feature = "distributed_limiter", feature = "hybrid_limiter"))]
    fn validate_distributed_limits(&self) -> Result<(), ConfigError> {
        let distributed = self.require_distributed()?;
        if distributed.window_ms == 0 {
            return Err(ConfigError::InvalidLimitValue {
                name: "distributed.window_ms",
                value: 0,
            });
        }
        if distributed.max == 0 {
            return Err(ConfigError::InvalidLimitValue {
                name: "distributed.max",
                value: distributed.max,
            });
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::RestServerConfig;

    #[test]
    fn observability_defaults_to_disabled() {
        let cfg = RestServerConfig::default();
        assert!(!cfg.observability_enabled());
    }

    #[test]
    fn observability_can_be_enabled_from_toml() {
        let toml = r#"
bind_address = "127.0.0.1:3000"
key_mode = "header_or_anonymous"
key_header = "x-api-key"
anonymous_key = "anonymous"
instrumentation = "off"

[limits]
global_per_second = 1000
per_key_per_second = 1000

[observability]
enabled = true
"#;

        let cfg: RestServerConfig = toml::from_str(toml).expect("parse config");
        assert!(cfg.observability_enabled());
    }
}
