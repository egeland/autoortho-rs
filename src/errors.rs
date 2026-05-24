// SPDX-License-Identifier: Apache-2.0 OR GPL-3.0
// Copyright (c) 2026 the AutoOrtho contributors

//! Config validation errors and helpers.
//!
//! `ConfigError` is the canonical error type for configuration validation.
//! `RateLimitConfig` lives here alongside its validator.

use serde::{Deserialize, Serialize};

/// Configuration validation error
#[derive(Debug, Clone, PartialEq)]
pub enum ConfigError {
    FieldInvalid {
        field: String,
        message: String,
    },
    FieldOutOfRange {
        field: String,
        min: u64,
        max: u64,
        value: u64,
    },
}

impl std::fmt::Display for ConfigError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::FieldInvalid { field, message } => write!(f, "Invalid {}: {}", field, message),
            Self::FieldOutOfRange {
                field,
                min,
                max,
                value,
            } => {
                write!(f, "{} out of range ({}-{}), got {}", field, min, max, value)
            }
        }
    }
}

impl std::error::Error for ConfigError {}

/// Validate a u64 field is within range.
pub fn validate_range(value: u64, min: u64, max: u64, field: &str) -> Result<(), ConfigError> {
    if value < min || value > max {
        return Err(ConfigError::FieldOutOfRange {
            field: field.to_string(),
            min,
            max,
            value,
        });
    }
    Ok(())
}

/// Validate a f32 field is within range.
pub fn validate_f32_range(value: f32, min: f32, max: f32, field: &str) -> Result<(), ConfigError> {
    if value < min || value > max {
        return Err(ConfigError::FieldInvalid {
            field: field.to_string(),
            message: format!("out of range ({}-{}), got {}", min, max, value),
        });
    }
    Ok(())
}

/// Validate log_rotation is a valid value.
pub fn validate_log_rotation(rotation: &str) -> Result<(), ConfigError> {
    match rotation {
        "daily" | "hourly" | "never" => Ok(()),
        _ => Err(ConfigError::FieldInvalid {
            field: "log_rotation".to_string(),
            message: "must be 'daily', 'hourly', or 'never'".to_string(),
        }),
    }
}

/// Rate limiting configuration for tile requests
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RateLimitConfig {
    pub requests_per_second: f64,
}

impl Default for RateLimitConfig {
    fn default() -> Self {
        Self {
            requests_per_second: 5.0,
        }
    }
}

impl RateLimitConfig {
    pub fn validate(&self) -> Result<(), ConfigError> {
        if self.requests_per_second < 1.0 || self.requests_per_second > 20.0 {
            return Err(ConfigError::FieldInvalid {
                field: "rate_limit.requests_per_second".to_string(),
                message: format!("out of range (1.0-20.0), got {}", self.requests_per_second),
            });
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validate_range_valid() {
        assert!(validate_range(5, 0, 10, "test").is_ok());
        assert!(validate_range(0, 0, 10, "test").is_ok());
        assert!(validate_range(10, 0, 10, "test").is_ok());
    }

    #[test]
    fn test_validate_range_invalid() {
        let r = validate_range(15, 0, 10, "test_field");
        assert!(r.is_err());
        let err = r.unwrap_err();
        assert!(matches!(err, ConfigError::FieldOutOfRange { field, .. } if field == "test_field"));
    }

    #[test]
    fn test_validate_f32_range_valid() {
        assert!(validate_f32_range(0.5, 0.0, 1.0, "test").is_ok());
        assert!(validate_f32_range(1.0, 0.0, 1.0, "test").is_ok());
        assert!(validate_f32_range(0.0, 0.0, 1.0, "test").is_ok());
    }

    #[test]
    fn test_validate_f32_range_invalid() {
        let r = validate_f32_range(2.0, 0.0, 1.0, "test_f32");
        assert!(r.is_err());
    }

    #[test]
    fn test_validate_log_rotation_valid() {
        assert!(validate_log_rotation("daily").is_ok());
        assert!(validate_log_rotation("hourly").is_ok());
        assert!(validate_log_rotation("never").is_ok());
    }

    #[test]
    fn test_validate_log_rotation_invalid() {
        assert!(validate_log_rotation("weekly").is_err());
        assert!(validate_log_rotation("").is_err());
    }

    #[test]
    fn test_rate_limit_config_validate_valid() {
        let rl = RateLimitConfig::default();
        assert!(rl.validate().is_ok());

        let mut rl = RateLimitConfig::default();
        rl.requests_per_second = 10.0;
        assert!(rl.validate().is_ok());
    }

    #[test]
    fn test_rate_limit_config_validate_invalid() {
        let mut rl = RateLimitConfig::default();
        rl.requests_per_second = 0.5;
        assert!(rl.validate().is_err());

        let mut rl = RateLimitConfig::default();
        rl.requests_per_second = 25.0;
        assert!(rl.validate().is_err());
    }

    #[test]
    fn test_config_error_display() {
        let err = ConfigError::FieldInvalid {
            field: "foo".to_string(),
            message: "bad value".to_string(),
        };
        assert_eq!(format!("{}", err), "Invalid foo: bad value");

        let err = ConfigError::FieldOutOfRange {
            field: "bar".to_string(),
            min: 1,
            max: 10,
            value: 15,
        };
        assert_eq!(format!("{}", err), "bar out of range (1-10), got 15");
    }
}
