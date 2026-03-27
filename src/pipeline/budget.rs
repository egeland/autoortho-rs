use lru::LruCache;
use std::num::NonZeroUsize;
use std::path::PathBuf;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum BudgetError {
    #[error("Disk budget exceeded")]
    BudgetExceeded,
    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),
}

/// LRU disk budget manager for cache eviction
pub struct DiskBudgetManager {
    max_bytes: u64,
    current_bytes: u64,
    file_list: LruCache<String, u64>, // path -> size
    cache_dir: PathBuf,
}

impl DiskBudgetManager {
    pub fn new(max_bytes: u64, cache_dir: PathBuf) -> Self {
        // Use reasonable cache size for LRU (10000 items)
        let cache_size = NonZeroUsize::new(10000).expect("10000 is nonzero");
        Self {
            max_bytes,
            current_bytes: 0,
            file_list: LruCache::new(cache_size),
            cache_dir,
        }
    }

    /// Try to add a file, evicting LRU items if needed
    pub fn add_file(&mut self, key: String, size: u64) -> Result<(), BudgetError> {
        // If adding this file exceeds budget, evict oldest items
        if self.current_bytes + size > self.max_bytes {
            self.evict_until_fits(size)?;
        }

        self.file_list.put(key.clone(), size);
        self.current_bytes += size;

        Ok(())
    }

    /// Evict LRU items until there's space for new_size
    fn evict_until_fits(&mut self, new_size: u64) -> Result<(), BudgetError> {
        while self.current_bytes + new_size > self.max_bytes && !self.file_list.is_empty() {
            if let Some((key, size)) = self.file_list.pop_lru() {
                let path = self.cache_dir.join(format!("{}.dds.zst", key));
                if path.exists() {
                    std::fs::remove_file(path)?;
                }
                self.current_bytes = self.current_bytes.saturating_sub(size);
            } else {
                break;
            }
        }

        if self.current_bytes + new_size > self.max_bytes {
            return Err(BudgetError::BudgetExceeded);
        }

        Ok(())
    }

    /// Get current usage
    pub fn current_bytes(&self) -> u64 {
        self.current_bytes
    }

    /// Get max budget
    pub fn max_bytes(&self) -> u64 {
        self.max_bytes
    }

    /// Check if adding size would exceed budget
    pub fn would_exceed(&self, size: u64) -> bool {
        self.current_bytes + size > self.max_bytes
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_budget_manager_creation() {
        let tmp_dir = TempDir::new().unwrap();
        let mgr = DiskBudgetManager::new(1024 * 1024, tmp_dir.path().to_path_buf());

        assert_eq!(mgr.current_bytes(), 0);
        assert_eq!(mgr.max_bytes(), 1024 * 1024);
    }

    #[test]
    fn test_budget_add_file() {
        let tmp_dir = TempDir::new().unwrap();
        let mut mgr = DiskBudgetManager::new(1000, tmp_dir.path().to_path_buf());

        mgr.add_file("file1".to_string(), 500).unwrap();
        assert_eq!(mgr.current_bytes(), 500);

        mgr.add_file("file2".to_string(), 400).unwrap();
        assert_eq!(mgr.current_bytes(), 900);
    }

    #[test]
    fn test_budget_exceeded() {
        let tmp_dir = TempDir::new().unwrap();
        let mut mgr = DiskBudgetManager::new(1000, tmp_dir.path().to_path_buf());

        mgr.add_file("file1".to_string(), 600).unwrap();
        assert!(mgr.would_exceed(500)); // 600 + 500 > 1000

        // Adding 500 should succeed (file1 evicted), leaving only 500 used
        assert!(mgr.add_file("file2".to_string(), 500).is_ok());
    }

    #[test]
    fn test_budget_lru_eviction() {
        let tmp_dir = TempDir::new().unwrap();
        let mut mgr = DiskBudgetManager::new(1000, tmp_dir.path().to_path_buf());

        // Create dummy files
        mgr.add_file("file1".to_string(), 300).unwrap();
        mgr.add_file("file2".to_string(), 300).unwrap();
        mgr.add_file("file3".to_string(), 300).unwrap();

        // Now add a file that forces eviction
        // file1 is least recently used, should be evicted
        mgr.add_file("file4".to_string(), 300).unwrap();

        // Should have freed up space by removing file1
        assert!(mgr.current_bytes() <= 1000);
    }

    #[test]
    fn test_budget_would_exceed() {
        let tmp_dir = TempDir::new().unwrap();
        let mgr = DiskBudgetManager::new(1000, tmp_dir.path().to_path_buf());

        assert!(!mgr.would_exceed(500));
        assert!(!mgr.would_exceed(1000));
        assert!(mgr.would_exceed(1001));
    }
}
