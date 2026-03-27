//! SimBrief OFP (Operational Flight Plan) client.
//!
//! Fetches flight plans from the SimBrief API and provides route-aware
//! prefetch points for tile loading. Handles the case where SID/STAR
//! waypoints are missing from the OFP by using airport coordinates
//! and aircraft position for near-field prefetch.

use serde::Deserialize;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum SimbriefError {
    #[error("HTTP request failed: {0}")]
    HttpError(String),
    #[error("Parse error: {0}")]
    ParseError(String),
    #[error("No flight plan available")]
    NoFlightPlan,
    #[error("API error: {0}")]
    ApiError(String),
}

const SIMBRIEF_API_URL: &str = "https://www.simbrief.com/api/xml.fetcher.php";
const API_TIMEOUT: std::time::Duration = std::time::Duration::from_secs(15);

/// A waypoint (fix) from the SimBrief navlog.
#[derive(Debug, Clone)]
pub struct FlightFix {
    /// Fix identifier (e.g., "KLAX", "BOACH", "RW25L")
    pub ident: String,
    /// Full name
    pub name: String,
    /// Fix type: "apt", "wpt", "vor", "ndb", "ltlg"
    pub fix_type: String,
    /// Latitude in degrees
    pub lat: f64,
    /// Longitude in degrees
    pub lon: f64,
    /// MSL altitude in feet
    pub altitude_ft: f32,
    /// Terrain elevation in feet
    pub ground_height_ft: f32,
    /// Cumulative time from departure in seconds
    pub time_total_sec: f32,
    /// Time for this leg only in seconds
    pub time_leg_sec: f32,
    /// Planned ground speed in knots
    pub ground_speed_kt: f32,
}

impl FlightFix {
    /// Altitude above ground level in feet.
    pub fn altitude_agl_ft(&self) -> f32 {
        (self.altitude_ft - self.ground_height_ft).max(0.0)
    }
}

/// A parsed SimBrief flight plan.
#[derive(Debug, Clone)]
pub struct FlightPlan {
    /// ICAO code of origin airport
    pub origin: String,
    /// ICAO code of destination airport
    pub destination: String,
    /// Origin field elevation in feet
    pub origin_elevation_ft: f32,
    /// Destination field elevation in feet
    pub destination_elevation_ft: f32,
    /// Cruise altitude in feet
    pub cruise_altitude_ft: f32,
    /// All waypoints in order (departure → arrival)
    pub fixes: Vec<FlightFix>,
}

impl FlightPlan {
    /// Get the origin airport fix (first fix).
    pub fn origin_fix(&self) -> Option<&FlightFix> {
        self.fixes.first()
    }

    /// Get the destination airport fix (last fix).
    pub fn destination_fix(&self) -> Option<&FlightFix> {
        self.fixes.last()
    }

    /// Check if a position is within `threshold_nm` of any route segment.
    ///
    /// Uses great-circle distance to each segment for accuracy.
    pub fn is_on_route(&self, lat: f64, lon: f64, threshold_nm: f64) -> bool {
        if self.fixes.len() < 2 {
            return false;
        }

        for window in self.fixes.windows(2) {
            let dist = distance_to_segment(
                lat,
                lon,
                window[0].lat,
                window[0].lon,
                window[1].lat,
                window[1].lon,
            );
            if dist <= threshold_nm {
                return true;
            }
        }

        false
    }

