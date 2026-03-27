// SPDX-License-Identifier: Apache-2.0 OR GPL-3.0
// Copyright (c) 2024 the AutoOrtho contributors

use chrono::{Datelike, Local, Timelike};

/// Sun elevation-based night exclusion
pub struct TimeExclusion {
    night_threshold: f32, // degrees (typically -12 for nautical twilight)
    day_threshold: f32,   // degrees (typically -10 for civil twilight)
}

impl TimeExclusion {
    pub fn new(night_threshold: f32, day_threshold: f32) -> Self {
        Self {
            night_threshold,
            day_threshold,
        }
    }

    /// Get current sun pitch (simplified, not astronomical)
    /// Returns degrees above horizon (-90 to 90)
    /// Negative = below horizon (night)
    pub fn current_sun_pitch() -> f32 {
        let now = Local::now();
        let hour = now.hour() as f32;

        // Simplified: sun is highest at noon, lowest at midnight
        // Peak sun pitch varies by season (+23.5 to -23.5 degrees)
        let month = now.month() as f32;
        let peak_declination = 23.5 * ((month - 3.0) / 12.0 * 360.0).to_radians().sin();

        // Hour angle: 0 at noon, ±90 at sunrise/sunset, ±180 at midnight
        let hour_angle = (hour - 12.0) * 15.0; // degrees

        // Simplified sun pitch calculation
        (hour_angle).to_radians().cos() * peak_declination
    }

    /// Check if it's night (sun below night threshold)
    pub fn is_night(&self, sun_pitch: f32) -> bool {
        sun_pitch <= self.night_threshold
    }

    /// Check if it's day (sun above day threshold)
    pub fn is_day(&self, sun_pitch: f32) -> bool {
        sun_pitch >= self.day_threshold
    }

    /// Get time of day: 0 = night, 1 = day, 0.5 = twilight
    pub fn day_phase(&self, sun_pitch: f32) -> f32 {
        if sun_pitch < self.night_threshold {
            0.0
        } else if sun_pitch > self.day_threshold {
            1.0
        } else {
            // Linear interpolation in twilight
            (sun_pitch - self.night_threshold) / (self.day_threshold - self.night_threshold)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_time_exclusion_creation() {
        let te = TimeExclusion::new(-12.0, -10.0);
        assert_eq!(te.night_threshold, -12.0);
        assert_eq!(te.day_threshold, -10.0);
    }

    #[test]
    fn test_is_night() {
        let te = TimeExclusion::new(-12.0, -10.0);
        assert!(te.is_night(-20.0));
        assert!(te.is_night(-12.0));
        assert!(!te.is_night(-11.0));
    }

    #[test]
    fn test_is_day() {
        let te = TimeExclusion::new(-12.0, -10.0);
        assert!(te.is_day(-9.0));
        assert!(te.is_day(-10.0));
        assert!(te.is_day(0.0));
        assert!(te.is_day(45.0));
        assert!(!te.is_day(-15.0));
    }

    #[test]
    fn test_day_phase_night() {
        let te = TimeExclusion::new(-12.0, -10.0);
        assert_eq!(te.day_phase(-20.0), 0.0);
    }

    #[test]
    fn test_day_phase_day() {
        let te = TimeExclusion::new(-12.0, -10.0);
        assert_eq!(te.day_phase(0.0), 1.0);
    }

    #[test]
    fn test_day_phase_twilight() {
        let te = TimeExclusion::new(-12.0, -10.0);
        let phase = te.day_phase(-11.0);
        assert!(phase > 0.0 && phase < 1.0);
    }

    #[test]
    fn test_current_sun_pitch_not_nan() {
        let pitch = TimeExclusion::current_sun_pitch();
        assert!(!pitch.is_nan());
        assert!(pitch >= -90.0 && pitch <= 90.0);
    }
}
