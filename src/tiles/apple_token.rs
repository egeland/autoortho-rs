// SPDX-License-Identifier: Apache-2.0 OR GPL-3.0
// Copyright (c) 2026 the AutoOrtho contributors

//! Apple Maps token service for authentication.
//!
//! Apple Maps requires dynamic authentication tokens obtained through a flow:
//! 1. Fetch a token from DuckDuckGo (origin authorization)
//! 2. Use that to fetch Apple's bootstrap API
//! 3. Extract satellite tile URL metadata (version + accessKey)

use parking_lot::RwLock;
use std::sync::Arc;
use thiserror::Error;

const DDG_TOKEN_URL: &str = "https://duckduckgo.com/local.js?get_mk_token=1";
const APPLE_BOOTSTRAP_URL: &str =
    "https://cdn.apple-mapkit.com/ma/bootstrap?apiVersion=2&mkjsVersion=5.79.95&poi=1";

#[derive(Debug, Clone)]
pub struct AppleToken {
    pub version: String,
    pub access_key: String,
}

#[derive(Debug, Error)]
pub enum AppleTokenError {
    #[error("Network error: {0}")]
    NetworkError(String),
    #[error("Parse error: {0}")]
    ParseError(String),
    #[error("Token service unavailable")]
    ServiceUnavailable,
}

pub struct AppleTokenService {
    client: reqwest::Client,
    token: Arc<RwLock<Option<AppleToken>>>,
}

impl Default for AppleTokenService {
    fn default() -> Self {
        Self::new()
    }
}

impl AppleTokenService {
    pub fn new() -> Self {
        Self {
            client: reqwest::Client::builder()
                .user_agent("Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7)")
                .timeout(std::time::Duration::from_secs(10))
                .build()
                .expect("failed to build HTTP client"),
            token: Arc::new(RwLock::new(None)),
        }
    }

    pub async fn get_token(&self) -> Result<AppleToken, AppleTokenError> {
        // Check cache first
        if let Some(token) = self.token.read().as_ref() {
            return Ok(token.clone());
        }

        // Fetch new token
        let token = self.fetch_token().await?;
        *self.token.write() = Some(token.clone());
        Ok(token)
    }

    async fn fetch_token(&self) -> Result<AppleToken, AppleTokenError> {
        // Step 1: Get DuckDuckGo token (returns JWT directly now, not mk_token=)
        let ddg_response = self
            .client
            .get(DDG_TOKEN_URL)
            .send()
            .await
            .map_err(|e| AppleTokenError::NetworkError(e.to_string()))?;

        let ddg_text = ddg_response
            .text()
            .await
            .map_err(|e| AppleTokenError::NetworkError(e.to_string()))?;

        // The DDG response is now a JWT directly, use it as Bearer token
        let ddg_token = ddg_text.trim().to_string();

        // Step 2: Get Apple bootstrap with DDG token as authorization
        let apple_response = self
            .client
            .get(APPLE_BOOTSTRAP_URL)
            .header("Origin", "https://duckduckgo.com")
            .header("Authorization", format!("Bearer {}", ddg_token))
            .send()
            .await
            .map_err(|e| AppleTokenError::NetworkError(e.to_string()))?;

        let apple_json: serde_json::Value = apple_response
            .json()
            .await
            .map_err(|e| AppleTokenError::NetworkError(e.to_string()))?;

        parse_bootstrap_response(&apple_json)
    }

    pub fn reset_token(&self) {
        *self.token.write() = None;
    }

    pub fn make_tile_url(&self, col: u32, row: u32, zoom: u32, token: &AppleToken) -> String {
        format!(
            "https://sat-cdn.apple-mapkit.com/tile?style=7&size=1&scale=1&z={}&x={}&y={}&v={}&accessKey={}",
            zoom, col, row, token.version, token.access_key
        )
    }
}

/// Shared global Apple token service instance
static APPLE_TOKEN_SERVICE: std::sync::OnceLock<AppleTokenService> = std::sync::OnceLock::new();

pub fn apple_token_service() -> &'static AppleTokenService {
    APPLE_TOKEN_SERVICE.get_or_init(AppleTokenService::new)
}

