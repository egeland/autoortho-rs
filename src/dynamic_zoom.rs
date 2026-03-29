// SPDX-License-Identifier: Apache-2.0 OR GPL-3.0
// Copyright (c) 2024 the AutoOrtho contributors

use crate::config::ZoomRule;
use crate::tiles::provider::PROVIDER_INFO;
use crate::xplane::simbrief::FlightPlan;
use crate::xplane::simbrief::haversine_nm;

/// Altitude-based dynamic zoom level selection using rules.
pub struct DynamicZoom {
    zoom_rules: Vec<ZoomRule>,
    provider_max_zoom: u32,
}

impl DynamicZoom {
    pub fn new(zoom_rules: Vec<ZoomRule>, provider_id: &str) -> Self {
        let provider_max_zoom = PROVIDER_INFO
            .iter()
            .find(|p| p.id == provider_id)
            .map(|p| p.max_zoom)
            .unwrap_or(19);

        let mut rules = zoom_rules;
        rules.sort_by(|a, b| a.min_altitude_ft.partial_cmp(&b.min_altitude_ft).unwrap());

        Self {
            zoom_rules: rules,
            provider_max_zoom,
        }
    }

    pub fn zoom_for_altitude_agl(&self, altitude_agl_ft: f32) -> u32 {
        let mut result = 16u32;
        for rule in &self.zoom_rules {
            if altitude_agl_ft >= rule.min_altitude_ft {
                result = rule.zoom_level;
            }
        }
        result.min(self.provider_max_zoom)
    }

    pub fn zoom_for_position_with_simbrief(
        &self,
        lat: f64,
        lon: f64,
        dataref_altitude_agl_ft: f32,
        flight_plan: Option<&FlightPlan>,
        consideration_radius_nm: f64,
    ) -> u32 {
        if let Some(plan) = flight_plan {
            let simbrief_altitude =
                self.lowest_agl_altitude_nearby(lat, lon, plan, consideration_radius_nm);
            if let Some(agl_ft) = simbrief_altitude {
                return self.zoom_for_altitude_agl(agl_ft);
            }
        }
        self.zoom_for_altitude_agl(dataref_altitude_agl_ft)
    }

    fn lowest_agl_altitude_nearby(
        &self,
        lat: f64,
        lon: f64,
        flight_plan: &FlightPlan,
        radius_nm: f64,
    ) -> Option<f32> {
        let mut lowest_agl: Option<f32> = None;

        for fix in &flight_plan.fixes {
            let dist = haversine_nm(lat, lon, fix.lat, fix.lon);
            if dist <= radius_nm {
                let agl = fix.altitude_agl_ft();
                match lowest_agl {
                    None => lowest_agl = Some(agl),
                    Some(current) if agl < current => lowest_agl = Some(agl),
                    Some(_) => {}
                }
            }
        }

        lowest_agl
    }

    pub fn max_zoom(&self) -> u32 {
        self.provider_max_zoom
    }

    pub fn zoom_rules(&self) -> &[ZoomRule] {
        &self.zoom_rules
    }

    pub fn provider_max_zoom(&self) -> u32 {
        self.provider_max_zoom
    }

    pub fn validate_rules(&self) -> Vec<String> {
        let mut errors = Vec::new();

        for rule in &self.zoom_rules {
            if rule.zoom_level > self.provider_max_zoom {
                errors.push(format!(
                    "Zoom level {} exceeds provider max {}",
                    rule.zoom_level, self.provider_max_zoom
                ));
            }
        }

        errors
    }
}

