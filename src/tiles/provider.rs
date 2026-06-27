use std::future::Future;
use std::pin::Pin;
use std::sync::Arc;
use std::sync::OnceLock;
use thiserror::Error;

use crate::tiles::coords::TileCoords;

use crate::tiles::apple_token::AppleTokenService;

include!(concat!(env!("OUT_DIR"), "/user_agent.rs"));

/// Shared HTTP client for tile providers.
/// Uses OnceLock for lazy initialization with connection pooling.
static HTTP_CLIENT: OnceLock<reqwest::Client> = OnceLock::new();

/// Shared HTTP client for Google Maps (requires browser User-Agent).
static GOOGLE_CLIENT: OnceLock<reqwest::Client> = OnceLock::new();

/// Get the shared HTTP client for most providers.
/// Uses default settings with connection pooling.
pub fn http_client() -> &'static reqwest::Client {
    HTTP_CLIENT.get_or_init(|| {
        reqwest::Client::builder()
            .tcp_keepalive(std::time::Duration::from_secs(60))
            .build()
            .expect("failed to build HTTP client")
    })
}

/// Get the shared HTTP client for Google Maps.
/// Includes browser User-Agent to avoid blocking.
pub fn google_http_client() -> &'static reqwest::Client {
    GOOGLE_CLIENT.get_or_init(|| {
        reqwest::Client::builder()
            .user_agent(CHROME_USER_AGENT)
            .tcp_keepalive(std::time::Duration::from_secs(60))
            .build()
            .expect("failed to build Google HTTP client")
    })
}

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
    ProviderInfo {
        id: "NAIP",
        display_name: "USGS NAIP",
        min_zoom: 0,
        max_zoom: 19,
        requires_auth: false,
    },
    ProviderInfo {
        id: "USGS",
        display_name: "USGS Topo",
        min_zoom: 0,
        max_zoom: 16,
        requires_auth: false,
    },
    ProviderInfo {
        id: "EOX",
        display_name: "EOX Maps",
        min_zoom: 0,
        max_zoom: 18,
        requires_auth: false,
    },
    ProviderInfo {
        id: "FIREFLY",
        display_name: "Firefly",
        min_zoom: 0,
        max_zoom: 17,
        requires_auth: false,
    },
    ProviderInfo {
        id: "YNDX",
        display_name: "Yandex Maps",
        min_zoom: 0,
        max_zoom: 17,
        requires_auth: false,
    },
    ProviderInfo {
        id: "APPLE",
        display_name: "Apple Maps",
        min_zoom: 0,
        max_zoom: 19,
        requires_auth: true,
    },
];

/// Provider IDs for UI pick lists (mirrors order in PROVIDER_INFO)
pub const PROVIDER_IDS: &[&str] = &[
    "GO2", "BI", "ARC", "NAIP", "USGS", "EOX", "FIREFLY", "YNDX", "APPLE",
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

/// Test if a provider has coverage at the given coordinates.
/// Returns Ok(()) if successful, Err(message) if failed.
/// If successful, the tile data is returned for optional caching.
pub async fn test_provider_coverage(
    provider_id: &str,
    lat: f64,
    lon: f64,
    zoom: u32,
) -> Result<Vec<u8>, String> {
    let provider = ProviderFactory::create(provider_id)
        .ok_or_else(|| format!("Unknown provider: {}", provider_id))?;

    let (row, col) = TileCoords::latlng_to_tile(lat, lon, zoom)
        .map_err(|e| format!("Invalid coordinates: {}", e))?;

    provider
        .fetch(row, col, zoom)
        .await
        .map_err(|e| format!("Coverage test failed: {}", e))
}
pub struct GoogleMapsProvider {
    client: &'static reqwest::Client,
}

impl GoogleMapsProvider {
    pub fn new() -> Self {
        Self {
            client: google_http_client(),
        }
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
            fetch_image(self.client, &url).await
        })
    }

    fn name(&self) -> &str {
        "Google Maps"
    }
}

/// Bing Maps provider (BI)
pub struct BingMapsProvider {
    client: &'static reqwest::Client,
}

impl BingMapsProvider {
    pub fn new() -> Self {
        Self {
            client: http_client(),
        }
    }
}

impl Default for BingMapsProvider {
    fn default() -> Self {
        Self::new()
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
            let quadkey = TileCoords::tile_to_quadkey(col, row, zoom);
            let url = format!(
                "https://ecn.t3.tiles.virtualearth.net/tiles/a{}.jpeg?g=1",
                quadkey
            );
            fetch_image(self.client, &url).await
        })
    }

    fn name(&self) -> &str {
        "Bing Maps"
    }
}

