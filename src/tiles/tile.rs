use crate::pipeline::dds;
use crate::tiles::chunk::Chunk;
use lru::LruCache;
use std::num::NonZeroUsize;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum TileError {
    #[error("Invalid mipmap level")]
    InvalidMipmap,
    #[error("Tile not ready")]
    NotReady,
}

/// A tile is a 16x16 grid of 256x256 chunks = 4096x4096 pixels
pub struct Tile {
    pub row: u32,
    pub col: u32,
    pub maptype: String,
    chunks: Vec<Vec<Chunk>>, // 16x16 grid
}

impl Tile {
    pub fn new(row: u32, col: u32, maptype: String) -> Self {
        let mut chunks = Vec::new();
        for r in 0..16 {
            let mut row_chunks = Vec::new();
            for c in 0..16 {
                row_chunks.push(Chunk::new(r, c, maptype.clone(), 0));
            }
            chunks.push(row_chunks);
        }

        Self {
            row,
            col,
            maptype,
            chunks,
        }
    }

    /// Get chunk at (r, c) in 16x16 grid
    pub fn chunk(&self, r: u32, c: u32) -> Option<&Chunk> {
        if r < 16 && c < 16 {
            Some(&self.chunks[r as usize][c as usize])
        } else {
            None
        }
    }

    /// Get mutable chunk
    pub fn chunk_mut(&mut self, r: u32, c: u32) -> Option<&mut Chunk> {
        if r < 16 && c < 16 {
            Some(&mut self.chunks[r as usize][c as usize])
        } else {
            None
        }
    }

    /// Calculate DDS file size for mipmap chain at zoom level
    pub fn dds_size_for_zoom(&self, _zoom: u32) -> usize {
        dds::dds_file_size_4096_bc3()
    }
}

/// LRU cache of tiles
pub struct TileCacher {
    cache: LruCache<String, Tile>,
}

impl TileCacher {
    pub fn new(capacity: usize) -> Self {
        let cache_size =
            NonZeroUsize::new(capacity).unwrap_or(NonZeroUsize::new(100).expect("100 is nonzero"));
        Self {
            cache: LruCache::new(cache_size),
        }
    }

    pub fn get(&mut self, row: u32, col: u32, maptype: &str) -> Option<&Tile> {
        let key = format!("{}_{}_z{}", row, col, maptype);
        self.cache.get(&key)
    }

    pub fn get_mut(&mut self, row: u32, col: u32, maptype: &str) -> Option<&mut Tile> {
        let key = format!("{}_{}_z{}", row, col, maptype);
        self.cache.get_mut(&key)
    }

    pub fn insert(&mut self, tile: Tile) -> Option<Tile> {
        let key = format!("{}_{}_z{}", tile.row, tile.col, tile.maptype);
        self.cache.put(key, tile)
    }

    pub fn len(&self) -> usize {
        self.cache.len()
    }

    pub fn is_empty(&self) -> bool {
        self.cache.is_empty()
    }

    pub fn clear(&mut self) {
        self.cache.clear();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tile_creation() {
        let tile = Tile::new(10, 20, "GO2".to_string());
        assert_eq!(tile.row, 10);
        assert_eq!(tile.col, 20);
    }

    #[test]
    fn test_tile_16x16_grid() {
        let tile = Tile::new(0, 0, "GO2".to_string());

        // All 16x16 chunks should exist
        for r in 0..16 {
            for c in 0..16 {
                assert!(tile.chunk(r, c).is_some());
            }
        }

        // Out of bounds should return None
        assert!(tile.chunk(16, 0).is_none());
        assert!(tile.chunk(0, 16).is_none());
    }

    #[test]
    fn test_tile_chunk_access_mut() {
        let mut tile = Tile::new(0, 0, "GO2".to_string());

        {
            let chunk = tile.chunk_mut(5, 7).unwrap();
            chunk.set_fetching().unwrap();
            chunk.set_cached(vec![1, 2, 3]).unwrap();
        }

        // Verify it was stored
        let chunk = tile.chunk(5, 7).unwrap();
        assert_eq!(
            chunk.data().map(|d| d.as_ref().as_slice()),
            Some(&[1, 2, 3][..])
        );
    }

    #[test]
    fn test_tile_dds_size() {
        let tile = Tile::new(0, 0, "GO2".to_string());
        let size = tile.dds_size_for_zoom(12);
        assert!(size > 128); // At least header
    }

    #[test]
    fn test_tile_cacher_basic() {
        let mut cacher = TileCacher::new(10);

        let tile1 = Tile::new(0, 0, "GO2".to_string());
        let tile2 = Tile::new(1, 1, "BI".to_string());

        cacher.insert(tile1);
        cacher.insert(tile2);

        assert_eq!(cacher.len(), 2);
        assert!(cacher.get(0, 0, "GO2").is_some());
        assert!(cacher.get(1, 1, "BI").is_some());
    }

    #[test]
    fn test_tile_cacher_lru_eviction() {
        let mut cacher = TileCacher::new(2);

        let tile1 = Tile::new(0, 0, "GO2".to_string());
        let tile2 = Tile::new(1, 1, "BI".to_string());
        let tile3 = Tile::new(2, 2, "ARC".to_string());

        cacher.insert(tile1);
        cacher.insert(tile2);
        cacher.insert(tile3); // Should evict tile1 (least recently used)

        assert_eq!(cacher.len(), 2);
        assert!(cacher.get(0, 0, "GO2").is_none()); // Evicted
        assert!(cacher.get(1, 1, "BI").is_some());
        assert!(cacher.get(2, 2, "ARC").is_some());
    }

    #[test]
    fn test_tile_cacher_clear() {
        let mut cacher = TileCacher::new(10);

        cacher.insert(Tile::new(0, 0, "GO2".to_string()));
        cacher.insert(Tile::new(1, 1, "BI".to_string()));

        assert!(!cacher.is_empty());
        cacher.clear();
        assert!(cacher.is_empty());
    }
}