    /// Get points along the route for prefetching, starting from `lookahead_start_nm`
    /// ahead of the given position, spaced `spacing_nm` apart.
    ///
    /// This is the core prefetch function. It handles missing SID/STAR by:
    /// 1. Always including departure and arrival airport positions
    /// 2. Interpolating between navlog fixes for cruise route
    /// 3. NOT relying on having waypoints near airports
    pub fn get_prefetch_points(
        &self,
        current_lat: f64,
        current_lon: f64,
        spacing_nm: f64,
        max_lookahead_sec: f32,
    ) -> Vec<PrefetchPoint> {
        if self.fixes.len() < 2 {
            return Vec::new();
        }

        let mut points = Vec::new();

        // Find closest segment to current position
        let (seg_idx, _seg_dist) = self.find_closest_segment(current_lat, current_lon);

        // Estimate current time on route by interpolating within the closest segment
        let current_time = self.interpolate_time_on_segment(seg_idx, current_lat, current_lon);

        // Walk forward through the route, generating prefetch points
        let mut accumulated_nm = 0.0;
        let remaining_fixes = &self.fixes[seg_idx..];

        if remaining_fixes.len() < 2 {
            return points;
        }

        for window in remaining_fixes.windows(2) {
            let seg_len = haversine_nm(window[0].lat, window[0].lon, window[1].lat, window[1].lon);
            if seg_len < 0.01 {
                continue;
            }

            let seg_time = window[1].time_total_sec - window[0].time_total_sec;

            // Generate points along this segment
            let mut d = 0.0;
            while d < seg_len {
                let frac = d / seg_len;
                let lat = window[0].lat + (window[1].lat - window[0].lat) * frac;
                let lon = window[0].lon + (window[1].lon - window[0].lon) * frac;
                let alt = window[0].altitude_ft
                    + (window[1].altitude_ft - window[0].altitude_ft) * frac as f32;
                let ground = window[0].ground_height_ft
                    + (window[1].ground_height_ft - window[0].ground_height_ft) * frac as f32;
                let time = window[0].time_total_sec + seg_time * frac as f32;

                let time_to_reach = time - current_time;
                if time_to_reach > max_lookahead_sec {
                    return points;
                }

                if time_to_reach >= 0.0 {
                    points.push(PrefetchPoint {
                        lat,
                        lon,
                        altitude_ft: alt,
                        ground_height_ft: ground,
                        time_to_reach_sec: time_to_reach,
                        distance_along_route_nm: accumulated_nm + d,
                    });
                }

                d += spacing_nm;
            }

            accumulated_nm += seg_len;
        }

        // Always include destination airport vicinity (handles missing STAR)
        if let Some(dest) = self.destination_fix() {
            let dest_dist = haversine_nm(current_lat, current_lon, dest.lat, dest.lon);
            if dest_dist > spacing_nm {
                points.push(PrefetchPoint {
                    lat: dest.lat,
                    lon: dest.lon,
                    altitude_ft: dest.altitude_ft,
                    ground_height_ft: dest.ground_height_ft,
                    time_to_reach_sec: dest.time_total_sec - current_time,
                    distance_along_route_nm: accumulated_nm,
                });
            }
        }

        points
    }

    fn find_closest_segment(&self, lat: f64, lon: f64) -> (usize, f64) {
        let mut best_idx = 0;
        let mut best_dist = f64::MAX;

        for (i, window) in self.fixes.windows(2).enumerate() {
            let dist = distance_to_segment(
                lat,
                lon,
                window[0].lat,
                window[0].lon,
                window[1].lat,
                window[1].lon,
            );
            if dist < best_dist {
                best_dist = dist;
                best_idx = i;
            }
        }

        (best_idx, best_dist)
    }

    fn interpolate_time_on_segment(&self, seg_idx: usize, lat: f64, lon: f64) -> f32 {
        if seg_idx + 1 >= self.fixes.len() {
            return self.fixes.last().map(|f| f.time_total_sec).unwrap_or(0.0);
        }

        let a = &self.fixes[seg_idx];
        let b = &self.fixes[seg_idx + 1];
        let seg_len = haversine_nm(a.lat, a.lon, b.lat, b.lon);
        if seg_len < 0.01 {
            return a.time_total_sec;
        }

        let dist_from_a = haversine_nm(a.lat, a.lon, lat, lon);
        let frac = (dist_from_a / seg_len).clamp(0.0, 1.0);

        a.time_total_sec + (b.time_total_sec - a.time_total_sec) * frac as f32
    }
}