/// ArcGIS provider (ARC)
pub struct ArcGisProvider {
    client: &'static reqwest::Client,
}

impl ArcGisProvider {
    pub fn new() -> Self {
        Self {
            client: http_client(),
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
            "BI" | "BING" => Some(Arc::new(BingMapsProvider::new())),
            "ARC" | "ARCGIS" => Some(Arc::new(ArcGisProvider::new())),
            "NAIP" => Some(Arc::new(UsgsNaipProvider::new())),
            "USGS" => Some(Arc::new(UsgsTopoProvider::new())),
            "EOX" => Some(Arc::new(EoxProvider::new())),
            "FIREFLY" => Some(Arc::new(FireflyProvider::new())),
            "YNDX" | "YANDEX" => Some(Arc::new(YandexMapsProvider::new())),
            "APPLE" => Some(Arc::new(AppleMapsProvider::new())),
            _ => None,
        }
    }

    /// List available provider names
    pub fn available_providers() -> Vec<&'static str> {
        vec![
            "GO2", "BI", "ARC", "NAIP", "USGS", "EOX", "FIREFLY", "YNDX", "APPLE",
        ]
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
            fetch_image(self.client, &url).await
        })
    }

    fn name(&self) -> &str {
        "ArcGIS"
    }
}

/// USGS NAIP provider (NAIP)
pub struct UsgsNaipProvider {
    client: &'static reqwest::Client,
}

impl UsgsNaipProvider {
    pub fn new() -> Self {
        Self {
            client: http_client(),
        }
    }
}

impl Default for UsgsNaipProvider {
    fn default() -> Self {
        Self::new()
    }
}

impl TileProvider for UsgsNaipProvider {
    fn fetch(
        &self,
        row: u32,
        col: u32,
        zoom: u32,
    ) -> Pin<Box<dyn Future<Output = Result<Vec<u8>, TileProviderError>> + Send + '_>> {
        Box::pin(async move {
            let url = format!(
                "https://naip.maptiles.arcgis.com/arcgis/rest/services/NAIP/MapServer/tile/{}/{}/{}",
                zoom, row, col
            );
            fetch_image(self.client, &url).await
        })
    }

    fn name(&self) -> &str {
        "USGS NAIP"
    }
}

/// USGS Topo provider (USGS)
pub struct UsgsTopoProvider {
    client: &'static reqwest::Client,
}

impl UsgsTopoProvider {
    pub fn new() -> Self {
        Self {
            client: http_client(),
        }
    }
}

impl Default for UsgsTopoProvider {
    fn default() -> Self {
        Self::new()
    }
}

impl TileProvider for UsgsTopoProvider {
    fn fetch(
        &self,
        row: u32,
        col: u32,
        zoom: u32,
    ) -> Pin<Box<dyn Future<Output = Result<Vec<u8>, TileProviderError>> + Send + '_>> {
        Box::pin(async move {
            let url = format!(
                "https://basemap.nationalmap.gov/arcgis/rest/services/USGSImageryOnly/MapServer/tile/{}/{}/{}",
                zoom, row, col
            );
            fetch_image(self.client, &url).await
        })
    }

    fn name(&self) -> &str {
        "USGS Topo"
    }
}

/// EOX provider (EOX)
pub struct EoxProvider {
    client: &'static reqwest::Client,
}

impl EoxProvider {
    pub fn new() -> Self {
        Self {
            client: http_client(),
        }
    }
}

impl Default for EoxProvider {
    fn default() -> Self {
        Self::new()
    }
}

impl TileProvider for EoxProvider {
    fn fetch(
        &self,
        row: u32,
        col: u32,
        zoom: u32,
    ) -> Pin<Box<dyn Future<Output = Result<Vec<u8>, TileProviderError>> + Send + '_>> {
        Box::pin(async move {
            let url = format!(
                "https://s2maps-tiles.eu/wmts?layer=s2cloudless-2024_3857&style=default&tilematrixset=g&Service=WMTS&Request=GetTile&Version=1.0.0&Format=image%2Fjpeg&TileMatrix={}&TileCol={}&TileRow={}",
                zoom, col, row
            );
            fetch_image(self.client, &url).await
        })
    }

    fn name(&self) -> &str {
        "EOX Maps"
    }
}

