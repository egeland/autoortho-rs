use std::f64::consts::PI;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum CoordError {
    #[error("Invalid latitude: {0}")]
    InvalidLatitude(f64),
    #[error("Invalid longitude: {0}")]
    InvalidLongitude(f64),
    #[error("Invalid zoom level: {0}")]
    InvalidZoom(u32),
}

/// Convert lat/lon to Web Mercator (slippy) tile coordinates
pub struct TileCoords;

impl TileCoords {
    /// Clamp latitude to valid Web Mercator range
    #[allow(dead_code)]
    fn clamp_latitude(lat: f64) -> f64 {
        let max = 85.051_129;
        lat.max(-max).min(max)
    }

    /// Convert lat/lon to tile x/y at given zoom level
    /// Returns (tile_col, tile_row)
    pub fn latlng_to_tile(lat: f64, lon: f64, zoom: u32) -> Result<(u32, u32), CoordError> {
        if !(-180.0..=180.0).contains(&lon) {
            return Err(CoordError::InvalidLongitude(lon));
        }
        if !(-85.051_129..=85.051_129).contains(&lat) {
            return Err(CoordError::InvalidLatitude(lat));
        }
        if zoom > 28 {
            return Err(CoordError::InvalidZoom(zoom));
        }

        let n = 2_f64.powi(zoom as i32);
        let x = ((lon + 180.0) / 360.0 * n).floor() as u32;

        let lat_rad = lat.to_radians();
        let y = ((1.0 - (lat_rad.tan() + 1.0 / lat_rad.cos()).ln() / PI) / 2.0 * n).floor() as u32;

        Ok((x, y))
    }

    /// Convert tile x/y to lat/lon at center of tile
    pub fn tile_to_latlng(col: u32, row: u32, zoom: u32) -> Result<(f64, f64), CoordError> {
        if zoom > 28 {
            return Err(CoordError::InvalidZoom(zoom));
        }

        let n = 2_f64.powi(zoom as i32);

        let lon = col as f64 / n * 360.0 - 180.0;

        let y_norm = (row as f64 + 0.5) / n;
        let lat_rad = (PI * (1.0 - 2.0 * y_norm)).sinh().atan();
        let lat = lat_rad.to_degrees();

        Ok((lat, lon))
    }

    /// Convert lat/lon to Bing Maps quadkey
    pub fn latlng_to_quadkey(lat: f64, lon: f64, zoom: u32) -> Result<String, CoordError> {
        let (col, row) = Self::latlng_to_tile(lat, lon, zoom)?;
        Ok(Self::tile_to_quadkey(col, row, zoom))
    }

    /// Convert tile coords to Bing Maps quadkey
    pub fn tile_to_quadkey(col: u32, row: u32, zoom: u32) -> String {
        let mut quadkey = String::new();
        for z in (0..zoom).rev() {
            let mut digit = 0;
            let mask = 1 << z;
            if (col & mask) != 0 {
                digit += 1;
            }
            if (row & mask) != 0 {
                digit += 2;
            }
            quadkey.push(char::from_digit(digit, 10).expect("quadkey digit 0-3 is valid"));
        }
        quadkey
    }

    /// Convert Bing Maps quadkey to tile coords
    pub fn quadkey_to_tile(quadkey: &str) -> Result<(u32, u32, u32), CoordError> {
        let zoom = quadkey.len() as u32;
        let mut col = 0u32;
        let mut row = 0u32;

        for (i, c) in quadkey.chars().enumerate() {
            let z = zoom - 1 - i as u32;
            let digit = c.to_digit(10).ok_or(CoordError::InvalidZoom(zoom))?;

            if (digit & 1) != 0 {
                col |= 1 << z;
            }
            if (digit & 2) != 0 {
                row |= 1 << z;
            }
        }

        Ok((col, row, zoom))
    }

