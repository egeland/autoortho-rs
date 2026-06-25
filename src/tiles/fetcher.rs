use crate::tiles::chunk::{Chunk, ChunkError, ChunkState};
use crate::tiles::provider::{ProviderFactory, TileProvider};
use crate::tiles::rate_limiter::RateLimiter;
use lru::LruCache;
use std::num::NonZero;
use std::sync::Arc;
use std::sync::atomic::{AtomicU64, Ordering};
use tokio::sync::RwLock;
use tracing::debug;

/// Default cache size (number of chunks).
const DEFAULT_CACHE_SIZE: usize = 1024;

/// Builder for creating `TileFetcher` instances with custom configuration.
pub struct TileFetcherBuilder {
    provider: Arc<dyn TileProvider>,
    provider_id: String,
    cache_size: usize,
    rate_limit: Option<f64>,
}

impl TileFetcherBuilder {
    /// Create a new builder with the given provider and ID.
    pub fn new(provider: Arc<dyn TileProvider>, provider_id: &str) -> Self {
        Self {
            provider,
            provider_id: provider_id.to_string(),
            cache_size: DEFAULT_CACHE_SIZE,
            rate_limit: None,
        }
    }

    /// Set the cache size (number of chunks). Defaults to 1024.
    pub fn cache_size(mut self, size: usize) -> Self {
        self.cache_size = size;
        self
    }

    /// Set the rate limit (requests per second). Defaults to system default.
    pub fn rate_limit(mut self, rate: f64) -> Self {
        self.rate_limit = Some(rate);
        self
    }

    /// Build the `TileFetcher`.
    pub fn build(self) -> TileFetcher {
        let rate_limiter = match self.rate_limit {
            Some(rate) => RateLimiter::new(rate),
            None => RateLimiter::default_rate(),
        };

        TileFetcher {
            chunks: Arc::new(RwLock::new(LruCache::new(
                NonZero::new(self.cache_size.max(1)).unwrap(),
            ))),
            default_provider: self.provider,
            default_provider_id: self.provider_id,
            cache_hits: AtomicU64::new(0),
            cache_misses: AtomicU64::new(0),
            rate_limiter,
        }
    }
}

/// Manages concurrent tile fetching with per-chunk state tracking and bounded LRU cache.
pub struct TileFetcher {
    chunks: Arc<RwLock<LruCache<String, Chunk>>>,
    default_provider: Arc<dyn TileProvider>,
    default_provider_id: String,
    cache_hits: AtomicU64,
    cache_misses: AtomicU64,
    rate_limiter: RateLimiter,
}

impl TileFetcher {
    /// Create a new TileFetcher with default settings.
    pub fn new(provider: Arc<dyn TileProvider>, default_provider_id: &str) -> Self {
        TileFetcherBuilder::new(provider, default_provider_id).build()
    }

    /// Create a builder for custom configuration.
    pub fn builder(provider: Arc<dyn TileProvider>, provider_id: &str) -> TileFetcherBuilder {
        TileFetcherBuilder::new(provider, provider_id)
    }

    /// Create a new TileFetcher with a specific cache size and default provider ID.
    /// This constructor is mainly for testing - it creates a default provider internally.
    pub fn with_cache_size(cache_entries: usize, default_provider_id: &str) -> Self {
        let provider = ProviderFactory::create(default_provider_id)
            .expect("ProviderFactory::create returned None - provider not found");
        Self {
            chunks: Arc::new(RwLock::new(LruCache::new(
                NonZero::new(cache_entries.max(1)).unwrap(),
            ))),
            default_provider: provider,
            default_provider_id: default_provider_id.to_string(),
            cache_hits: AtomicU64::new(0),
            cache_misses: AtomicU64::new(0),
            rate_limiter: RateLimiter::default_rate(),
        }
    }

    #[cfg(test)]
    pub fn with_provider_and_cache_size(
        provider: Arc<dyn TileProvider>,
        cache_entries: usize,
        default_provider_id: &str,
    ) -> Self {
        Self {
            chunks: Arc::new(RwLock::new(LruCache::new(
                NonZero::new(cache_entries.max(1)).unwrap(),
            ))),
            default_provider: provider,
            default_provider_id: default_provider_id.to_string(),
            cache_hits: AtomicU64::new(0),
            cache_misses: AtomicU64::new(0),
            rate_limiter: RateLimiter::default_rate(),
        }
    }

    pub fn with_rate_limit(
        provider: Arc<dyn TileProvider>,
        default_provider_id: &str,
        rate_per_second: f64,
    ) -> Self {
        TileFetcherBuilder::new(provider, default_provider_id)
            .rate_limit(rate_per_second)
            .build()
    }

    /// Get the default provider ID for this fetcher.
    pub fn default_provider_id(&self) -> &str {
        &self.default_provider_id
    }

    /// Get or create a chunk, returning its current data if cached.
    /// Uses the default provider from the TileFetcher.
    pub async fn get_chunk_data(
        &self,
        row: u32,
        col: u32,
        maptype: &str,
        zoom: u32,
    ) -> Result<Option<Arc<Vec<u8>>>, ChunkError> {
        self.get_chunk_data_with_provider(row, col, maptype, zoom, &self.default_provider_id)
            .await
    }