/// Firefly provider (FIREFLY)
pub struct FireflyProvider {
    client: &'static reqwest::Client,
}

impl FireflyProvider {
    pub fn new() -> Self {
        Self {
            client: http_client(),
        }
    }
}

impl Default for FireflyProvider {
    fn default() -> Self {
        Self::new()
    }
}

impl TileProvider for FireflyProvider {
    fn fetch(
        &self,
        row: u32,
        col: u32,
        zoom: u32,
    ) -> Pin<Box<dyn Future<Output = Result<Vec<u8>, TileProviderError>> + Send + '_>> {
        Box::pin(async move {
            let url = format!(
                "https://fly.maptiles.arcgis.com/arcgis/rest/services/World_Imagery_Firefly/MapServer/tile/{}/{}/{}",
                zoom, row, col
            );
            fetch_image(self.client, &url).await
        })
    }

    fn name(&self) -> &str {
        "Firefly"
    }
}

/// Yandex Maps provider (YNDX)
/// Uses round-robin server selection like Google Maps.
pub struct YandexMapsProvider {
    client: &'static reqwest::Client,
    server: std::sync::atomic::AtomicU32,
}

impl YandexMapsProvider {
    pub fn new() -> Self {
        Self {
            client: http_client(),
            server: std::sync::atomic::AtomicU32::new(1),
        }
    }

    fn next_server(&self) -> u32 {
        // Cycle through servers 1-4
        self.server
            .fetch_add(1, std::sync::atomic::Ordering::Relaxed)
            % 4
            + 1
    }
}

impl Default for YandexMapsProvider {
    fn default() -> Self {
        Self::new()
    }
}

impl TileProvider for YandexMapsProvider {
    fn fetch(
        &self,
        row: u32,
        col: u32,
        zoom: u32,
    ) -> Pin<Box<dyn Future<Output = Result<Vec<u8>, TileProviderError>> + Send + '_>> {
        let server = self.next_server();
        Box::pin(async move {
            let url = format!(
                "https://sat{:02}.maps.yandex.net/tiles?l=sat&v=3.1814.0&x={}&y={}&z={}",
                server, col, row, zoom
            );
            fetch_image(self.client, &url).await
        })
    }

    fn name(&self) -> &str {
        "Yandex Maps"
    }
}

/// Apple Maps provider (APPLE)
/// Requires dynamic authentication tokens that must be refreshed on 403/410 errors.
pub struct AppleMapsProvider {
    client: reqwest::Client,
    token_service: AppleTokenService,
}

impl AppleMapsProvider {
    pub fn new() -> Self {
        Self {
            client: reqwest::Client::builder()
                .user_agent("Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7)")
                .build()
                .expect("failed to build Apple HTTP client"),
            token_service: AppleTokenService::new(),
        }
    }
}

impl Default for AppleMapsProvider {
    fn default() -> Self {
        Self::new()
    }
}

