// SPDX-License-Identifier: Apache-2.0 OR GPL-3.0
// Copyright (c) 2024 the AutoOrtho contributors

use crate::config::ZoomRule;
use crate::tiles::provider::PROVIDER_INFO;

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
}
