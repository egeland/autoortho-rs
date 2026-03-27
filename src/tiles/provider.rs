use std::future::Future;
use std::pin::Pin;
use std::sync::Arc;
use thiserror::Error;

/// Metadata about a tile provider's capabilities.
#[derive(Debug, Clone)]
pub struct ProviderInfo {
    /// Short identifier (e.g., "GO2", "BI", "ARC")
    pub id: &'static str,
    /// Display name (e.g., "Google Maps", "Bing Maps")
    pub display_name: &'static str,
    /// Minimum supported zoom level
    pub min_zoom: u32,
    /// Maximum supported zoom level
    pub max_zoom: u32,
    /// Whether this provider requires authentication/cookies
    pub requires_auth: bool,
}

/// All known providers with their metadata.
pub const PROVIDER_INFO: &[ProviderInfo] = &[
    ProviderInfo {
        id: "GO2",
        display_name: "Google Maps",
        min_zoom: 0,
        max_zoom: 21,
        requires_auth: true,
    },
    ProviderInfo {
        id: "BI",
        display_name: "Bing Maps",
        min_zoom: 1,
        max_zoom: 19,
        requires_auth: false,
    },
    ProviderInfo {
        id: "ARC",
        display_name: "ArcGIS",
        min_zoom: 0,
        max_zoom: 19,
        requires_auth: false,
    },
];

/// Get provider info by ID. Returns None for unknown providers.
pub fn provider_info(id: &str) -> Option<&'static ProviderInfo> {
    PROVIDER_INFO.iter().find(|p| p.id.eq_ignore_ascii_case(id))
}

/// Tile provider trait — object-safe via boxed futures.
/// All providers must be Send + Sync for use behind Arc<dyn TileProvider>.
pub trait TileProvider: Send + Sync {
    /// Fetch a single 256x256 tile at the given coordinates.
    /// Returns the tile data (JPEG) or an error.
    fn fetch(
        &self,
        row: u32,
        col: u32,
        zoom: u32,
    ) -> Pin<Box<dyn Future<Output = Result<Vec<u8>, TileProviderError>> + Send + '_>>;

    /// Get the name of this provider for logging
    fn name(&self) -> &str;
}

#[derive(Debug, Clone, Error)]
pub enum TileProviderError {
    #[error("Network error: {0}")]
    NetworkError(String),
    #[error("Invalid tile coordinates")]
    InvalidCoordinates,
    #[error("Tile not found")]
    NotFound,
    #[error("Rate limited")]
    RateLimited,
}

/// Validate that a response looks like image data (JPEG or PNG), not an HTML error page.
fn validate_image_response(data: &[u8]) -> Result<(), TileProviderError> {
    if data.len() < 4 {
        return Err(TileProviderError::NotFound);
    }
    // JPEG starts with FF D8
    if data[0] == 0xFF && data[1] == 0xD8 {
        return Ok(());
    }
    // PNG starts with 89 50 4E 47
    if data[0] == 0x89 && data[1] == 0x50 && data[2] == 0x4E && data[3] == 0x47 {
        return Ok(());
    }
    // If it starts with '<' it's probably an HTML error page
    if data[0] == b'<' {
        return Err(TileProviderError::NetworkError(
            "Received HTML instead of image data (possible rate limit or auth error)".to_string(),
        ));
    }
    // Unknown format but not obviously wrong — allow it
    Ok(())
}

/// Fetch URL and validate that the response is image data.
async fn fetch_image(client: &reqwest::Client, url: &str) -> Result<Vec<u8>, TileProviderError> {
    let response = client
        .get(url)
        .send()
        .await
        .map_err(|e| TileProviderError::NetworkError(e.to_string()))?;

    let status = response.status();
    if !status.is_success() {
        return Err(TileProviderError::NetworkError(format!("HTTP {}", status)));
    }

    let bytes = response
        .bytes()
        .await
        .map(|b| b.to_vec())
        .map_err(|e| TileProviderError::NetworkError(e.to_string()))?;

    validate_image_response(&bytes)?;
    Ok(bytes)
}

/// Google Maps provider (GO2)
pub struct GoogleMapsProvider {
    client: reqwest::Client,
}

impl GoogleMapsProvider {
    pub fn new() -> Self {
        // Google requires a browser-like User-Agent to avoid blocks
        let client = reqwest::Client::builder()
            .user_agent("Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/120.0.0.0 Safari/537.36")
            .build()
            .expect("failed to build HTTP client");
        Self { client }
    }
}

impl Default for GoogleMapsProvider {
    fn default() -> Self {
        Self::new()
    }
}

impl TileProvider for GoogleMapsProvider {
    fn fetch(
        &self,
        row: u32,
        col: u32,
        zoom: u32,
    ) -> Pin<Box<dyn Future<Output = Result<Vec<u8>, TileProviderError>> + Send + '_>> {
        Box::pin(async move {
            let url = format!(
                "https://mt0.google.com/vt/lyrs=s&x={}&y={}&z={}",
                col, row, zoom
            );
            fetch_image(&self.client, &url).await
        })
    }

    fn name(&self) -> &str {
        "Google Maps"
    }
}

