// SPDX-License-Identifier: Apache-2.0 OR GPL-3.0
// Copyright (c) 2026 the AutoOrtho contributors

use serde::{Deserialize, Serialize};

use crate::errors::{ConfigError, validate_range};

/// Network configuration — X-Plane connection and rate limiting.
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct NetworkConfig {
    #[serde(default = "default_xplane_host")]
    pub xplane_host: String,
    #[serde(default = "default_xplane_port")]
    pub xplane_port: u16,
    #[serde(default)]
    pub rate_limit: crate::errors::RateLimitConfig,
}

impl Default for NetworkConfig {
    fn default() -> Self {
        Self {
            xplane_host: default_xplane_host(),
            xplane_port: default_xplane_port(),
            rate_limit: Default::default(),
        }
    }
}

impl NetworkConfig {
    pub fn validate(&self) -> Result<(), ConfigError> {
        validate_range(self.xplane_port as u64, 1, 65535, "xplane_port")?;
        Ok(())
    }
}

fn default_xplane_host() -> String {
    "127.0.0.1".to_string()
}

fn default_xplane_port() -> u16 {
    49000
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_network_config_defaults() {
        let config = NetworkConfig::default();
        assert_eq!(config.xplane_host, "127.0.0.1");
        assert_eq!(config.xplane_port, 49000);
    }

    #[test]
    fn test_network_config_validate_valid() {
        let config = NetworkConfig::default();
        assert!(config.validate().is_ok());
    }

    #[test]
    fn test_network_config_validate_invalid_port() {
        let mut config = NetworkConfig::default();
        config.xplane_port = 0;
        assert!(config.validate().is_err());
    }

    #[test]
    fn test_network_config_validate_high_port() {
        let mut config = NetworkConfig::default();
        config.xplane_port = 65535;
        assert!(config.validate().is_ok());
    }

    #[test]
    fn test_network_config_serde_roundtrip() {
        let config = NetworkConfig::default();
        let toml = toml::to_string(&config).unwrap();
        let loaded: NetworkConfig = toml::from_str(&toml).unwrap();
        assert_eq!(config.xplane_host, loaded.xplane_host);
        assert_eq!(config.xplane_port, loaded.xplane_port);
    }
}
