//! Configuration module stub for Sentinel proxy
//!
//! This is a temporary stub for Phase 0. The full configuration
//! system is implemented in the sentinel-config crate.

use serde::{Deserialize, Serialize};
use std::time::Duration;

/// Basic configuration for Phase 0 testing
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    /// Server configuration
    pub server: ServerConfig,
    /// Default upstream for testing
    pub default_upstream: UpstreamConfig,
    /// Limits configuration
    pub limits: LimitsConfig,
}

/// Server configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServerConfig {
    /// Listen address
    pub listen_address: String,
    /// Number of worker threads
    pub worker_threads: usize,
    /// Graceful shutdown timeout
    pub shutdown_timeout: Duration,
}

/// Upstream configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpstreamConfig {
    /// Upstream address
    pub address: String,
    /// Use TLS
    pub tls: bool,
    /// SNI hostname
    pub sni: Option<String>,
    /// Connect timeout
    pub connect_timeout: Duration,
    /// Read timeout
    pub read_timeout: Duration,
    /// Write timeout
    pub write_timeout: Duration,
}

/// Basic limits configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LimitsConfig {
    /// Maximum header count
    pub max_header_count: usize,
    /// Maximum header size
    pub max_header_size: usize,
    /// Maximum body size
    pub max_body_size: usize,
    /// Maximum concurrent connections
    pub max_connections: usize,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            server: ServerConfig {
                listen_address: "0.0.0.0:8080".to_string(),
                worker_threads: 4,
                shutdown_timeout: Duration::from_secs(30),
            },
            default_upstream: UpstreamConfig {
                address: "127.0.0.1:8081".to_string(),
                tls: false,
                sni: None,
                connect_timeout: Duration::from_secs(10),
                read_timeout: Duration::from_secs(30),
                write_timeout: Duration::from_secs(30),
            },
            limits: LimitsConfig {
                max_header_count: 100,
                max_header_size: 8192,
                max_body_size: 10 * 1024 * 1024, // 10MB
                max_connections: 10000,
            },
        }
    }
}

impl Config {
    /// Load configuration from environment variables
    pub fn from_env() -> Self {
        let mut config = Self::default();

        if let Ok(addr) = std::env::var("SENTINEL_LISTEN") {
            config.server.listen_address = addr;
        }

        if let Ok(upstream) = std::env::var("SENTINEL_UPSTREAM") {
            config.default_upstream.address = upstream;
        }

        if let Ok(threads) = std::env::var("SENTINEL_WORKERS") {
            if let Ok(n) = threads.parse() {
                config.server.worker_threads = n;
            }
        }

        config
    }

    /// Validate the configuration
    pub fn validate(&self) -> Result<(), String> {
        if self.server.worker_threads == 0 {
            return Err("Worker threads must be > 0".to_string());
        }

        if self.limits.max_header_count == 0 {
            return Err("Max header count must be > 0".to_string());
        }

        if self.limits.max_header_size == 0 {
            return Err("Max header size must be > 0".to_string());
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let config = Config::default();
        assert_eq!(config.server.listen_address, "0.0.0.0:8080");
        assert_eq!(config.default_upstream.address, "127.0.0.1:8081");
        assert_eq!(config.limits.max_header_count, 100);
    }

    #[test]
    fn test_config_validation() {
        let mut config = Config::default();
        assert!(config.validate().is_ok());

        config.server.worker_threads = 0;
        assert!(config.validate().is_err());

        config.server.worker_threads = 4;
        config.limits.max_header_count = 0;
        assert!(config.validate().is_err());
    }
}