/// A point along the route for tile prefetching.
#[derive(Debug, Clone)]
pub struct PrefetchPoint {
    pub lat: f64,
    pub lon: f64,
    pub altitude_ft: f32,
    pub ground_height_ft: f32,
    /// Estimated seconds until aircraft reaches this point
    pub time_to_reach_sec: f32,
    /// Distance from the start of the remaining route
    pub distance_along_route_nm: f64,
}

impl PrefetchPoint {
    pub fn altitude_agl_ft(&self) -> f32 {
        (self.altitude_ft - self.ground_height_ft).max(0.0)
    }
}

/// Fetch a flight plan from SimBrief.
pub async fn fetch_flight_plan(user_id: &str) -> Result<FlightPlan, SimbriefError> {
    let url = format!("{}?userid={}&json=1", SIMBRIEF_API_URL, user_id);

    let client = reqwest::Client::builder()
        .timeout(API_TIMEOUT)
        .build()
        .map_err(|e| SimbriefError::HttpError(e.to_string()))?;

    let response = client
        .get(&url)
        .send()
        .await
        .map_err(|e| SimbriefError::HttpError(e.to_string()))?;

    let body: SimbriefResponse = response
        .json()
        .await
        .map_err(|e| SimbriefError::ParseError(e.to_string()))?;

    // Check for API errors
    if let Some(fetch) = &body.fetch
        && let Some(status) = &fetch.status
        && status.starts_with("Error")
    {
        return Err(SimbriefError::ApiError(status.clone()));
    }

    let origin = body
        .origin
        .as_ref()
        .map(|o| o.icao_code.clone())
        .unwrap_or_default();
    let origin_elevation_ft = body
        .origin
        .as_ref()
        .and_then(|o| o.elevation.as_ref())
        .and_then(|e| e.parse::<f32>().ok())
        .unwrap_or(0.0);
    let destination = body
        .destination
        .as_ref()
        .map(|d| d.icao_code.clone())
        .unwrap_or_default();
    let destination_elevation_ft = body
        .destination
        .as_ref()
        .and_then(|d| d.elevation.as_ref())
        .and_then(|e| e.parse::<f32>().ok())
        .unwrap_or(0.0);
    let cruise_alt = body
        .general
        .as_ref()
        .and_then(|g| g.initial_altitude.as_ref())
        .and_then(|a| a.parse::<f32>().ok())
        .unwrap_or(35000.0);

    let fixes: Vec<FlightFix> = body
        .navlog
        .and_then(|n| n.fix)
        .unwrap_or_default()
        .into_iter()
        .filter_map(|f| {
            let lat = f.pos_lat.parse::<f64>().ok()?;
            let lon = f.pos_long.parse::<f64>().ok()?;
            Some(FlightFix {
                ident: f.ident,
                name: f.name.unwrap_or_default(),
                fix_type: f.fix_type.unwrap_or_default(),
                lat,
                lon,
                altitude_ft: f.altitude_feet.unwrap_or_default().parse().unwrap_or(0.0),
                ground_height_ft: f.ground_height.unwrap_or_default().parse().unwrap_or(0.0),
                time_total_sec: f.time_total.unwrap_or_default().parse().unwrap_or(0.0),
                time_leg_sec: f.time_leg.unwrap_or_default().parse().unwrap_or(0.0),
                ground_speed_kt: f.groundspeed.unwrap_or_default().parse().unwrap_or(0.0),
            })
        })
        .collect();

    if fixes.is_empty() {
        return Err(SimbriefError::NoFlightPlan);
    }

    Ok(FlightPlan {
        origin,
        destination,
        origin_elevation_ft,
        destination_elevation_ft,
        cruise_altitude_ft: cruise_alt,
        fixes,
    })
}

// --- SimBrief JSON response types ---

#[derive(Debug, Deserialize)]
struct SimbriefResponse {
    origin: Option<SimbriefAirport>,
    destination: Option<SimbriefAirport>,
    general: Option<SimbriefGeneral>,
    navlog: Option<SimbriefNavlog>,
    fetch: Option<SimbriefFetch>,
}