    /// Clear all cached chunks
    pub async fn clear_cache(&self) {
        self.chunks.write().await.clear();
    }

    /// Get current cache size
    pub async fn cache_size(&self) -> usize {
        self.chunks.read().await.len()
    }

    /// Get chunk data with a specific provider.
    /// The provider_id is included in the cache key so different providers
    /// for the same tile coordinates are cached separately.
    pub async fn get_chunk_data_with_provider(
        &self,
        row: u32,
        col: u32,
        maptype: &str,
        zoom: u32,
        provider_id: &str,
    ) -> Result<Option<Arc<Vec<u8>>>, ChunkError> {
        let key = format!("{}_{}_{}_{}_{}", row, col, maptype, zoom, provider_id);

        // Fast path: check if already cached (write lock needed for LRU ordering)
        {
            let mut chunks = self.chunks.write().await;
            if let Some(chunk) = chunks.get_mut(&key)
                && let Some(data) = chunk.data_arc()
            {
                self.cache_hits.fetch_add(1, Ordering::Relaxed);
                return Ok(Some(Arc::clone(&data)));
            }
        }
        // Write lock released here
        self.cache_misses.fetch_add(1, Ordering::Relaxed);

        // Check if we need to fetch (write lock to mark as fetching)
        let needs_fetch = {
            let mut chunks = self.chunks.write().await;
            // Check if chunk exists and is fetchable
            if let Some(chunk) = chunks.get_mut(&key) {
                if chunk.state() == ChunkState::Fetching {
                    // Another task is already fetching this chunk
                    false
                } else if chunk.data().is_some() {
                    // Already cached
                    true
                } else {
                    // Missing or error state - try to fetch
                    if chunk.set_fetching().is_err() {
                        debug!("Chunk {} already being fetched", key);
                    }
                    true
                }
            } else {
                // Chunk doesn't exist - create and fetch
                chunks.push(key.clone(), Chunk::new(row, col, maptype.to_string(), zoom));
                if let Some(chunk) = chunks.get_mut(&key)
                    && chunk.set_fetching().is_err()
                {
                    debug!("Chunk {} already being fetched", key);
                }
                true
            }
        };
        // Write lock released before the async network call

        if needs_fetch {
            // Use stored provider if it matches, otherwise try to create from factory
            let provider: Arc<dyn TileProvider> = if provider_id == self.default_provider_id {
                Arc::clone(&self.default_provider)
            } else {
                match ProviderFactory::create(provider_id) {
                    Some(p) => p,
                    None => {
                        let mut chunks = self.chunks.write().await;
                        if let Some(chunk) = chunks.get_mut(&key) {
                            chunk.set_error()?;
                        }
                        return Err(ChunkError::DownloadFailed(format!(
                            "Unknown provider: {}",
                            provider_id
                        )));
                    }
                }
            };

            // Acquire rate limiter token before fetching
            self.rate_limiter.acquire().await;

            // Perform the actual fetch without holding any lock
            let result = provider.fetch(row, col, zoom).await;

            // Re-acquire lock to store the result
            let mut chunks = self.chunks.write().await;
            if let Some(chunk) = chunks.get_mut(&key) {
                match result {
                    Ok(data) => {
                        chunk.set_cached(data)?;
                    }
                    Err(e) => {
                        chunk.set_error()?;
                        return Err(ChunkError::DownloadFailed(e.to_string()));
                    }
                }
            }
        }

        // Return the cached data
        let mut chunks = self.chunks.write().await;
        Ok(chunks.get_mut(&key).and_then(|c| c.data_arc()))
    }

    /// Try to get chunk data at optimal zoom level (upserving).
    /// Tries from max_zoom down to min_zoom, returns first found in cache.
    /// Returns (data, actual_zoom_level) if found, None if not cached at any zoom.
    pub async fn get_chunk_data_with_upserving(
        &self,
        row: u32,
        col: u32,
        maptype: &str,
        min_zoom: u32,
        max_zoom: u32,
        provider_id: &str,
    ) -> Option<(Arc<Vec<u8>>, u32)> {
        let mut chunks = self.chunks.write().await;

        for zoom in (min_zoom..=max_zoom).rev() {
            let key = format!("{}_{}_{}_{}_{}", row, col, maptype, zoom, provider_id);
            if let Some(chunk) = chunks.get_mut(&key)
                && let Some(data) = chunk.data_arc()
            {
                return Some((Arc::clone(&data), zoom));
            }
        }

        None
    }

    /// Get cache statistics.
    pub async fn cache_stats(&self) -> ChunkCacheStats {
        ChunkCacheStats {
            hits: self.cache_hits.load(Ordering::Relaxed),
            misses: self.cache_misses.load(Ordering::Relaxed),
            entries: self.cache_size().await,
        }
    }
}

/// Chunk cache statistics.
#[derive(Debug, Clone, Default)]
pub struct ChunkCacheStats {
    pub hits: u64,
    pub misses: u64,
    pub entries: usize,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_utils::MockProvider;

