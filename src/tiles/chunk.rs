use crate::tiles::provider::TileProvider;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum ChunkError {
    #[error("Download failed: {0}")]
    DownloadFailed(String),
    #[error("Cache error: {0}")]
    CacheError(String),
    #[error("Invalid chunk state")]
    InvalidState,
}

/// State of a 256x256 tile chunk
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ChunkState {
    Missing,
    Fetching,
    Cached,
    Error,
}

/// Single 256x256 image chunk (part of a larger tile)
pub struct Chunk {
    pub row: u32,
    pub col: u32,
    pub maptype: String,
    pub zoom: u32,
    state: ChunkState,
    data: Option<Vec<u8>>, // JPEG data
}

impl Chunk {
    pub fn new(row: u32, col: u32, maptype: String, zoom: u32) -> Self {
        Self {
            row,
            col,
            maptype,
            zoom,
            state: ChunkState::Missing,
            data: None,
        }
    }

    pub fn state(&self) -> ChunkState {
        self.state
    }

    pub fn set_fetching(&mut self) -> Result<(), ChunkError> {
        if self.state == ChunkState::Missing {
            self.state = ChunkState::Fetching;
            Ok(())
        } else {
            Err(ChunkError::InvalidState)
        }
    }

    pub fn set_cached(&mut self, data: Vec<u8>) -> Result<(), ChunkError> {
        if self.state == ChunkState::Fetching {
            self.data = Some(data);
            self.state = ChunkState::Cached;
            Ok(())
        } else {
            Err(ChunkError::InvalidState)
        }
    }

    pub fn set_error(&mut self) -> Result<(), ChunkError> {
        if self.state == ChunkState::Fetching {
            self.state = ChunkState::Error;
            Ok(())
        } else {
            Err(ChunkError::InvalidState)
        }
    }

    pub fn data(&self) -> Option<&[u8]> {
        self.data.as_deref()
    }

    pub fn cache_key(&self) -> String {
        format!("{}_{}_{}_z{}", self.row, self.col, self.maptype, self.zoom)
    }

    /// Async fetch from provider
    pub async fn fetch(&mut self, provider: &dyn TileProvider) -> Result<(), ChunkError> {
        self.set_fetching()?;

        match provider.fetch(self.row, self.col, self.zoom).await {
            Ok(data) => self.set_cached(data),
            Err(e) => {
                self.set_error()?;
                Err(ChunkError::DownloadFailed(e.to_string()))
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_chunk_creation() {
        let chunk = Chunk::new(0, 0, "GO2".to_string(), 12);
        assert_eq!(chunk.state(), ChunkState::Missing);
        assert_eq!(chunk.data(), None);
    }

    #[test]
    fn test_chunk_state_transitions() {
        let mut chunk = Chunk::new(0, 0, "GO2".to_string(), 12);

        // Missing -> Fetching
        assert!(chunk.set_fetching().is_ok());
        assert_eq!(chunk.state(), ChunkState::Fetching);

        // Fetching -> Cached
        let data = vec![0xFF, 0xD8]; // JPEG magic bytes
        assert!(chunk.set_cached(data).is_ok());
        assert_eq!(chunk.state(), ChunkState::Cached);
    }

    #[test]
    fn test_chunk_invalid_state_transition() {
        let mut chunk = Chunk::new(0, 0, "GO2".to_string(), 12);

        // Can't cache without fetching first
        assert!(chunk.set_cached(vec![1, 2, 3]).is_err());
    }

    #[test]
    fn test_chunk_error_on_fetch() {
        let mut chunk = Chunk::new(0, 0, "GO2".to_string(), 12);

        assert!(chunk.set_fetching().is_ok());
        assert!(chunk.set_error().is_ok());
        assert_eq!(chunk.state(), ChunkState::Error);
    }

    #[test]
    fn test_chunk_data_storage() {
        let mut chunk = Chunk::new(0, 0, "GO2".to_string(), 12);
        let test_data = vec![1, 2, 3, 4, 5];

        chunk.set_fetching().unwrap();
        chunk.set_cached(test_data.clone()).unwrap();

        assert_eq!(chunk.data(), Some(&test_data[..]));
    }

    #[test]
    fn test_chunk_cache_key() {
        let chunk = Chunk::new(10, 20, "BI".to_string(), 15);
        assert_eq!(chunk.cache_key(), "10_20_BI_z15");
    }

    #[test]
    fn test_chunk_state_invalid_fetching_twice() {
        let mut chunk = Chunk::new(0, 0, "GO2".to_string(), 12);
        assert!(chunk.set_fetching().is_ok());
        assert!(chunk.set_fetching().is_err()); // Can't fetch twice
    }
}