/// Bing Maps provider (BI)
pub struct BingMapsProvider {
    client: reqwest::Client,
    _key: Option<String>,
}

impl BingMapsProvider {
    pub fn new(key: Option<String>) -> Self {
        Self {
            client: reqwest::Client::new(),
            _key: key,
        }
    }
}

impl TileProvider for BingMapsProvider {
    fn fetch(
        &self,
        row: u32,
        col: u32,
        zoom: u32,
    ) -> Pin<Box<dyn Future<Output = Result<Vec<u8>, TileProviderError>> + Send + '_>> {
        Box::pin(async move {
            use crate::tiles::coords::TileCoords;

            let quadkey = TileCoords::tile_to_quadkey(col, row, zoom);
            let url = format!(
                "http://ecn.t3.tiles.virtualearth.net/tiles/a{}.jpeg?g=1",
                quadkey
            );
            fetch_image(&self.client, &url).await
        })
    }

    fn name(&self) -> &str {
        "Bing Maps"
    }
}

/// ArcGIS provider (ARC)
pub struct ArcGisProvider {
    client: reqwest::Client,
}

impl ArcGisProvider {
    pub fn new() -> Self {
        Self {
            client: reqwest::Client::new(),
        }
    }
}

impl Default for ArcGisProvider {
    fn default() -> Self {
        Self::new()
    }
}

/// Provider factory
pub struct ProviderFactory;

impl ProviderFactory {
    /// Create a provider by name
    pub fn create(name: &str) -> Option<Arc<dyn TileProvider>> {
        match name.to_uppercase().as_str() {
            "GO2" | "GOOGLE" => Some(Arc::new(GoogleMapsProvider::new())),
            "BI" | "BING" => Some(Arc::new(BingMapsProvider::new(None))),
            "ARC" | "ARCGIS" => Some(Arc::new(ArcGisProvider::new())),
            _ => None,
        }
    }

    /// List available provider names
    pub fn available_providers() -> Vec<&'static str> {
        vec!["GO2", "BI", "ARC"]
    }
}

impl TileProvider for ArcGisProvider {
    fn fetch(
        &self,
        row: u32,
        col: u32,
        zoom: u32,
    ) -> Pin<Box<dyn Future<Output = Result<Vec<u8>, TileProviderError>> + Send + '_>> {
        Box::pin(async move {
            let url = format!(
                "https://server.arcgisonline.com/ArcGIS/rest/services/World_Imagery/MapServer/tile/{}/{}/{}",
                zoom, row, col
            );
            fetch_image(&self.client, &url).await
        })
    }

    fn name(&self) -> &str {
        "ArcGIS"
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_google_provider_name() {
        let provider = GoogleMapsProvider::new();
        assert_eq!(provider.name(), "Google Maps");
    }

    #[test]
    fn test_bing_provider_name() {
        let provider = BingMapsProvider::new(None);
        assert_eq!(provider.name(), "Bing Maps");
    }

    #[test]
    fn test_bing_provider_with_key() {
        let provider = BingMapsProvider::new(Some("test_key".to_string()));
        assert_eq!(provider.name(), "Bing Maps");
    }

    #[test]
    fn test_provider_error_display() {
        let err = TileProviderError::NetworkError("Connection refused".to_string());
        assert!(err.to_string().contains("Network error"));

        let err = TileProviderError::InvalidCoordinates;
        assert!(err.to_string().contains("Invalid"));

        let err = TileProviderError::NotFound;
        assert!(err.to_string().contains("not found"));
    }

    #[test]
    fn test_arcgis_provider_creation() {
        let provider = ArcGisProvider::new();
        assert_eq!(provider.name(), "ArcGIS");
    }

    #[test]
    fn test_arcgis_provider_default() {
        let provider = ArcGisProvider::default();
        assert_eq!(provider.name(), "ArcGIS");
    }

    #[test]
    fn test_provider_factory_google() {
        let provider = ProviderFactory::create("GO2").unwrap();
        assert_eq!(provider.name(), "Google Maps");
    }

    #[test]
    fn test_provider_factory_bing() {
        let provider = ProviderFactory::create("BI").unwrap();
        assert_eq!(provider.name(), "Bing Maps");
    }

    #[test]
    fn test_provider_factory_arcgis() {
        let provider = ProviderFactory::create("ARC").unwrap();
        assert_eq!(provider.name(), "ArcGIS");
    }

    #[test]
    fn test_provider_factory_invalid() {
        let provider = ProviderFactory::create("INVALID");
        assert!(provider.is_none());
    }

    #[test]
    fn test_provider_factory_available() {
        let providers = ProviderFactory::available_providers();
        assert!(providers.contains(&"GO2"));
        assert!(providers.contains(&"BI"));
        assert!(providers.contains(&"ARC"));
    }
}