#[derive(Debug, Deserialize)]
struct SimbriefAirport {
    icao_code: String,
    elevation: Option<String>,
}

#[derive(Debug, Deserialize)]
struct SimbriefGeneral {
    initial_altitude: Option<String>,
}

#[derive(Debug, Deserialize)]
struct SimbriefNavlog {
    fix: Option<Vec<SimbriefFix>>,
}

#[derive(Debug, Deserialize)]
struct SimbriefFix {
    ident: String,
    name: Option<String>,
    #[serde(rename = "type")]
    fix_type: Option<String>,
    pos_lat: String,
    pos_long: String,
    altitude_feet: Option<String>,
    ground_height: Option<String>,
    time_total: Option<String>,
    time_leg: Option<String>,
    groundspeed: Option<String>,
}

#[derive(Debug, Deserialize)]
struct SimbriefFetch {
    status: Option<String>,
}

// --- Great-circle math ---

/// Haversine distance between two points in nautical miles.
pub fn haversine_nm(lat1: f64, lon1: f64, lat2: f64, lon2: f64) -> f64 {
    let r_nm = 3440.065; // Earth radius in nautical miles

    let dlat = (lat2 - lat1).to_radians();
    let dlon = (lon2 - lon1).to_radians();
    let lat1_r = lat1.to_radians();
    let lat2_r = lat2.to_radians();

    let a = (dlat / 2.0).sin().powi(2) + lat1_r.cos() * lat2_r.cos() * (dlon / 2.0).sin().powi(2);
    let c = 2.0 * a.sqrt().asin();

    r_nm * c
}

