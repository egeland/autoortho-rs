// SPDX-License-Identifier: Apache-2.0 OR GPL-3.0
// Copyright (c) 2024 the AutoOrtho contributors

use parking_lot::{Mutex, RwLock};
use std::error::Error;
use std::path::PathBuf;
use std::sync::Arc;

use crate::config::AutoOrthoConfig;
use crate::fuse::filesystem::DdsFileSystem;
use crate::pipeline::cache::DdsCache;
use crate::stats::StatsStore;
use crate::tiles::fetcher::TileFetcher;
use crate::tiles::provider::ProviderFactory;
use crate::webui::custommap::CustomMapStore;
use crate::xplane::dataref::DatarefTracker;

pub struct AppContext {
    pub config: Arc<RwLock<AutoOrthoConfig>>,
    pub stats: Arc<StatsStore>,
    pub tracker: Arc<DatarefTracker>,
    pub fetcher: Arc<TileFetcher>,
    pub dds_cache: Option<Arc<Mutex<DdsCache>>>,
    pub fs: Arc<DdsFileSystem>,
    pub custom_map: Arc<CustomMapStore>,
}

impl AppContext {
    pub async fn init(config: AutoOrthoConfig) -> Result<Self, Box<dyn Error>> {
        let _provider = ProviderFactory::create(&config.tile_provider)
            .ok_or_else(|| format!("Unknown tile provider: {}", config.tile_provider))?;

        let custom_map_path = dirs::config_dir()
            .unwrap_or_else(|| PathBuf::from("."))
            .join("autoortho")
            .join("custom_map.json");
        let custom_map = CustomMapStore::load(custom_map_path);

        let chunk_cache_entries = config.chunk_memory_cache_entries();
        let dds_cache_entries = config.dds_memory_cache_entries();

        let fetcher = Arc::new(TileFetcher::with_cache_size(
            chunk_cache_entries,
            &config.tile_provider,
        ));

        let dds_cache = if config.enable_dds_cache {
            let cache_dir = PathBuf::from(&config.cache_dir).join("dds");
            match DdsCache::open(cache_dir, config.dds_cache_size_mb * 1024 * 1024) {
                Ok(cache) => Some(Arc::new(Mutex::new(cache))),
                Err(_) => None,
            }
        } else {
            None
        };

        let fs = if let Some(dc) = dds_cache.clone() {
            Arc::new(DdsFileSystem::with_disk_cache_and_custom_map(
                fetcher.clone(),
                dc,
                custom_map.clone(),
                &config.tile_provider,
                dds_cache_entries,
            ))
        } else {
            Arc::new(DdsFileSystem::new_with_custom_map(
                fetcher.clone(),
                custom_map.clone(),
                &config.tile_provider,
                dds_cache_entries,
            ))
        };

        let stats = Arc::new(StatsStore::new());
        let tracker = Arc::new(DatarefTracker::new());

        Ok(Self {
            config: Arc::new(RwLock::new(config)),
            stats,
            tracker,
            fetcher,
            dds_cache,
            fs,
            custom_map,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[tokio::test]
    async fn test_app_context_init_default() {
        let tmp = TempDir::new().unwrap();
        let mut config = AutoOrthoConfig::default();
        config.cache_dir = tmp.path().to_string_lossy().to_string();

        let context = AppContext::init(config)
            .await
            .expect("Failed to init context");
        assert_eq!(context.config.read().tile_provider, "ARC");
    }
}
