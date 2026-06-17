// SPDX-License-Identifier: Apache-2.0 OR GPL-3.0
// Copyright (c) 2026 the AutoOrtho contributors

use serde::{Deserialize, Serialize};

use crate::errors::ConfigError;

/// UI configuration — display, window, and debug settings.
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct UiConfig {
    #[serde(default = "default_ui_scale")]
    pub ui_scale: f64,
    #[serde(default)]
    pub window_x: Option<f32>,
    #[serde(default)]
    pub window_y: Option<f32>,
    #[serde(default)]
    pub window_width: Option<f32>,
    #[serde(default)]
    pub window_height: Option<f32>,
    #[serde(default)]
    pub debug_mode: bool,
    #[serde(default = "default_log_rotation")]
    pub log_rotation: String,
}

impl Default for UiConfig {
    fn default() -> Self {
        Self {
            ui_scale: default_ui_scale(),
            window_x: None,
            window_y: None,
            window_width: None,
            window_height: None,
            debug_mode: false,
            log_rotation: default_log_rotation(),
        }
    }
}

impl UiConfig {
    pub fn validate(&self) -> Result<(), ConfigError> {
        if self.ui_scale < 0.5 || self.ui_scale > 1.5 {
            return Err(ConfigError::FieldOutOfRange {
                field: "ui_scale".to_string(),
                min: 50,
                max: 150,
                value: (self.ui_scale * 100.0) as u64,
            });
        }
        validate_log_rotation(&self.log_rotation)?;
        Ok(())
    }

    /// Clear saved window position/size.
    pub fn reset_window_position(&mut self) {
        self.window_x = None;
        self.window_y = None;
        self.window_width = None;
        self.window_height = None;
    }
}

fn default_ui_scale() -> f64 {
    1.0
}

fn default_log_rotation() -> String {
    "daily".to_string()
}

fn validate_log_rotation(value: &str) -> Result<(), ConfigError> {
    match value {
        "daily" | "hourly" | "never" => Ok(()),
        _ => Err(ConfigError::FieldInvalid {
            field: "log_rotation".to_string(),
            message: format!(
                "Invalid log_rotation '{}'. Must be 'daily', 'hourly', or 'never'",
                value
            ),
        }),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ui_config_defaults() {
        let config = UiConfig::default();
        assert_eq!(config.ui_scale, 1.0);
        assert!(config.window_x.is_none());
        assert!(!config.debug_mode);
        assert_eq!(config.log_rotation, "daily");
    }

    #[test]
    fn test_ui_config_validate_valid() {
        let config = UiConfig::default();
        assert!(config.validate().is_ok());
    }

    #[test]
    fn test_ui_config_validate_invalid_scale() {
        let mut config = UiConfig::default();
        config.ui_scale = 0.3;
        assert!(config.validate().is_err());
    }

    #[test]
    fn test_ui_config_validate_invalid_log_rotation() {
        let mut config = UiConfig::default();
        config.log_rotation = "weekly".to_string();
        assert!(config.validate().is_err());
    }

    #[test]
    fn test_ui_config_reset_window_position() {
        let mut config = UiConfig::default();
        config.window_x = Some(100.0);
        config.window_y = Some(200.0);
        config.window_width = Some(800.0);
        config.window_height = Some(600.0);
        config.reset_window_position();
        assert!(config.window_x.is_none());
        assert!(config.window_y.is_none());
        assert!(config.window_width.is_none());
        assert!(config.window_height.is_none());
    }

    #[test]
    fn test_ui_config_serde_roundtrip() {
        let config = UiConfig::default();
        let toml = toml::to_string(&config).unwrap();
        let loaded: UiConfig = toml::from_str(&toml).unwrap();
        assert_eq!(config.ui_scale, loaded.ui_scale);
    }
}