/// Parse Apple bootstrap JSON response into an AppleToken.
///
/// Extracts the satellite tile source version and accessKey from the
/// `tileSources` array in the bootstrap response.
pub(crate) fn parse_bootstrap_response(
    json: &serde_json::Value,
) -> Result<AppleToken, AppleTokenError> {
    let tile_sources = json
        .get("tileSources")
        .and_then(|ts| ts.as_array())
        .ok_or_else(|| AppleTokenError::ParseError("No tileSources found".to_string()))?;

    for ts in tile_sources {
        if ts
            .get("tileSource")
            .and_then(|s| s.as_str())
            .map(|s| s == "satellite")
            .unwrap_or(false)
        {
            let path = ts
                .get("path")
                .and_then(|p| p.as_str())
                .ok_or_else(|| AppleTokenError::ParseError("No path in tileSource".to_string()))?;

            let version = path
                .split("v=")
                .nth(1)
                .and_then(|s| s.split('&').next())
                .ok_or_else(|| AppleTokenError::ParseError("No version in path".to_string()))?
                .to_string();

            let access_key = path
                .split("accessKey=")
                .nth(1)
                .and_then(|s| s.split('&').next())
                .ok_or_else(|| AppleTokenError::ParseError("No accessKey in path".to_string()))?
                .to_string();

            return Ok(AppleToken {
                version,
                access_key,
            });
        }
    }

    Err(AppleTokenError::ParseError(
        "Could not find satellite tile source".to_string(),
    ))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_token_service_creation() {
        let service = AppleTokenService::new();
        assert!(service.token.read().is_none());
    }

    #[test]
    fn test_token_service_default() {
        let service = AppleTokenService::default();
        assert!(service.token.read().is_none());
    }

    #[test]
    fn test_reset_token() {
        let service = AppleTokenService::new();
        *service.token.write() = Some(AppleToken {
            version: "test".to_string(),
            access_key: "key".to_string(),
        });
        assert!(service.token.read().is_some());

        service.reset_token();
        assert!(service.token.read().is_none());
    }

    #[test]
    fn test_make_tile_url() {
        let service = AppleTokenService::new();
        let token = AppleToken {
            version: "21.04.15".to_string(),
            access_key: "abc123".to_string(),
        };

        let url = service.make_tile_url(100, 200, 14, &token);
        assert!(url.contains("x=100"));
        assert!(url.contains("y=200"));
        assert!(url.contains("z=14"));
        assert!(url.contains("v=21.04.15"));
        assert!(url.contains("accessKey=abc123"));
    }

    #[test]
    fn test_apple_token_service_singleton() {
        let s1 = apple_token_service();
        let s2 = apple_token_service();
        assert!(std::ptr::eq(s1, s2));
    }

    #[tokio::test]
    async fn test_get_token_cached() {
        let service = AppleTokenService::new();
        let token = AppleToken {
            version: "v1".to_string(),
            access_key: "k1".to_string(),
        };
        *service.token.write() = Some(token.clone());
        let result = service.get_token().await.unwrap();
        assert_eq!(result.version, "v1");
        assert_eq!(result.access_key, "k1");
    }

    // --- parse_bootstrap_response tests ---

    #[test]
    fn test_parse_happy_path() {
        let json = serde_json::json!({
            "tileSources": [{
                "tileSource": "satellite",
                "path": "/tile?style=7&v=21.04.15&accessKey=abc123&z={z}"
            }]
        });
        let token = parse_bootstrap_response(&json).unwrap();
        assert_eq!(token.version, "21.04.15");
        assert_eq!(token.access_key, "abc123");
    }

    #[test]
    fn test_parse_access_key_with_trailing_params() {
        let json = serde_json::json!({
            "tileSources": [{"tileSource": "satellite", "path": "/tile?v=1&accessKey=xyz789&foo=bar"}]
        });
        let token = parse_bootstrap_response(&json).unwrap();
        assert_eq!(token.access_key, "xyz789");
    }

    #[test]
    fn test_parse_no_tile_sources() {
        let json = serde_json::json!({"other": true});
        assert!(matches!(
            parse_bootstrap_response(&json),
            Err(AppleTokenError::ParseError(msg)) if msg == "No tileSources found"
        ));
    }

    #[test]
    fn test_parse_tile_sources_not_array() {
        let json = serde_json::json!({"tileSources": "not_array"});
        assert!(matches!(
            parse_bootstrap_response(&json),
            Err(AppleTokenError::ParseError(_))
        ));
    }

    #[test]
    fn test_parse_no_satellite_source() {
        let json = serde_json::json!({
            "tileSources": [{"tileSource": "road", "path": "/tile?v=1&accessKey=k"}]
        });
        assert!(matches!(
            parse_bootstrap_response(&json),
            Err(AppleTokenError::ParseError(msg)) if msg == "Could not find satellite tile source"
        ));
    }

    #[test]
    fn test_parse_no_path_in_tile_source() {
        let json = serde_json::json!({
            "tileSources": [{"tileSource": "satellite"}]
        });
        assert!(matches!(
            parse_bootstrap_response(&json),
            Err(AppleTokenError::ParseError(msg)) if msg == "No path in tileSource"
        ));
    }

    #[test]
    fn test_parse_no_version_in_path() {
        let json = serde_json::json!({
            "tileSources": [{"tileSource": "satellite", "path": "/tile?accessKey=abc"}]
        });
        assert!(matches!(
            parse_bootstrap_response(&json),
            Err(AppleTokenError::ParseError(msg)) if msg == "No version in path"
        ));
    }

    #[test]
    fn test_parse_no_access_key_in_path() {
        let json = serde_json::json!({
            "tileSources": [{"tileSource": "satellite", "path": "/tile?v=1.0"}]
        });
        assert!(matches!(
            parse_bootstrap_response(&json),
            Err(AppleTokenError::ParseError(msg)) if msg == "No accessKey in path"
        ));
    }

    #[test]
    fn test_parse_version_before_access_key() {
        let json = serde_json::json!({
            "tileSources": [{"tileSource": "satellite", "path": "/tile?v=22.01.01&accessKey=k"}]
        });
        let token = parse_bootstrap_response(&json).unwrap();
        assert_eq!(token.version, "22.01.01");
    }

    #[test]
    fn test_parse_access_key_at_end_of_path() {
        let json = serde_json::json!({
            "tileSources": [{"tileSource": "satellite", "path": "/tile?v=1&accessKey=xyz"}]
        });
        let token = parse_bootstrap_response(&json).unwrap();
        assert_eq!(token.access_key, "xyz");
    }

    #[test]
    fn test_parse_version_with_trailing_params() {
        let json = serde_json::json!({
            "tileSources": [{"tileSource": "satellite", "path": "/tile?v=3.0.1&accessKey=k"}]
        });
        let token = parse_bootstrap_response(&json).unwrap();
        assert_eq!(token.version, "3.0.1");
    }

    #[test]
    fn test_parse_tile_source_without_tile_source_field() {
        let json = serde_json::json!({
            "tileSources": [{}]
        });
        assert!(matches!(
            parse_bootstrap_response(&json),
            Err(AppleTokenError::ParseError(msg)) if msg == "Could not find satellite tile source"
        ));
    }

    #[test]
    fn test_parse_empty_tile_sources() {
        let json = serde_json::json!({"tileSources": []});
        assert!(matches!(
            parse_bootstrap_response(&json),
            Err(AppleTokenError::ParseError(msg)) if msg == "Could not find satellite tile source"
        ));
    }

    #[test]
    fn test_parse_multiple_sources_picks_satellite() {
        let json = serde_json::json!({
            "tileSources": [
                {"tileSource": "road", "path": "/tile?v=1&accessKey=skip"},
                {"tileSource": "satellite", "path": "/tile?v=2&accessKey=hit"}
            ]
        });
        let token = parse_bootstrap_response(&json).unwrap();
        assert_eq!(token.version, "2");
        assert_eq!(token.access_key, "hit");
    }
}