    #[tokio::test]
    async fn test_fetcher_creation() {
        let provider = Arc::new(MockProvider);
        let fetcher = TileFetcher::new(provider, "GO2");
        assert_eq!(fetcher.cache_size().await, 0);
    }

    #[tokio::test]
    async fn test_fetcher_fetch_chunk() {
        let provider = Arc::new(MockProvider);
        let fetcher = TileFetcher::new(provider, "GO2");

        let data = fetcher.get_chunk_data(0, 0, "GO2", 10).await.unwrap();

        assert!(data.is_some());
        assert_eq!(fetcher.cache_size().await, 1);
    }

    #[tokio::test]
    async fn test_fetcher_cache_hit() {
        let provider = Arc::new(MockProvider);
        let fetcher = TileFetcher::new(provider, "GO2");

        // First fetch
        let _data1 = fetcher.get_chunk_data(0, 0, "GO2", 10).await.unwrap();

        // Second fetch should hit cache
        let _data2 = fetcher.get_chunk_data(0, 0, "GO2", 10).await.unwrap();

        assert_eq!(fetcher.cache_size().await, 1); // Still only 1 chunk
    }

    #[tokio::test]
    async fn test_fetcher_clear() {
        let provider = Arc::new(MockProvider);
        let fetcher = TileFetcher::new(provider, "GO2");

        fetcher.get_chunk_data(0, 0, "GO2", 10).await.ok();
        fetcher.get_chunk_data(1, 1, "BI", 11).await.ok();

        assert_eq!(fetcher.cache_size().await, 2);

        fetcher.clear_cache().await;
        assert_eq!(fetcher.cache_size().await, 0);
    }

    #[tokio::test]
    async fn test_fetcher_lru_eviction() {
        let provider = Arc::new(MockProvider);
        // Create fetcher with small cache (3 entries)
        let fetcher = TileFetcher::with_provider_and_cache_size(provider, 3, "GO2");

        // Fill cache with 3 entries
        fetcher.get_chunk_data(0, 0, "GO2", 10).await.ok();
        fetcher.get_chunk_data(1, 0, "GO2", 10).await.ok();
        fetcher.get_chunk_data(2, 0, "GO2", 10).await.ok();
        assert_eq!(fetcher.cache_size().await, 3);

        // Add 4th entry - should evict the oldest (0,0)
        fetcher.get_chunk_data(3, 0, "GO2", 10).await.ok();
        assert_eq!(fetcher.cache_size().await, 3); // Still 3

        // Access (0,0) again to make it most recent
        fetcher.get_chunk_data(0, 0, "GO2", 10).await.ok();
        assert_eq!(fetcher.cache_size().await, 3);

        // Add another - should evict (1,0) which is now oldest
        fetcher.get_chunk_data(4, 0, "GO2", 10).await.ok();
        assert_eq!(fetcher.cache_size().await, 3);
    }

    #[tokio::test]
    async fn test_fetcher_cache_stats() {
        let provider = Arc::new(MockProvider);
        let fetcher = TileFetcher::with_provider_and_cache_size(provider, 10, "GO2");

        // First fetch - should be a miss
        let _ = fetcher.get_chunk_data(0, 0, "GO2", 10).await.unwrap();

        // Same fetch - should be a hit
        let _ = fetcher.get_chunk_data(0, 0, "GO2", 10).await.unwrap();

        let stats = fetcher.cache_stats().await;
        assert_eq!(stats.hits, 1);
        assert_eq!(stats.misses, 1);
    }

    #[tokio::test]
    async fn test_builder_default() {
        let provider = Arc::new(MockProvider);
        let fetcher = TileFetcher::builder(provider, "ARC").build();

        let result = fetcher.get_chunk_data(0, 0, "BI", 16).await;
        assert!(result.is_ok());
        assert!(result.unwrap().is_some());
    }

    #[tokio::test]
    async fn test_builder_custom_cache_size() {
        let provider = Arc::new(MockProvider);
        let fetcher = TileFetcher::builder(provider, "ARC")
            .cache_size(100)
            .build();

        assert_eq!(fetcher.cache_size().await, 0);

        let _ = fetcher.get_chunk_data(0, 0, "BI", 16).await;
        assert_eq!(fetcher.cache_size().await, 1);
    }

    #[tokio::test]
    async fn test_builder_custom_rate_limit() {
        let provider = Arc::new(MockProvider);
        let fetcher = TileFetcher::builder(provider, "ARC")
            .rate_limit(100.0)
            .build();

        let result = fetcher.get_chunk_data(0, 0, "BI", 16).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_builder_chaining() {
        let provider = Arc::new(MockProvider);
        let fetcher = TileFetcher::builder(provider, "ARC")
            .cache_size(512)
            .rate_limit(50.0)
            .build();

        let result = fetcher.get_chunk_data(100, 200, "GO2", 14).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_with_rate_limit_uses_builder() {
        let provider = Arc::new(MockProvider);
        let fetcher = TileFetcher::with_rate_limit(provider, "ARC", 20.0);

        let result = fetcher.get_chunk_data(0, 0, "BI", 16).await;
        assert!(result.is_ok());
    }
}
