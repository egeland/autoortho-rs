// SPDX-License-Identifier: Apache-2.0 OR GPL-3.0
// Copyright (c) 2024 the AutoOrtho contributors

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

        // Extract tile source info for satellite
        let tile_sources = apple_json
            .get("tileSources")
            .and_then(|ts| ts.as_array())
            .ok_or_else(|| AppleTokenError::ParseError("No tileSources found".to_string()))?;

        for ts in tile_sources {
            if ts.get("tileSource")
                .and_then(|s| s.as_str())
                .map(|s| s == "satellite")
                .unwrap_or(false)
            {
                let path = ts
                    .get("path")
                    .and_then(|p| p.as_str())
                    .ok_or_else(|| AppleTokenError::ParseError("No path in tileSource".to_string()))?;

                // Extract version and accessKey from path like:
                // /tile?style=7&size=1&scale=1&z={z}&x={x}&y={y}&v=21.04.15&accessKey=abc123...
                let version = path
                    .split("v=")
                    .nth(1)
                    .and_then(|s| s.split('&').next())
                    .ok_or_else(|| AppleTokenError::ParseError("No version in path".to_string()))?
                    .to_string();

                let access_key = path
                    .split("accessKey=")
                    .nth(1)
                    .ok_or_else(|| AppleTokenError::ParseError("No accessKey in path".to_string()))?
                    .to_string();

                return Ok(AppleToken { version, access_key });
            }
        }

        Err(AppleTokenError::ParseError(
            "Could not find satellite tile source".to_string(),
        ))
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_token_service_creation() {
        let service = AppleTokenService::new();
        assert!(service.token.read().is_none());
    }

    #[test]
    fn test_reset_token() {
        let service = AppleTokenService::new();
        // Manually set a token
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
}
