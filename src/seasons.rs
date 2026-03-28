// SPDX-License-Identifier: Apache-2.0 OR GPL-3.0
// Copyright (c) 2024 the AutoOrtho contributors

use crate::config::Season;
use chrono::{Datelike, Local};

/// Seasonal color saturation adjustment
pub struct SeasonalAdjustment {
    enabled: bool,
    season: Season,
    spring_sat: f32,
    summer_sat: f32,
    autumn_sat: f32,
    winter_sat: f32,
}

impl Default for SeasonalAdjustment {
    fn default() -> Self {
        Self {
            enabled: false,
            season: Season::Disabled,
            spring_sat: 0.70,
            summer_sat: 1.0,
            autumn_sat: 0.80,
            winter_sat: 0.55,
        }
    }
}

impl SeasonalAdjustment {
    pub fn new(season: Season, spring: f32, summer: f32, autumn: f32, winter: f32) -> Self {
        let enabled = season != Season::Disabled;
        Self {
            enabled,
            season,
            spring_sat: spring.clamp(0.0, 2.0),
            summer_sat: summer.clamp(0.0, 2.0),
            autumn_sat: autumn.clamp(0.0, 2.0),
            winter_sat: winter.clamp(0.0, 2.0),
        }
    }

    /// Check if seasonal adjustment is enabled
    pub fn is_enabled(&self) -> bool {
        self.enabled
    }

    /// Get current season (0=spring, 1=summer, 2=autumn, 3=winter)
    fn auto_season() -> u32 {
        let month = Local::now().month();
        match month {
            3..=5 => 0,  // Spring (Mar-May)
            6..=8 => 1,  // Summer (Jun-Aug)
            9..=11 => 2, // Autumn (Sep-Nov)
            _ => 3,      // Winter (Dec-Feb)
        }
    }

    /// Get the effective season (manual override or auto-detected)
    fn effective_season(&self) -> Season {
        if self.season != Season::Disabled {
            self.season
        } else {
            match Self::auto_season() {
                0 => Season::Spring,
                1 => Season::Summer,
                2 => Season::Autumn,
                3 => Season::Winter,
                _ => Season::Summer,
            }
        }
    }

    /// Get saturation multiplier for current season
    pub fn current_saturation(&self) -> f32 {
        if !self.enabled {
            return 1.0;
        }

        match self.effective_season() {
            Season::Spring => self.spring_sat,
            Season::Summer => self.summer_sat,
            Season::Autumn => self.autumn_sat,
            Season::Winter => self.winter_sat,
            Season::Disabled => 1.0,
        }
    }

    /// Get saturation for specific month (1-12)
    pub fn saturation_for_month(&self, month: u32) -> f32 {
        if !self.enabled {
            return 1.0;
        }

        match month {
            3..=5 => self.spring_sat,
            6..=8 => self.summer_sat,
            9..=11 => self.autumn_sat,
            _ => self.winter_sat,
        }
    }