/// Distance from a point to a great-circle segment (A→B) in nautical miles.
/// Uses cross-track distance with along-track clamping.
fn distance_to_segment(lat: f64, lon: f64, lat_a: f64, lon_a: f64, lat_b: f64, lon_b: f64) -> f64 {
    let d_ap = haversine_nm(lat_a, lon_a, lat, lon);
    let d_ab = haversine_nm(lat_a, lon_a, lat_b, lon_b);

    if d_ab < 0.01 {
        return d_ap; // Degenerate segment
    }

    let d_bp = haversine_nm(lat_b, lon_b, lat, lon);

    // Check if point projects onto the segment
    // Using the cosine rule approximation
    let cos_a = if d_ap > 0.001 {
        ((d_ap * d_ap + d_ab * d_ab - d_bp * d_bp) / (2.0 * d_ap * d_ab)).clamp(-1.0, 1.0)
    } else {
        return 0.0; // Point is at A
    };

    let along_track = d_ap * cos_a;

    if along_track < 0.0 {
        d_ap // Before segment start
    } else if along_track > d_ab {
        d_bp // After segment end
    } else {
        // Cross-track distance
        (d_ap * d_ap - along_track * along_track).max(0.0).sqrt()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_fixes() -> Vec<FlightFix> {
        vec![
            FlightFix {
                ident: "KLAX".into(),
                name: "Los Angeles".into(),
                fix_type: "apt".into(),
                lat: 33.9425,
                lon: -118.4081,
                altitude_ft: 126.0,
                ground_height_ft: 126.0,
                time_total_sec: 0.0,
                time_leg_sec: 0.0,
                ground_speed_kt: 0.0,
            },
            FlightFix {
                ident: "BOACH".into(),
                name: "BOACH".into(),
                fix_type: "wpt".into(),
                lat: 34.5,
                lon: -117.5,
                altitude_ft: 25000.0,
                ground_height_ft: 2000.0,
                time_total_sec: 600.0,
                time_leg_sec: 600.0,
                ground_speed_kt: 350.0,
            },
            FlightFix {
                ident: "KLAS".into(),
                name: "Las Vegas".into(),
                fix_type: "apt".into(),
                lat: 36.08,
                lon: -115.15,
                altitude_ft: 2181.0,
                ground_height_ft: 2181.0,
                time_total_sec: 2400.0,
                time_leg_sec: 1800.0,
                ground_speed_kt: 300.0,
            },
        ]
    }

    fn sample_plan() -> FlightPlan {
        FlightPlan {
            origin: "KLAX".into(),
            destination: "KLAS".into(),
            origin_elevation_ft: 126.0,
            destination_elevation_ft: 2181.0,
            cruise_altitude_ft: 35000.0,
            fixes: sample_fixes(),
        }
    }

    #[test]
    fn test_haversine_zero() {
        assert!(haversine_nm(0.0, 0.0, 0.0, 0.0) < 0.001);
    }

    #[test]
    fn test_haversine_known_distance() {
        // LAX to LAS is roughly 230nm
        let dist = haversine_nm(33.94, -118.41, 36.08, -115.15);
        assert!(dist > 200.0 && dist < 260.0, "LAX-LAS distance: {}", dist);
    }

    #[test]
    fn test_is_on_route_near() {
        let plan = sample_plan();
        // Point near BOACH waypoint
        assert!(plan.is_on_route(34.5, -117.5, 40.0));
    }

    #[test]
    fn test_is_on_route_far() {
        let plan = sample_plan();
        // Point far from route
        assert!(!plan.is_on_route(45.0, -90.0, 40.0));
    }

    #[test]
    fn test_is_on_route_between_fixes() {
        let plan = sample_plan();
        // Point between BOACH and KLAS (should be on route)
        assert!(plan.is_on_route(35.3, -116.3, 40.0));
    }

    #[test]
    fn test_origin_destination() {
        let plan = sample_plan();
        assert_eq!(plan.origin_fix().unwrap().ident, "KLAX");
        assert_eq!(plan.destination_fix().unwrap().ident, "KLAS");
    }

    #[test]
    fn test_prefetch_points_generated() {
        let plan = sample_plan();
        let points = plan.get_prefetch_points(33.94, -118.41, 10.0, 3600.0);
        assert!(!points.is_empty());

        // Points should have increasing time_to_reach
        for window in points.windows(2) {
            assert!(window[1].time_to_reach_sec >= window[0].time_to_reach_sec);
        }
    }

    #[test]
    fn test_prefetch_max_lookahead_respected() {
        let plan = sample_plan();
        // Very short lookahead — should not get all points
        let points = plan.get_prefetch_points(33.94, -118.41, 10.0, 300.0);
        let all_points = plan.get_prefetch_points(33.94, -118.41, 10.0, 9999.0);
        assert!(points.len() < all_points.len());
    }

    #[test]
    fn test_prefetch_includes_destination() {
        let plan = sample_plan();
        let points = plan.get_prefetch_points(33.94, -118.41, 10.0, 9999.0);
        // Last point should be near destination
        let last = points.last().unwrap();
        let dist = haversine_nm(last.lat, last.lon, 36.08, -115.15);
        assert!(dist < 20.0, "Last point distance to KLAS: {}", dist);
    }

    #[test]
    fn test_flight_fix_agl() {
        let fix = FlightFix {
            ident: "TEST".into(),
            name: "".into(),
            fix_type: "wpt".into(),
            lat: 0.0,
            lon: 0.0,
            altitude_ft: 10000.0,
            ground_height_ft: 2000.0,
            time_total_sec: 0.0,
            time_leg_sec: 0.0,
            ground_speed_kt: 0.0,
        };
        assert!((fix.altitude_agl_ft() - 8000.0).abs() < 0.1);
    }

    #[test]
    fn test_distance_to_segment_at_endpoint() {
        let dist = distance_to_segment(33.94, -118.41, 33.94, -118.41, 36.08, -115.15);
        assert!(dist < 0.1);
    }

    #[test]
    fn test_empty_plan_is_on_route() {
        let plan = FlightPlan {
            origin: "".into(),
            destination: "".into(),
            origin_elevation_ft: 0.0,
            destination_elevation_ft: 0.0,
            cruise_altitude_ft: 0.0,
            fixes: vec![],
        };
        assert!(!plan.is_on_route(0.0, 0.0, 100.0));
    }
}
