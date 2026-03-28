use crate::tiles::chunk::{Chunk, ChunkError, ChunkState};
use crate::tiles::provider::{TileProvider, ProviderFactory};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

/// Manages concurrent tile fetching with per-chunk state tracking
pub struct TileFetcher {
    chunks: Arc<RwLock<HashMap<String, Chunk>>>,
    provider: Arc<dyn TileProvider>,
}

impl TileFetcher {
    pub fn new(provider: Arc<dyn TileProvider>) -> Self {
        Self {
            chunks: Arc::new(RwLock::new(HashMap::new())),
            provider,
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

        // Fast path: check if already cached (read lock only)
        {
            let chunks = self.chunks.read().await;
            if let Some(chunk) = chunks.get(&key)
                && let Some(data) = chunk.data()
            {
                return Ok(Some(data.to_vec()));
            }
        }
        // Read lock released here

        // Check if we need to fetch (brief write lock to mark as fetching)
        let needs_fetch = {
            let mut chunks = self.chunks.write().await;
            let chunk = chunks
                .entry(key.clone())
                .or_insert_with(|| Chunk::new(row, col, maptype.to_string(), zoom));

            if chunk.state() == ChunkState::Missing {
                chunk.set_fetching().ok();
                true
            } else {
                false
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
        let chunks = self.chunks.read().await;
        Ok(chunks.get(&key).and_then(|c| c.data().map(|d| d.to_vec())))
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

        // Fast path: check if already cached (read lock only)
        {
            let chunks = self.chunks.read().await;
            if let Some(chunk) = chunks.get(&key)
                && let Some(data) = chunk.data()
            {
                return Ok(Some(data.to_vec()));
            }
        }
        // Read lock released here

        // Check if we need to fetch (brief write lock to mark as fetching)
        let needs_fetch = {
            let mut chunks = self.chunks.write().await;
            let chunk = chunks
                .entry(key.clone())
                .or_insert_with(|| Chunk::new(row, col, maptype.to_string(), zoom));

            if chunk.state() == ChunkState::Missing {
                chunk.set_fetching().ok();
                true
            } else {
                false
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
                    return Err(ChunkError::DownloadFailed(format!("Unknown provider: {}", provider_id)));
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
        let chunks = self.chunks.read().await;
        Ok(chunks.get(&key).and_then(|c| c.data().map(|d| d.to_vec())))
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
        let chunks = self.chunks.read().await;

        for zoom in (min_zoom..=max_zoom).rev() {
            let key = format!("{}_{}_{}_{}_{}", row, col, maptype, zoom, provider_id);
            if let Some(chunk) = chunks.get(&key)
                && let Some(data) = chunk.data() {
                    return Some((data.to_vec(), zoom));
                }
        }

        None
    }
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
}