    /// Apply saturation to RGB color (returns new RGB)
    pub fn apply_to_rgb(rgb: (u8, u8, u8), saturation: f32) -> (u8, u8, u8) {
        let r = rgb.0 as f32 / 255.0;
        let g = rgb.1 as f32 / 255.0;
        let b = rgb.2 as f32 / 255.0;

        let max = r.max(g).max(b);
        let min = r.min(g).min(b);
        let l = (max + min) / 2.0;

        let s = if max == min {
            0.0
        } else if l < 0.5 {
            (max - min) / (max + min)
        } else {
            (max - min) / (2.0 - max - min)
        };

        let h = if max == min {
            0.0
        } else if max == r {
            ((g - b) / (max - min) + if g < b { 6.0 } else { 0.0 }) / 6.0
        } else if max == g {
            ((b - r) / (max - min) + 2.0) / 6.0
        } else {
            ((r - g) / (max - min) + 4.0) / 6.0
        };

        let new_s = (s * saturation).min(1.0);

        // Convert back to RGB
        let c = (1.0 - (2.0 * l - 1.0).abs()) * new_s;
        let x = c * (1.0 - ((h * 6.0) % 2.0 - 1.0).abs());
        let m = l - c / 2.0;

        let (r_, g_, b_) = match (h * 6.0) as i32 {
            0 => (c, x, 0.0),
            1 => (x, c, 0.0),
            2 => (0.0, c, x),
            3 => (0.0, x, c),
            4 => (x, 0.0, c),
            _ => (c, 0.0, x),
        };

        (
            ((r_ + m) * 255.0) as u8,
            ((g_ + m) * 255.0) as u8,
            ((b_ + m) * 255.0) as u8,
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_seasonal_adjustment_default() {
        let adj = SeasonalAdjustment::default();
        assert!(!adj.is_enabled());
        assert_eq!(adj.spring_sat, 0.70);
        assert_eq!(adj.summer_sat, 1.0);
    }

    #[test]
    fn test_seasonal_adjustment_clamp() {
        let adj = SeasonalAdjustment::new(Season::Spring, 2.5, 0.0, 1.0, 1.0);
        assert_eq!(adj.spring_sat, 2.0); // Clamped to max
        assert_eq!(adj.summer_sat, 0.0); // Clamped to min
    }

    #[test]
    fn test_seasonal_adjustment_enabled() {
        let adj_disabled = SeasonalAdjustment::new(Season::Disabled, 1.0, 1.0, 1.0, 1.0);
        assert!(!adj_disabled.is_enabled());
        assert_eq!(adj_disabled.current_saturation(), 1.0);

        let adj_summer = SeasonalAdjustment::new(Season::Summer, 1.0, 0.8, 1.0, 1.0);
        assert!(adj_summer.is_enabled());
        assert_eq!(adj_summer.current_saturation(), 0.8);
    }

    #[test]
    fn test_saturation_for_month_spring() {
        let adj = SeasonalAdjustment::new(Season::Spring, 1.1, 1.0, 0.9, 0.8);
        assert!((adj.saturation_for_month(3) - 1.1).abs() < 0.01);
        assert!((adj.saturation_for_month(4) - 1.1).abs() < 0.01);
        assert!((adj.saturation_for_month(5) - 1.1).abs() < 0.01);
    }

    #[test]
    fn test_saturation_for_month_all() {
        let adj = SeasonalAdjustment::new(Season::Disabled, 1.0, 1.0, 0.95, 0.85);

        // When disabled, always returns 1.0
        assert_eq!(adj.saturation_for_month(3), 1.0);
        assert_eq!(adj.saturation_for_month(6), 1.0);
        assert_eq!(adj.saturation_for_month(9), 1.0);
        assert_eq!(adj.saturation_for_month(1), 1.0);
    }

    #[test]
    fn test_saturation_enabled_for_month() {
        let adj = SeasonalAdjustment::new(Season::Spring, 1.1, 1.0, 0.9, 0.8);

        // Spring
        assert!((adj.saturation_for_month(3) - 1.1).abs() < 0.01);
        assert!((adj.saturation_for_month(4) - 1.1).abs() < 0.01);
        assert!((adj.saturation_for_month(5) - 1.1).abs() < 0.01);
    }

    #[test]
    fn test_apply_to_rgb_identity() {
        let rgb = (200, 100, 50);
        let result = SeasonalAdjustment::apply_to_rgb(rgb, 1.0);
        // Should be approximately equal (HSL conversions aren't perfectly reversible)
        assert!((result.0 as i32 - rgb.0 as i32).abs() <= 1);
    }

    #[test]
    fn test_apply_to_rgb_desaturate() {
        let rgb = (200, 100, 50);
        let result_full = SeasonalAdjustment::apply_to_rgb(rgb, 1.0);
        let result_half = SeasonalAdjustment::apply_to_rgb(rgb, 0.5);

        // Half saturation should be closer to gray
        let gray_dist = ((result_half.0 as i32 - result_half.1 as i32).abs()
            + (result_half.1 as i32 - result_half.2 as i32).abs()) as u32;
        let color_dist = ((result_full.0 as i32 - result_full.1 as i32).abs()
            + (result_full.1 as i32 - result_full.2 as i32).abs()) as u32;

        assert!(gray_dist < color_dist);
    }

    #[test]
    fn test_apply_to_rgb_grayscale() {
        let gray = (128, 128, 128);
        let result = SeasonalAdjustment::apply_to_rgb(gray, 0.5);
        // Gray should remain approximately gray at any saturation
        assert!((result.0 as i32 - result.1 as i32).abs() <= 1);
    }
}
