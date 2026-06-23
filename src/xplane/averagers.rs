// SPDX-License-Identifier: Apache-2.0 OR GPL-3.0
// Copyright (c) 2026 the AutoOrtho contributors

use std::collections::VecDeque;

/// Flight data averager (sliding window)
#[derive(Debug)]
pub struct FlightDataAverager {
    window_size: usize,
    data: VecDeque<f32>,
}

impl FlightDataAverager {
    pub fn new(window_size: usize) -> Self {
        Self {
            window_size,
            data: VecDeque::with_capacity(window_size),
        }
    }

    /// Add data point and return average
    pub fn add(&mut self, value: f32) -> f32 {
        self.data.push_back(value);

        if self.data.len() > self.window_size {
            self.data.pop_front();
        }

        let sum: f32 = self.data.iter().sum();
        sum / self.data.len() as f32
    }

    pub fn average(&self) -> f32 {
        if self.data.is_empty() {
            0.0
        } else {
            let sum: f32 = self.data.iter().sum();
            sum / self.data.len() as f32
        }
    }

    pub fn count(&self) -> usize {
        self.data.len()
    }

    pub fn clear(&mut self) {
        self.data.clear();
    }
}

/// Heading averaging with wrap-around (0-360)
#[derive(Debug)]
pub struct HeadingAverager {
    window_size: usize,
    headings: VecDeque<f32>,
}

impl HeadingAverager {
    pub fn new(window_size: usize) -> Self {
        Self {
            window_size,
            headings: VecDeque::with_capacity(window_size),
        }
    }

    /// Add heading and return wrapped average
    pub fn add(&mut self, heading: f32) -> f32 {
        self.headings.push_back(heading % 360.0);

        if self.headings.len() > self.window_size {
            self.headings.pop_front();
        }

        self.average()
    }

    pub fn average(&self) -> f32 {
        if self.headings.is_empty() {
            0.0
        } else {
            // Use circular average
            let sum_cos: f32 = self.headings.iter().map(|h| h.to_radians().cos()).sum();
            let sum_sin: f32 = self.headings.iter().map(|h| h.to_radians().sin()).sum();

            let avg_rad = sum_sin.atan2(sum_cos);
            let avg_deg = avg_rad.to_degrees();

            if avg_deg < 0.0 {
                avg_deg + 360.0
            } else {
                avg_deg
            }
        }
    }

    pub fn clear(&mut self) {
        self.headings.clear();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_flight_data_averager_single() {
        let mut avg = FlightDataAverager::new(5);
        let result = avg.add(10.0);
        assert!((result - 10.0).abs() < 0.001);
    }

    #[test]
    fn test_flight_data_averager_multiple() {
        let mut avg = FlightDataAverager::new(5);
        avg.add(10.0);
        avg.add(20.0);
        let result = avg.add(30.0);
        assert!((result - 20.0).abs() < 0.001);
    }

    #[test]
    fn test_flight_data_averager_window() {
        let mut avg = FlightDataAverager::new(3);
        avg.add(10.0);
        avg.add(20.0);
        avg.add(30.0);
        assert_eq!(avg.count(), 3);

        avg.add(40.0);
        assert_eq!(avg.count(), 3);
    }

    #[test]
    fn test_heading_averager_north() {
        let mut avg = HeadingAverager::new(3);
        let _h1 = avg.add(350.0);
        let _h2 = avg.add(10.0);
        let h3 = avg.add(0.0);
        assert!(h3 < 10.0 || h3 > 350.0);
    }

    #[test]
    fn test_heading_averager_wrap() {
        let mut avg = HeadingAverager::new(2);
        avg.add(359.0);
        let result = avg.add(1.0);
        assert!(result < 10.0 || result > 350.0);
    }

    #[test]
    fn test_heading_averager_single() {
        let mut avg = HeadingAverager::new(1);
        let result = avg.add(45.0);
        assert!((result - 45.0).abs() < 1.0);
    }
}