    /// Get tile bounds in lat/lon
    pub fn tile_bounds(col: u32, row: u32, zoom: u32) -> Result<(f64, f64, f64, f64), CoordError> {
        let (lat_n, lon_w) = Self::tile_to_latlng(col, row, zoom)?;
        let (lat_s, lon_e) = Self::tile_to_latlng(col + 1, row + 1, zoom)?;
        Ok((lat_n, lon_w, lat_s, lon_e))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_latlng_to_tile_simple() {
        // San Francisco at zoom 10
        let (col, row) = TileCoords::latlng_to_tile(37.7749, -122.4194, 10).unwrap();
        assert!(col > 0);
        assert!(row > 0);
    }

    #[test]
    fn test_latlng_to_tile_equator() {
        // Equator and prime meridian
        let (col, row) = TileCoords::latlng_to_tile(0.0, 0.0, 10).unwrap();
        assert_eq!(col, 512); // Center tile at zoom 10
        assert_eq!(row, 512);
    }

    #[test]
    fn test_latlng_to_tile_invalid_latitude() {
        assert!(TileCoords::latlng_to_tile(90.0, 0.0, 10).is_err());
        assert!(TileCoords::latlng_to_tile(-90.0, 0.0, 10).is_err());
    }

    #[test]
    fn test_latlng_to_tile_invalid_longitude() {
        assert!(TileCoords::latlng_to_tile(0.0, 181.0, 10).is_err());
        assert!(TileCoords::latlng_to_tile(0.0, -181.0, 10).is_err());
    }

    #[test]
    fn test_tile_to_latlng_roundtrip() {
        let (lat_orig, lon_orig) = (37.7749, -122.4194);
        let (col, row) = TileCoords::latlng_to_tile(lat_orig, lon_orig, 15).unwrap();
        let (lat_new, lon_new) = TileCoords::tile_to_latlng(col, row, 15).unwrap();

        // Should be within tile bounds
        assert!((lat_new - lat_orig).abs() < 0.01);
        assert!((lon_new - lon_orig).abs() < 0.01);
    }

    #[test]
    fn test_tile_to_quadkey() {
        // Known Bing quadkey example
        let quadkey = TileCoords::tile_to_quadkey(0, 0, 1);
        assert_eq!(quadkey, "0");

        let quadkey = TileCoords::tile_to_quadkey(1, 0, 1);
        assert_eq!(quadkey, "1");

        let quadkey = TileCoords::tile_to_quadkey(0, 1, 1);
        assert_eq!(quadkey, "2");

        let quadkey = TileCoords::tile_to_quadkey(1, 1, 1);
        assert_eq!(quadkey, "3");
    }

    #[test]
    fn test_quadkey_to_tile() {
        let (col, row, zoom) = TileCoords::quadkey_to_tile("0").unwrap();
        assert_eq!((col, row, zoom), (0, 0, 1));

        let (_col, _row, zoom) = TileCoords::quadkey_to_tile("03").unwrap();
        assert_eq!(zoom, 2);
    }

    #[test]
    fn test_quadkey_roundtrip() {
        let quadkey_orig = "0123012301";
        let (col, row, zoom) = TileCoords::quadkey_to_tile(quadkey_orig).unwrap();
        let quadkey_new = TileCoords::tile_to_quadkey(col, row, zoom);
        assert_eq!(quadkey_orig, quadkey_new);
    }

    #[test]
    fn test_latlng_to_quadkey() {
        // Should not error for valid inputs
        let quadkey = TileCoords::latlng_to_quadkey(37.7749, -122.4194, 10).unwrap();
        assert_eq!(quadkey.len(), 10);
    }

    #[test]
    fn test_tile_bounds() {
        let (lat_n, lon_w, lat_s, lon_e) = TileCoords::tile_bounds(0, 0, 1).unwrap();
        assert!(lat_n > lat_s); // North > South
        assert!(lon_w < lon_e); // West < East
    }
}
