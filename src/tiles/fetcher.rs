use crate::tiles::chunk::{Chunk, ChunkError, ChunkState};
use crate::tiles::provider::{ProviderFactory, TileProvider};
use lru::LruCache;
use std::num::NonZero;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;
use tokio::sync::RwLock;

/// Manages concurrent tile fetching with per-chunk state tracking and bounded LRU cache.
pub struct TileFetcher {
    chunks: Arc<RwLock<LruCache<String, Chunk>>>,
    provider: Arc<dyn TileProvider>,
    cache_hits: AtomicU64,
    cache_misses: AtomicU64,
}

impl TileFetcher {
    /// Create a new TileFetcher with default cache size (1024 entries).
    pub fn new(provider: Arc<dyn TileProvider>) -> Self {
        Self::with_cache_size(provider, 1024)
    }

    /// Create a new TileFetcher with a specific cache size.
    pub fn with_cache_size(provider: Arc<dyn TileProvider>, cache_entries: usize) -> Self {
        Self {
            chunks: Arc::new(RwLock::new(LruCache::new(
                NonZero::new(cache_entries.max(1)).unwrap(),
            ))),
            provider,
            cache_hits: AtomicU64::new(0),
            cache_misses: AtomicU64::new(0),
        }
    }

    /// Get or create a chunk, returning its current data if cached
    pub async fn get_chunk_data(
        &self,
        row: u32,
        col: u32,
        maptype: &str,
        zoom: u32,
    ) -> Result<Option<Vec<u8>>, ChunkError> {
        let key = format!("{}_{}_{}_{}", row, col, maptype, zoom);

        // Fast path: check if already cached (write lock needed for LRU ordering)
        {
            let mut chunks = self.chunks.write().await;
            if let Some(chunk) = chunks.get_mut(&key)
                && let Some(data) = chunk.data()
            {
                self.cache_hits.fetch_add(1, Ordering::Relaxed);
                return Ok(Some(data.to_vec()));
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
                    // Already cached (shouldn't happen due to first check, but handle anyway)
                    true
                } else {
                    // Missing or error state - try to fetch
                    chunk.set_fetching().ok();
                    true
                }
            } else {
                // Chunk doesn't exist - create and fetch
                chunks.push(key.clone(), Chunk::new(row, col, maptype.to_string(), zoom));
                if let Some(chunk) = chunks.get_mut(&key) {
                    chunk.set_fetching().ok();
                }
                true
            }
        };
        // Write lock released before the async network call

        if needs_fetch {
            // Perform the actual fetch without holding any lock
            let result = self.provider.fetch(row, col, zoom).await;

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
        Ok(chunks.get_mut(&key).and_then(|c| c.data().map(|d| d.to_vec())))
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
    ) -> Result<Option<Vec<u8>>, ChunkError> {
        let key = format!("{}_{}_{}_{}_{}", row, col, maptype, zoom, provider_id);

        // Fast path: check if already cached (write lock needed for LRU ordering)
        {
            let mut chunks = self.chunks.write().await;
            if let Some(chunk) = chunks.get_mut(&key)
                && let Some(data) = chunk.data()
            {
                self.cache_hits.fetch_add(1, Ordering::Relaxed);
                return Ok(Some(data.to_vec()));
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
                    chunk.set_fetching().ok();
                    true
                }
            } else {
                // Chunk doesn't exist - create and fetch
                chunks.push(key.clone(), Chunk::new(row, col, maptype.to_string(), zoom));
                if let Some(chunk) = chunks.get_mut(&key) {
                    chunk.set_fetching().ok();
                }
                true
            }
        };
        // Write lock released before the async network call

        if needs_fetch {
            // Create the provider for this specific request
            let provider = match ProviderFactory::create(provider_id) {
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
            };

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
        Ok(chunks.get_mut(&key).and_then(|c| c.data().map(|d| d.to_vec())))
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
    ) -> Option<(Vec<u8>, u32)> {
        let mut chunks = self.chunks.write().await;

        for zoom in (min_zoom..=max_zoom).rev() {
            let key = format!("{}_{}_{}_{}_{}", row, col, maptype, zoom, provider_id);
            if let Some(chunk) = chunks.get_mut(&key)
                && let Some(data) = chunk.data()
            {
                return Some((data.to_vec(), zoom));
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
    use std::future::Future;
    use std::pin::Pin;

    struct MockProvider;

    impl TileProvider for MockProvider {
        fn fetch(
            &self,
            _row: u32,
            _col: u32,
            _zoom: u32,
        ) -> Pin<
            Box<
                dyn Future<Output = Result<Vec<u8>, crate::tiles::provider::TileProviderError>>
                    + Send
                    + '_,
            >,
        > {
            Box::pin(async { Ok(vec![0xFF, 0xD8]) })
        }

        fn name(&self) -> &str {
            "Mock"
        }
    }

    #[tokio::test]
    async fn test_fetcher_creation() {
        let provider = Arc::new(MockProvider);
        let fetcher = TileFetcher::new(provider);
        assert_eq!(fetcher.cache_size().await, 0);
    }

    #[tokio::test]
    async fn test_fetcher_fetch_chunk() {
        let provider = Arc::new(MockProvider);
        let fetcher = TileFetcher::new(provider);

        let data = fetcher.get_chunk_data(0, 0, "GO2", 10).await.unwrap();

        assert!(data.is_some());
        assert_eq!(fetcher.cache_size().await, 1);
    }

    #[tokio::test]
    async fn test_fetcher_cache_hit() {
        let provider = Arc::new(MockProvider);
        let fetcher = TileFetcher::new(provider);

        // First fetch
        let _data1 = fetcher.get_chunk_data(0, 0, "GO2", 10).await.unwrap();

        // Second fetch should hit cache
        let _data2 = fetcher.get_chunk_data(0, 0, "GO2", 10).await.unwrap();

        assert_eq!(fetcher.cache_size().await, 1); // Still only 1 chunk
    }

    #[tokio::test]
    async fn test_fetcher_clear() {
        let provider = Arc::new(MockProvider);
        let fetcher = TileFetcher::new(provider);

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
        let fetcher = TileFetcher::with_cache_size(provider, 3);

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
        let fetcher = TileFetcher::with_cache_size(provider, 10);

        // First fetch - should be a miss
        let _ = fetcher.get_chunk_data(0, 0, "GO2", 10).await.unwrap();

        // Same fetch - should be a hit
        let _ = fetcher.get_chunk_data(0, 0, "GO2", 10).await.unwrap();

        let stats = fetcher.cache_stats().await;
        assert_eq!(stats.hits, 1);
        assert_eq!(stats.misses, 1);
    }
}
