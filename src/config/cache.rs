// SPDX-License-Identifier: Apache-2.0 OR GPL-3.0
// Copyright (c) 2026 the AutoOrtho contributors

use serde::{Deserialize, Serialize};

use crate::errors::{ConfigError, validate_range};

/// Cache configuration — disk and memory cache sizes.
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct CacheConfig {
    #[serde(default = "default_dds_cache_size_mb")]
    pub dds_cache_size_mb: u64,
    #[serde(default = "default_enable_dds_cache")]
    pub enable_dds_cache: bool,
    #[serde(default = "default_dds_memory_cache_mb")]
    pub dds_memory_cache_mb: u64,
    #[serde(default = "default_chunk_memory_cache_mb")]
    pub chunk_memory_cache_mb: u64,
}

impl Default for CacheConfig {
    fn default() -> Self {
        Self {
            dds_cache_size_mb: default_dds_cache_size_mb(),
            enable_dds_cache: default_enable_dds_cache(),
            dds_memory_cache_mb: default_dds_memory_cache_mb(),
            chunk_memory_cache_mb: default_chunk_memory_cache_mb(),
        }
    }
}

impl CacheConfig {
    /// Estimated memory per DDS tile in MB (4096x4096 BC3 compressed).
    const DDS_TILE_SIZE_MB: u64 = 22;

    /// Estimated memory per chunk in KB (256x256 JPEG).
    const CHUNK_SIZE_KB: u64 = 30;

    /// Calculate the number of DDS tiles that fit in the configured memory.
    pub fn dds_memory_cache_entries(&self) -> usize {
        ((self.dds_memory_cache_mb / Self::DDS_TILE_SIZE_MB).max(1)) as usize
    }

    /// Calculate the number of chunks that fit in the configured memory.
    pub fn chunk_memory_cache_entries(&self) -> usize {
        ((self.chunk_memory_cache_mb * 1024 / Self::CHUNK_SIZE_KB).max(1)) as usize
    }

    pub fn validate(&self) -> Result<(), ConfigError> {
        validate_range(self.dds_cache_size_mb, 0, 102400, "dds_cache_size_mb")?;
        Ok(())
    }
}

fn default_dds_cache_size_mb() -> u64 {
    4096
}

fn default_enable_dds_cache() -> bool {
    true
}

fn default_dds_memory_cache_mb() -> u64 {
    256
}

fn default_chunk_memory_cache_mb() -> u64 {
    512
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cache_config_defaults() {
        let config = CacheConfig::default();
        assert_eq!(config.dds_cache_size_mb, 4096);
        assert!(config.enable_dds_cache);
        assert_eq!(config.dds_memory_cache_mb, 256);
        assert_eq!(config.chunk_memory_cache_mb, 512);
    }

    #[test]
    fn test_dds_memory_cache_entries() {
        let config = CacheConfig::default();
        // 256 / 22 ≈ 11
        assert_eq!(config.dds_memory_cache_entries(), 11);
    }

    #[test]
    fn test_dds_memory_cache_entries_custom() {
        let mut config = CacheConfig::default();
        config.dds_memory_cache_mb = 4096;
        // 4096 / 22 ≈ 186
        assert_eq!(config.dds_memory_cache_entries(), 186);
    }

    #[test]
    fn test_dds_memory_cache_entries_minimum() {
        let mut config = CacheConfig::default();
        config.dds_memory_cache_mb = 0;
        assert_eq!(config.dds_memory_cache_entries(), 1);
    }

    #[test]
    fn test_chunk_memory_cache_entries() {
        let config = CacheConfig::default();
        // 512 * 1024 / 30 ≈ 17476
        assert_eq!(config.chunk_memory_cache_entries(), 17476);
    }

    #[test]
    fn test_cache_config_validate_valid() {
        let config = CacheConfig::default();
        assert!(config.validate().is_ok());
    }

    #[test]
    fn test_cache_config_validate_invalid() {
        let mut config = CacheConfig::default();
        config.dds_cache_size_mb = 200000;
        assert!(config.validate().is_err());
    }

    #[test]
    fn test_cache_config_serde_roundtrip() {
        let config = CacheConfig::default();
        let toml = toml::to_string(&config).unwrap();
        let loaded: CacheConfig = toml::from_str(&toml).unwrap();
        assert_eq!(config.dds_cache_size_mb, loaded.dds_cache_size_mb);
    }
}
