// SPDX-License-Identifier: Apache-2.0 OR GPL-3.0
// Copyright (c) 2026 the AutoOrtho contributors

//! Route and waypoint definitions for the X-Plane simulator.

use std::fmt;

/// A single waypoint along a route.
#[derive(Debug, Clone)]
pub struct Waypoint {
    pub name: String,
    pub lat: f64,
    pub lon: f64,
}

/// A flight route consisting of multiple waypoints.
#[derive(Debug, Clone)]
pub struct Route {
    pub name: String,
    pub waypoints: Vec<Waypoint>,
}

impl Route {
    /// Parse a route specification string like "YSSY→YMML" or "YSSY→YCB→YMML"
    pub fn from_spec(spec: &str) -> Option<Self> {
        let parts: Vec<&str> = spec.split('→').collect();
        if parts.len() < 2 {
            return None;
        }

        let waypoints: Vec<Waypoint> = parts
            .iter()
            .filter_map(|name| airport_waypoint(name.trim()))
            .collect();

        if waypoints.len() < 2 {
            return None;
        }

        Some(Route {
            name: spec.to_string(),
            waypoints,
        })
    }

    /// Get the route name for display.
    pub fn name(&self) -> &str {
        &self.name
    }

    /// Interpolate additional waypoints along the route for tile sampling.
    /// Adds `samples` intermediate points between each pair of waypoints.
    pub fn interpolate_waypoints(&self, samples: usize) -> Vec<Waypoint> {
        let mut result = vec![self.waypoints[0].clone()];

        for window in self.waypoints.windows(2) {
            let from = &window[0];
            let to = &window[1];

            for i in 1..=samples {
                let t = i as f64 / (samples + 1) as f64;
                let lat = from.lat + (to.lat - from.lat) * t;
                let lon = from.lon + (to.lon - from.lon) * t;
                let name = if i == samples {
                    to.name.clone()
                } else {
                    String::new()
                };

                result.push(Waypoint { name, lat, lon });
            }
        }

        result
    }
}

impl fmt::Display for Route {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}: ", self.name)?;
        for (i, wp) in self.waypoints.iter().enumerate() {
            if i > 0 {
                write!(f, " → ")?;
            }
            write!(f, "{} ({:.4}, {:.4})", wp.name, wp.lat, wp.lon)?;
        }
        Ok(())
    }
}

/// Get a waypoint by ICAO code.
/// Returns None for unknown airports.
fn airport_waypoint(icao: &str) -> Option<Waypoint> {
    let entry = AIRPORTS.iter().find(|(code, _, _)| *code == icao)?;
    Some(Waypoint {
        name: entry.0.to_string(),
        lat: entry.1,
        lon: entry.2,
    })
}

/// Helper to build an airport tuple.
const fn airport(code: &'static str, lat: f64, lon: f64) -> (&'static str, f64, f64) {
    (code, lat, lon)
}

/// Known airport ICAO codes with coordinates.
const AIRPORTS: &[(&str, f64, f64)] = &[
    // Australia
    airport("YSSY", -33.9399, 151.1753), // Sydney
    airport("YMML", -37.6690, 144.8410), // Melbourne
    airport("YBR", -27.3845, 153.1172),  // Brisbane
    airport("YPPH", -31.9403, 115.9672), // Perth
    airport("YADL", -34.9432, 138.5008), // Adelaide
    airport("YBBN", -27.3942, 153.1218), // Brisbane
    airport("YSCB", -35.3075, 149.1944), // Canberra
    airport("YNTN", -32.1608, 148.2456), // Narrabri
    airport("YLD", -33.7052, 150.0249),  // Lithgow
    // USA
    airport("KJFK", 40.6413, -73.7781),  // New York JFK
    airport("KLAX", 33.9425, -118.4081), // Los Angeles
    airport("KSFO", 37.6213, -122.3790), // San Francisco
    airport("KORD", 41.9742, -87.9073),  // Chicago O'Hare
    // Europe
    airport("EGLL", 51.4700, -0.4543), // London Heathrow
    airport("LFPG", 49.0097, 2.5479),  // Paris CDG
    airport("EDDF", 50.0379, 8.5622),  // Frankfurt
    airport("LEMD", 40.4983, -3.5676), // Madrid
];

#[cfg(test)]
mod tests {
    use super::*;
    use autoortho_lib::tiles::coords::TileCoords;

    #[test]
    fn test_route_yssy_ymml() {
        let route = Route::from_spec("YSSY→YMML").unwrap();
        assert_eq!(route.name, "YSSY→YMML");
        assert_eq!(route.waypoints.len(), 2);
        assert_eq!(route.waypoints[0].name, "YSSY");
        assert_eq!(route.waypoints[1].name, "YMML");
    }

    #[test]
    fn test_route_interpolation() {
        let route = Route::from_spec("YSSY→YMML").unwrap();
        let waypoints = route.interpolate_waypoints(5);
        // 2 airports + 5 samples between them = 6 total
        assert_eq!(waypoints.len(), 6);
        // First is YSSY
        assert_eq!(waypoints[0].name, "YSSY");
        // Last is YMML
        assert_eq!(waypoints[5].name, "YMML");
    }

    #[test]
    fn test_route_unknown_airport() {
        // Unknown airports should return None
        let result = Route::from_spec("YXXX→YZZZ");
        assert!(result.is_none());
    }

    #[test]
    fn test_route_single_waypoint() {
        // Only one airport - should fail
        let result = Route::from_spec("YSSY");
        assert!(result.is_none());
    }

    #[test]
    fn test_tile_coords_for_route() {
        let route = Route::from_spec("YSSY→YMML").unwrap();
        let waypoints = route.interpolate_waypoints(10);

        // All waypoints should have valid tile coords at zoom 12
        for wp in &waypoints {
            let result = TileCoords::latlng_to_tile(wp.lat, wp.lon, 12);
            assert!(
                result.is_ok(),
                "Failed for {} at ({}, {})",
                wp.name,
                wp.lat,
                wp.lon
            );
        }
    }

    #[test]
    fn test_route_display() {
        let route = Route::from_spec("YSSY→YMML").unwrap();
        let display = format!("{}", route);
        assert!(display.contains("YSSY"));
        assert!(display.contains("YMML"));
    }
}
