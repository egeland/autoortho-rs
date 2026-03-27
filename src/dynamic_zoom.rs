// SPDX-License-Identifier: Apache-2.0 OR GPL-3.0
// Copyright (c) 2024 the AutoOrtho contributors

/// Altitude-based dynamic zoom level selection
pub struct DynamicZoom {
    min_zoom: u32,
    max_zoom: u32,
    near_airport_zoom: u32,
}

impl DynamicZoom {
    pub fn new(min_zoom: u32, max_zoom: u32, near_airport_zoom: u32) -> Self {
        Self {
            min_zoom: min_zoom.min(28),
            max_zoom: max_zoom.min(28),
            near_airport_zoom: near_airport_zoom.min(28),
        }
    }

    /// Get zoom level based on altitude in feet
    /// Linear scaling from min_zoom at high altitude to max_zoom at low altitude
    pub fn zoom_for_altitude(&self, altitude_ft: f32) -> u32 {
        let altitude_m = altitude_ft * 0.3048;

        // Scale: 30000m -> min_zoom, 100m -> max_zoom
        let min_alt = 100.0;
        let max_alt = 30000.0;

        let t = ((max_alt - altitude_m) / (max_alt - min_alt)).clamp(0.0, 1.0);

        let zoom_range = (self.max_zoom - self.min_zoom) as f32;
        let zoom = self.min_zoom as f32 + t * zoom_range;

        zoom.round() as u32
    }

    /// Get zoom level when near airport (lower altitude threshold)
    pub fn zoom_for_near_airport(&self) -> u32 {
        self.near_airport_zoom
    }

    /// Check if aircraft is "near airport" (below 1000 ft AGL)
    pub fn is_near_airport(&self, altitude_agl_ft: f32) -> bool {
        altitude_agl_ft < 1000.0
    }

    /// Get recommended zoom with airport consideration
    pub fn recommended_zoom(&self, altitude_ft: f32, altitude_agl_ft: f32) -> u32 {
        if self.is_near_airport(altitude_agl_ft) {
            self.near_airport_zoom
        } else {
            self.zoom_for_altitude(altitude_ft)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_dynamic_zoom_creation() {
        let dz = DynamicZoom::new(12, 16, 18);
        assert_eq!(dz.min_zoom, 12);
        assert_eq!(dz.max_zoom, 16);
        assert_eq!(dz.near_airport_zoom, 18);
    }

    #[test]
    fn test_dynamic_zoom_clamping() {
        let dz = DynamicZoom::new(50, 0, 40);
        assert_eq!(dz.min_zoom, 28);
        assert_eq!(dz.max_zoom, 0);
        assert_eq!(dz.near_airport_zoom, 28);
    }

    #[test]
    fn test_zoom_for_altitude_high() {
        let dz = DynamicZoom::new(12, 16, 18);
        let zoom = dz.zoom_for_altitude(30000.0 * 3.28084); // 30km in feet
        assert_eq!(zoom, 12); // Minimum zoom
    }

    #[test]
    fn test_zoom_for_altitude_low() {
        let dz = DynamicZoom::new(12, 16, 18);
        let zoom = dz.zoom_for_altitude(328.0); // ~100m in feet
        assert_eq!(zoom, 16); // Maximum zoom
    }

    #[test]
    fn test_zoom_for_altitude_mid() {
        let dz = DynamicZoom::new(12, 16, 18);
        let zoom = dz.zoom_for_altitude(5000.0); // 5000 feet
        assert!(zoom >= 12 && zoom <= 16);
    }

    #[test]
    fn test_is_near_airport() {
        let dz = DynamicZoom::new(12, 16, 18);
        assert!(dz.is_near_airport(500.0));
        assert!(dz.is_near_airport(999.0));
        assert!(!dz.is_near_airport(1000.0));
        assert!(!dz.is_near_airport(5000.0));
    }

    #[test]
    fn test_recommended_zoom_near_airport() {
        let dz = DynamicZoom::new(12, 16, 18);
        let zoom = dz.recommended_zoom(10000.0, 500.0); // Near airport
        assert_eq!(zoom, 18);
    }

    #[test]
    fn test_recommended_zoom_high_altitude() {
        let dz = DynamicZoom::new(12, 16, 18);
        let zoom = dz.recommended_zoom(30000.0 * 3.28084, 29500.0 * 3.28084);
        assert_eq!(zoom, 12);
    }
}
