// SPDX-License-Identifier: Apache-2.0 OR GPL-3.0
// Copyright (c) 2026 the AutoOrtho contributors

use serde::{Deserialize, Serialize};

/// Seasonal configuration — current season and per-season saturation.
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct SeasonConfig {
    #[serde(default)]
    pub season: crate::seasons::Season,
    #[serde(default = "default_spring_saturation")]
    pub spring_saturation: f32,
    #[serde(default = "default_summer_saturation")]
    pub summer_saturation: f32,
    #[serde(default = "default_autumn_saturation")]
    pub autumn_saturation: f32,
    #[serde(default = "default_winter_saturation")]
    pub winter_saturation: f32,
}

impl Default for SeasonConfig {
    fn default() -> Self {
        Self {
            season: Default::default(),
            spring_saturation: default_spring_saturation(),
            summer_saturation: default_summer_saturation(),
            autumn_saturation: default_autumn_saturation(),
            winter_saturation: default_winter_saturation(),
        }
    }
}

fn default_spring_saturation() -> f32 {
    0.70
}

fn default_summer_saturation() -> f32 {
    1.0
}

fn default_autumn_saturation() -> f32 {
    0.80
}

fn default_winter_saturation() -> f32 {
    0.55
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_season_config_defaults() {
        let config = SeasonConfig::default();
        assert_eq!(config.spring_saturation, 0.70);
        assert_eq!(config.summer_saturation, 1.0);
        assert_eq!(config.autumn_saturation, 0.80);
        assert_eq!(config.winter_saturation, 0.55);
    }

    #[test]
    fn test_season_config_serde_roundtrip() {
        let config = SeasonConfig::default();
        let toml = toml::to_string(&config).unwrap();
        let loaded: SeasonConfig = toml::from_str(&toml).unwrap();
        assert_eq!(config.spring_saturation, loaded.spring_saturation);
    }
}