impl TileProvider for AppleMapsProvider {
    fn fetch(
        &self,
        row: u32,
        col: u32,
        zoom: u32,
    ) -> Pin<Box<dyn Future<Output = Result<Vec<u8>, TileProviderError>> + Send + '_>> {
        Box::pin(async move {
            // Fetch token (with retry on auth failures) - max 3 attempts
            let mut token_attempts = 0;
            let token = loop {
                token_attempts += 1;
                match self.token_service.get_token().await {
                    Ok(t) => break t,
                    Err(e) => {
                        if token_attempts >= 3 {
                            return Err(TileProviderError::NetworkError(format!(
                                "Failed to get Apple token after {} attempts: {}",
                                token_attempts, e
                            )));
                        }
                        // Token fetch failed, try again after reset
                        self.token_service.reset_token();
                        // Small delay to avoid hammering
                        tokio::time::sleep(std::time::Duration::from_millis(500)).await;
                        continue;
                    }
                }
            };

            let url = self.token_service.make_tile_url(col, row, zoom, &token);

            // Make request with retry on auth failures
            let mut attempts = 0;
            loop {
                attempts += 1;
                let response = self
                    .client
                    .get(&url)
                    .send()
                    .await
                    .map_err(|e| TileProviderError::NetworkError(e.to_string()))?;

                let status = response.status();
                if status == 200 {
                    let bytes = response
                        .bytes()
                        .await
                        .map(|b| b.to_vec())
                        .map_err(|e| TileProviderError::NetworkError(e.to_string()))?;
                    return validate_image_response(&bytes).map(|_| bytes);
                } else if status == 403 || status == 410 {
                    // Token expired, reset and retry (once)
                    if attempts < 2 {
                        self.token_service.reset_token();
                        continue;
                    }
                    return Err(TileProviderError::RateLimited);
                } else {
                    return Err(TileProviderError::NetworkError(format!(
                        "HTTP {} after {} attempts",
                        status, attempts
                    )));
                }
            }
        })
    }

    fn name(&self) -> &str {
        "Apple Maps"
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
        let provider = BingMapsProvider::new();
        assert_eq!(provider.name(), "Bing Maps");
    }

    #[test]
    fn test_bing_provider_creation() {
        let provider = BingMapsProvider::new();
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

    // --- validate_image_response tests ---

    #[test]
    fn test_validate_jpeg() {
        // JPEG magic: FF D8 FF E0
        let jpeg = [0xFF, 0xD8, 0xFF, 0xE0, 0x00, 0x10, 0x4A, 0x46, 0x49, 0x46];
        assert!(validate_image_response(&jpeg).is_ok());
    }

    #[test]
    fn test_validate_png() {
        // PNG magic: 89 50 4E 47
        let png = [0x89, 0x50, 0x4E, 0x47, 0x0D, 0x0A, 0x1A, 0x0A];
        assert!(validate_image_response(&png).is_ok());
    }

    #[test]
    fn test_validate_html_error() {
        let html = b"<html><body>Error</body></html>";
        let err = validate_image_response(html).unwrap_err();
        match err {
            TileProviderError::NetworkError(msg) => {
                assert!(msg.contains("HTML instead of image"));
            }
            _ => panic!("Expected NetworkError for HTML response"),
        }
    }

    #[test]
    fn test_validate_too_short() {
        let short = [0xFF];
        assert!(validate_image_response(&short).is_err());
    }

    #[test]
    fn test_validate_empty() {
        let empty: [u8; 0] = [];
        assert!(validate_image_response(&empty).is_err());
    }

    #[test]
    fn test_validate_unknown_format_allowed() {
        // Not JPEG, PNG, or HTML — allowed
        let data = [0x00, 0x00, 0x00, 0x00];
        assert!(validate_image_response(&data).is_ok());
    }

    // --- provider_info tests ---

    #[test]
    fn test_provider_info_found() {
        let info = provider_info("GO2").unwrap();
        assert_eq!(info.id, "GO2");
        assert_eq!(info.display_name, "Google Maps");
        assert!(info.requires_auth);
    }

    #[test]
    fn test_provider_info_case_insensitive() {
        let info = provider_info("go2").unwrap();
        assert_eq!(info.id, "GO2");
    }

    #[test]
    fn test_provider_info_not_found() {
        assert!(provider_info("UNKNOWN").is_none());
    }

    #[test]
    fn test_provider_info_all_providers() {
        for id in PROVIDER_IDS {
            let info = provider_info(id).unwrap();
            assert_eq!(info.id, *id);
        }
    }

    // --- ProviderFactory::create — all providers ---

    #[test]
    fn test_factory_create_naip() {
        let p = ProviderFactory::create("NAIP").unwrap();
        assert_eq!(p.name(), "USGS NAIP");
    }

    #[test]
    fn test_factory_create_usgs() {
        let p = ProviderFactory::create("USGS").unwrap();
        assert_eq!(p.name(), "USGS Topo");
    }

    #[test]
    fn test_factory_create_eox() {
        let p = ProviderFactory::create("EOX").unwrap();
        assert_eq!(p.name(), "EOX Maps");
    }

    #[test]
    fn test_factory_create_firefly() {
        let p = ProviderFactory::create("FIREFLY").unwrap();
        assert_eq!(p.name(), "Firefly");
    }

    #[test]
    fn test_factory_create_yndx() {
        let p = ProviderFactory::create("YNDX").unwrap();
        assert_eq!(p.name(), "Yandex Maps");
    }

    #[test]
    fn test_factory_create_apple() {
        let p = ProviderFactory::create("APPLE").unwrap();
        assert_eq!(p.name(), "Apple Maps");
    }

    #[test]
    fn test_factory_create_aliases() {
        // Alternate names
        assert!(ProviderFactory::create("GOOGLE").is_some());
        assert!(ProviderFactory::create("BING").is_some());
        assert!(ProviderFactory::create("ARCGIS").is_some());
        assert!(ProviderFactory::create("YANDEX").is_some());
    }

    #[test]
    fn test_factory_available_providers_full() {
        let providers = ProviderFactory::available_providers();
        assert_eq!(providers.len(), 9);
        let expected = [
            "GO2", "BI", "ARC", "NAIP", "USGS", "EOX", "FIREFLY", "YNDX", "APPLE",
        ];
        for id in &expected {
            assert!(providers.contains(id), "missing provider {}", id);
        }
    }

    // --- Provider name tests for uncovered providers ---

    #[test]
    fn test_naip_provider_name() {
        assert_eq!(UsgsNaipProvider::new().name(), "USGS NAIP");
    }

    #[test]
    fn test_usgs_topo_provider_name() {
        assert_eq!(UsgsTopoProvider::new().name(), "USGS Topo");
    }

    #[test]
    fn test_eox_provider_name() {
        assert_eq!(EoxProvider::new().name(), "EOX Maps");
    }

    #[test]
    fn test_firefly_provider_name() {
        assert_eq!(FireflyProvider::new().name(), "Firefly");
    }

    #[test]
    fn test_yandex_provider_name() {
        assert_eq!(YandexMapsProvider::new().name(), "Yandex Maps");
    }

    #[test]
    fn test_yandex_next_server() {
        let provider = YandexMapsProvider::new();
        // Should cycle 1-4
        let s1 = provider.next_server();
        assert!((1..=4).contains(&s1));
        let s2 = provider.next_server();
        assert!((1..=4).contains(&s2));
    }

    #[test]
    fn test_apple_provider_name() {
        assert_eq!(AppleMapsProvider::new().name(), "Apple Maps");
    }

    #[test]
    fn test_tile_provider_error_rate_limited() {
        let err = TileProviderError::RateLimited;
        assert!(err.to_string().contains("Rate limited"));
    }

    // --- Default impl coverage ---

    #[test]
    fn test_google_provider_default() {
        let p = GoogleMapsProvider::default();
        assert_eq!(p.name(), "Google Maps");
    }

    #[test]
    fn test_bing_provider_default() {
        let p = BingMapsProvider::default();
        assert_eq!(p.name(), "Bing Maps");
    }

    #[test]
    fn test_naip_provider_default() {
        let p = UsgsNaipProvider::default();
        assert_eq!(p.name(), "USGS NAIP");
    }

    #[test]
    fn test_usgs_topo_provider_default() {
        let p = UsgsTopoProvider::default();
        assert_eq!(p.name(), "USGS Topo");
    }

    #[test]
    fn test_eox_provider_default() {
        let p = EoxProvider::default();
        assert_eq!(p.name(), "EOX Maps");
    }

    #[test]
    fn test_firefly_provider_default() {
        let p = FireflyProvider::default();
        assert_eq!(p.name(), "Firefly");
    }

    #[test]
    fn test_yandex_provider_default() {
        let p = YandexMapsProvider::default();
        assert_eq!(p.name(), "Yandex Maps");
    }

    #[test]
    fn test_apple_provider_default() {
        let p = AppleMapsProvider::default();
        assert_eq!(p.name(), "Apple Maps");
    }

    // --- fetch_image / test_provider_coverage ---

    #[tokio::test]
    async fn test_provider_coverage_unknown_provider() {
        let result = test_provider_coverage("NONEXISTENT", 0.0, 0.0, 10).await;
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("Unknown provider"));
    }

    #[tokio::test]
    async fn test_provider_coverage_invalid_coords() {
        // Use lat/lng that tile_to_tile rejects
        let result = test_provider_coverage("GO2", f64::NAN, 0.0, 10).await;
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(err.contains("Invalid coordinates") || err.contains("Coverage test failed"));
    }

    // fetch() methods are network-dependent, skip per plan

    // --- PROVIDER_INFO properties ---

    #[test]
    fn test_provider_info_zoom_ranges() {
        let info = provider_info("GO2").unwrap();
        assert_eq!(info.min_zoom, 0);
        assert_eq!(info.max_zoom, 21);
        assert!(info.requires_auth);

        let info = provider_info("BI").unwrap();
        assert_eq!(info.min_zoom, 1);
        assert_eq!(info.max_zoom, 19);
        assert!(!info.requires_auth);

        let info = provider_info("APPLE").unwrap();
        assert!(info.requires_auth);
    }

    #[test]
    fn test_provider_info_all_have_names() {
        for info in PROVIDER_INFO {
            assert!(
                !info.display_name.is_empty(),
                "{} has empty display_name",
                info.id
            );
            assert!(!info.id.is_empty(), "provider has empty id");
        }
    }
}