impl Default for DynamicZoom {
    fn default() -> Self {
        Self::new(
            vec![
                ZoomRule {
                    min_altitude_ft: 0.0,
                    zoom_level: 19,
                },
                ZoomRule {
                    min_altitude_ft: 10000.0,
                    zoom_level: 16,
                },
            ],
            "ARC",
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_dynamic_zoom_default() {
        let dz = DynamicZoom::default();
        assert_eq!(dz.provider_max_zoom, 19);
        assert_eq!(dz.zoom_rules.len(), 2);
    }

    #[test]
    fn test_zoom_for_altitude_agl_low() {
        let dz = DynamicZoom::default();
        assert_eq!(dz.zoom_for_altitude_agl(0.0), 19);
        assert_eq!(dz.zoom_for_altitude_agl(5000.0), 19);
    }

    #[test]
    fn test_zoom_for_altitude_agl_high() {
        let dz = DynamicZoom::default();
        assert_eq!(dz.zoom_for_altitude_agl(15000.0), 16);
    }

    #[test]
    fn test_zoom_for_altitude_agl_boundary() {
        let dz = DynamicZoom::default();
        assert_eq!(dz.zoom_for_altitude_agl(9999.0), 19);
        assert_eq!(dz.zoom_for_altitude_agl(10000.0), 16);
    }

    #[test]
    fn test_rules_are_sorted() {
        let rules = vec![
            ZoomRule {
                min_altitude_ft: 15000.0,
                zoom_level: 14,
            },
            ZoomRule {
                min_altitude_ft: 0.0,
                zoom_level: 19,
            },
            ZoomRule {
                min_altitude_ft: 5000.0,
                zoom_level: 17,
            },
        ];
        let dz = DynamicZoom::new(rules, "ARC");
        let rules = dz.zoom_rules();
        assert_eq!(rules[0].min_altitude_ft, 0.0);
        assert_eq!(rules[1].min_altitude_ft, 5000.0);
        assert_eq!(rules[2].min_altitude_ft, 15000.0);
    }

    #[test]
    fn test_provider_max_zoom() {
        let dz = DynamicZoom::new(vec![], "BI");
        assert_eq!(dz.provider_max_zoom(), 19);

        let dz = DynamicZoom::new(vec![], "GO2");
        assert_eq!(dz.provider_max_zoom(), 21);
    }

    #[test]
    fn test_zoom_for_position_with_simbrief_uses_simbrief() {
        use crate::xplane::simbrief::{FlightFix, FlightPlan};

        let dz = DynamicZoom::default();
        let plan = FlightPlan {
            origin: "KLAX".to_string(),
            destination: "KSFO".to_string(),
            origin_elevation_ft: 126.0,
            destination_elevation_ft: 8.0,
            cruise_altitude_ft: 35000.0,
            fixes: vec![FlightFix {
                ident: "PMD".to_string(),
                name: "Palmdale".to_string(),
                fix_type: "wpt".to_string(),
                lat: 34.5,
                lon: -118.0,
                altitude_ft: 15000.0,
                ground_height_ft: 2500.0,
                time_total_sec: 600.0,
                time_leg_sec: 600.0,
                ground_speed_kt: 450.0,
            }],
        };

        let zoom = dz.zoom_for_position_with_simbrief(34.4, -118.0, 5000.0, Some(&plan), 50.0);
        assert_eq!(zoom, 16);
    }

    #[test]
    fn test_zoom_for_position_with_simbrief_fallback_to_dataref() {
        use crate::xplane::simbrief::{FlightFix, FlightPlan};

        let dz = DynamicZoom::default();
        let plan = FlightPlan {
            origin: "KLAX".to_string(),
            destination: "KSFO".to_string(),
            origin_elevation_ft: 126.0,
            destination_elevation_ft: 8.0,
            cruise_altitude_ft: 35000.0,
            fixes: vec![FlightFix {
                ident: "KLAX".to_string(),
                name: "Los Angeles Intl".to_string(),
                fix_type: "apt".to_string(),
                lat: 33.9425,
                lon: -118.4081,
                altitude_ft: 1500.0,
                ground_height_ft: 126.0,
                time_total_sec: 0.0,
                time_leg_sec: 0.0,
                ground_speed_kt: 0.0,
            }],
        };

        let zoom = dz.zoom_for_position_with_simbrief(40.0, -120.0, 8000.0, Some(&plan), 50.0);
        assert_eq!(zoom, 19);
    }

    #[test]
    fn test_zoom_for_position_with_simbrief_no_plan() {
        let dz = DynamicZoom::default();

        let zoom = dz.zoom_for_position_with_simbrief(34.0, -118.0, 5000.0, None, 50.0);
        assert_eq!(zoom, 19);
    }
}
