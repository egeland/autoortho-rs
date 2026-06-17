// SPDX-License-Identifier: Apache-2.0 OR GPL-3.0
// Copyright (c) 2026 the AutoOrtho contributors

use serde::{Deserialize, Serialize};

/// Flight configuration — SimBrief, prefetch, and route parameters.
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct FlightConfig {
    #[serde(default)]
    pub simbrief_user_id: String,
    #[serde(default = "default_route_consideration_radius_nm")]
    pub route_consideration_radius_nm: u32,
    #[serde(default = "default_route_deviation_threshold_nm")]
    pub route_deviation_threshold_nm: u32,
    #[serde(default = "default_route_prefetch_radius_nm")]
    pub route_prefetch_radius_nm: u32,
    #[serde(default = "default_prefetch_route_percent")]
    pub prefetch_route_percent: u32,
    #[serde(default = "default_prefetch_airports")]
    pub prefetch_airports: bool,
    #[serde(default = "default_airport_radius_nm")]
    pub airport_radius_nm: u32,
    #[serde(default = "default_near_airport_zoom")]
    pub near_airport_zoom: u32,
    #[serde(default = "default_use_simbrief_altitude")]
    pub use_simbrief_altitude: bool,
}

impl Default for FlightConfig {
    fn default() -> Self {
        Self {
            simbrief_user_id: Default::default(),
            route_consideration_radius_nm: default_route_consideration_radius_nm(),
            route_deviation_threshold_nm: default_route_deviation_threshold_nm(),
            route_prefetch_radius_nm: default_route_prefetch_radius_nm(),
            prefetch_route_percent: default_prefetch_route_percent(),
            prefetch_airports: default_prefetch_airports(),
            airport_radius_nm: default_airport_radius_nm(),
            near_airport_zoom: default_near_airport_zoom(),
            use_simbrief_altitude: default_use_simbrief_altitude(),
        }
    }
}

fn default_route_consideration_radius_nm() -> u32 {
    50
}

fn default_route_deviation_threshold_nm() -> u32 {
    40
}

fn default_route_prefetch_radius_nm() -> u32 {
    40
}

fn default_prefetch_route_percent() -> u32 {
    20
}

fn default_prefetch_airports() -> bool {
    true
}

fn default_airport_radius_nm() -> u32 {
    60
}

fn default_near_airport_zoom() -> u32 {
    19
}

fn default_use_simbrief_altitude() -> bool {
    true
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_flight_config_defaults() {
        let config = FlightConfig::default();
        assert!(config.simbrief_user_id.is_empty());
        assert_eq!(config.route_consideration_radius_nm, 50);
        assert_eq!(config.route_deviation_threshold_nm, 40);
        assert_eq!(config.route_prefetch_radius_nm, 40);
        assert_eq!(config.prefetch_route_percent, 20);
        assert!(config.prefetch_airports);
        assert_eq!(config.airport_radius_nm, 60);
        assert_eq!(config.near_airport_zoom, 19);
        assert!(config.use_simbrief_altitude);
    }

    #[test]
    fn test_flight_config_serde_roundtrip() {
        let config = FlightConfig::default();
        let toml = toml::to_string(&config).unwrap();
        let loaded: FlightConfig = toml::from_str(&toml).unwrap();
        assert_eq!(
            config.route_consideration_radius_nm,
            loaded.route_consideration_radius_nm
        );
    }
}
