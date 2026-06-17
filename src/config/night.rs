// SPDX-License-Identifier: Apache-2.0 OR GPL-3.0
// Copyright (c) 2026 the AutoOrtho contributors

use serde::{Deserialize, Serialize};

use crate::errors::{ConfigError, validate_f32_range};

/// Night exclusion configuration — sun pitch thresholds.
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct NightConfig {
    #[serde(default = "default_enable_night_exclusion")]
    pub enable_night_exclusion: bool,
    #[serde(default = "default_night_threshold")]
    pub night_threshold: f32,
    #[serde(default = "default_day_threshold")]
    pub day_threshold: f32,
}

impl Default for NightConfig {
    fn default() -> Self {
        Self {
            enable_night_exclusion: default_enable_night_exclusion(),
            night_threshold: default_night_threshold(),
            day_threshold: default_day_threshold(),
        }
    }
}

impl NightConfig {
    pub fn validate(&self) -> Result<(), ConfigError> {
        validate_f32_range(self.night_threshold, -90.0, 0.0, "night_threshold")?;
        validate_f32_range(self.day_threshold, -90.0, 90.0, "day_threshold")?;
        if self.night_threshold > self.day_threshold {
            return Err(ConfigError::FieldInvalid {
                field: "night_threshold".to_string(),
                message: "night_threshold must be <= day_threshold".to_string(),
            });
        }
        Ok(())
    }
}

fn default_enable_night_exclusion() -> bool {
    true
}

fn default_night_threshold() -> f32 {
    -12.0
}

fn default_day_threshold() -> f32 {
    -10.0
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_night_config_defaults() {
        let config = NightConfig::default();
        assert!(config.enable_night_exclusion);
        assert_eq!(config.night_threshold, -12.0);
        assert_eq!(config.day_threshold, -10.0);
    }

    #[test]
    fn test_night_config_validate_valid() {
        let config = NightConfig::default();
        assert!(config.validate().is_ok());
    }

    #[test]
    fn test_night_config_validate_invalid_order() {
        let mut config = NightConfig::default();
        config.night_threshold = 20.0;
        config.day_threshold = 10.0;
        assert!(config.validate().is_err());
    }

    #[test]
    fn test_night_config_validate_threshold_out_of_range() {
        let mut config = NightConfig::default();
        config.night_threshold = -100.0;
        assert!(config.validate().is_err());
    }

    #[test]
    fn test_night_config_serde_roundtrip() {
        let config = NightConfig::default();
        let toml = toml::to_string(&config).unwrap();
        let loaded: NightConfig = toml::from_str(&toml).unwrap();
        assert_eq!(config.night_threshold, loaded.night_threshold);
    }
}
