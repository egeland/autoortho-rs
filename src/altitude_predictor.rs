// SPDX-License-Identifier: Apache-2.0 OR GPL-3.0
// Copyright (c) 2024 the AutoOrtho contributors

/// Predict altitude at closest point of approach
pub struct AltitudePredictor;

impl AltitudePredictor {
    /// Linear interpolation of altitude along a route
    /// Returns altitude at distance 't' along a path from alt1 to alt2
    /// t is in the range [0, 1] where 0 is start and 1 is end
    pub fn interpolate_altitude(alt1: f32, alt2: f32, t: f32) -> f32 {
        // Handle edge cases
        if t <= 0.0 {
            return alt1;
        }
        if t >= 1.0 {
            return alt2;
        }
        // Simple linear interpolation
        alt1 + (alt2 - alt1) * t
    }

    /// Predict altitude at a given time (t in seconds)
    /// given current altitude, current ground speed, and target waypoint altitude
    pub fn altitude_at_time(
        current_alt_ft: f32,
        target_alt_ft: f32,
        descent_rate_fpm: f32,
        time_sec: f32,
    ) -> f32 {
        if time_sec <= 0.0 || descent_rate_fpm == 0.0 {
            return current_alt_ft;
        }

        let descent_ft = descent_rate_fpm / 60.0 * time_sec;
        (current_alt_ft - descent_ft).max(target_alt_ft)
    }

    /// Calculate vertical speed needed to reach target altitude
    /// given current altitude and distance to waypoint
    pub fn vertical_speed_for_descent(
        current_alt_ft: f32,
        target_alt_ft: f32,
        distance_nm: f32,
        ground_speed_kt: f32,
    ) -> f32 {
        if ground_speed_kt <= 0.0 || distance_nm <= 0.0 {
            return 0.0;
        }

        let time_to_wp_min = (distance_nm / ground_speed_kt) * 60.0;
        let alt_diff = current_alt_ft - target_alt_ft;

        if time_to_wp_min <= 0.0 {
            0.0
        } else {
            alt_diff / time_to_wp_min
        }
    }

    /// Find altitude at closest point on route to a given point
    /// Simple version: average altitude between waypoints
    pub fn altitude_at_closest_point(wp1_alt: f32, wp2_alt: f32) -> f32 {
        // Simplified: average altitude between waypoints
        (wp1_alt + wp2_alt) / 2.0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_interpolate_altitude_start() {
        let alt = AltitudePredictor::interpolate_altitude(10000.0, 5000.0, 0.0);
        assert_eq!(alt, 10000.0);
    }

    #[test]
    fn test_interpolate_altitude_end() {
        let alt = AltitudePredictor::interpolate_altitude(10000.0, 5000.0, 1.0);
        assert_eq!(alt, 5000.0);
    }

    #[test]
    fn test_interpolate_altitude_mid() {
        let alt = AltitudePredictor::interpolate_altitude(10000.0, 6000.0, 0.5);
        assert_eq!(alt, 8000.0);
    }

    #[test]
    fn test_altitude_at_time_steady() {
        let alt = AltitudePredictor::altitude_at_time(10000.0, 5000.0, 0.0, 60.0);
        assert_eq!(alt, 10000.0); // No descent
    }

    #[test]
    fn test_altitude_at_time_zero_time() {
        let alt = AltitudePredictor::altitude_at_time(10000.0, 5000.0, 500.0, 0.0);
        assert_eq!(alt, 10000.0); // No time passed
    }

    #[test]
    fn test_altitude_at_time_descending() {
        let alt = AltitudePredictor::altitude_at_time(10000.0, 5000.0, 500.0, 10.0);
        // 500 fpm * 10 sec = 83.3 ft descent
        assert!((alt - (10000.0 - 500.0 / 6.0)).abs() < 1.0);
    }

    #[test]
    fn test_altitude_at_time_reaches_target() {
        let alt = AltitudePredictor::altitude_at_time(6000.0, 5000.0, 1000.0, 60.0);
        // Reaches target, caps at target altitude
        assert_eq!(alt, 5000.0);
    }

    #[test]
    fn test_vertical_speed_for_descent() {
        let vs = AltitudePredictor::vertical_speed_for_descent(10000.0, 5000.0, 10.0, 100.0);
        // Distance: 10 nm, speed: 100 kt -> time = 6 min
        // Alt diff: 5000 ft, time: 6 min -> 833 fpm
        assert!((vs - 833.0).abs() < 10.0);
    }

    #[test]
    fn test_vertical_speed_zero_speed() {
        let vs = AltitudePredictor::vertical_speed_for_descent(10000.0, 5000.0, 10.0, 0.0);
        assert_eq!(vs, 0.0);
    }

    #[test]
    fn test_vertical_speed_zero_distance() {
        let vs = AltitudePredictor::vertical_speed_for_descent(10000.0, 5000.0, 0.0, 100.0);
        assert_eq!(vs, 0.0);
    }

    #[test]
    fn test_altitude_at_closest_point() {
        let alt = AltitudePredictor::altitude_at_closest_point(10000.0, 5000.0);
        assert_eq!(alt, 7500.0);
    }
}
