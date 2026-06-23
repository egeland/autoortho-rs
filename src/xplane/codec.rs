// SPDX-License-Identifier: Apache-2.0 OR GPL-3.0
// Copyright (c) 2026 the AutoOrtho contributors

use thiserror::Error;

#[derive(Debug, Error)]
pub enum XPlaneError {
    #[error("RREF packet error")]
    PacketError,
    #[error("Invalid dataref")]
    InvalidDataref,
}

/// X-Plane RREF protocol encoder/decoder
pub struct RrefCodec;

impl RrefCodec {
    /// Encode RREF request packet
    /// Returns: b"RREF" + freq(i32) + index(i32) + dataref(400 bytes)
    pub fn encode_request(freq_hz: i32, index: i32, dataref: &str) -> Vec<u8> {
        let mut packet = Vec::with_capacity(413);
        packet.extend_from_slice(b"RREF");
        packet.extend_from_slice(&freq_hz.to_le_bytes());
        packet.extend_from_slice(&index.to_le_bytes());

        // Dataref (null-terminated, 400 bytes max)
        let dataref_bytes = dataref.as_bytes();
        let max_len = 400.min(dataref_bytes.len());
        packet.extend_from_slice(&dataref_bytes[..max_len]);
        packet.resize(packet.len() + (400 - max_len), 0);

        packet
    }

    /// Decode RREF response: b"RREF" + (index(i32) + value(f32))*
    pub fn decode_response(data: &[u8]) -> Result<Vec<(i32, f32)>, XPlaneError> {
        if data.len() < 4 || &data[0..4] != b"RREF" {
            return Err(XPlaneError::PacketError);
        }

        let mut results = Vec::new();
        let mut offset = 4;

        while offset + 8 <= data.len() {
            let index = i32::from_le_bytes([
                data[offset],
                data[offset + 1],
                data[offset + 2],
                data[offset + 3],
            ]);
            let value = f32::from_le_bytes([
                data[offset + 4],
                data[offset + 5],
                data[offset + 6],
                data[offset + 7],
            ]);
            results.push((index, value));
            offset += 8;
        }

        Ok(results)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_rref_encode_request() {
        let packet = RrefCodec::encode_request(2, 0, "sim/flightmodel/position/latitude");
        assert!(packet.starts_with(b"RREF"));
        assert_eq!(packet.len(), 412); // 4 + 4 + 4 + 400
    }

    #[test]
    fn test_rref_encode_short_dataref() {
        let packet = RrefCodec::encode_request(2, 5, "x");
        assert_eq!(&packet[0..4], b"RREF");
        assert_eq!(packet[4], 2);
    }

    #[test]
    fn test_rref_decode_response() {
        let mut response = Vec::new();
        response.extend_from_slice(b"RREF");
        response.extend_from_slice(&0i32.to_le_bytes());
        response.extend_from_slice(&1.5f32.to_le_bytes());

        let results = RrefCodec::decode_response(&response).unwrap();
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].0, 0);
        assert!((results[0].1 - 1.5).abs() < 0.001);
    }

    #[test]
    fn test_rref_decode_invalid() {
        let bad = b"XXXX".to_vec();
        assert!(RrefCodec::decode_response(&bad).is_err());
    }
}
