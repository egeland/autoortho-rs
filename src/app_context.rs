// SPDX-License-Identifier: Apache-2.0 OR GPL-3.0
// Copyright (c) 2026 the AutoOrtho contributors

use parking_lot::{Mutex as ParkMutex, RwLock};
use std::error::Error;
use std::path::PathBuf;
use std::sync::Arc;

use crate::config::AutoOrthoConfig;
#[cfg(feature = "fuse")]
use crate::fuse::filesystem::DdsFileSystem;
use crate::pipeline::cache::DdsCache;
use crate::scenery::paths::scenery_data_dir;
use crate::stats::StatsStore;
use crate::tiles::fetcher::TileFetcher;
use crate::tiles::provider::ProviderFactory;
use crate::webui::custommap::CustomMapStore;
use crate::xplane::dataref::DatarefTracker;

#[derive(Clone)]
pub struct AppContext {
    pub config: Arc<RwLock<AutoOrthoConfig>>,
    pub stats: Arc<StatsStore>,
    pub tracker: Arc<DatarefTracker>,
    pub fetcher: Arc<TileFetcher>,
    pub dds_cache: Option<Arc<ParkMutex<DdsCache>>>,
    #[cfg(feature = "fuse")]
    pub fs: Arc<DdsFileSystem>,
    pub custom_map: Arc<CustomMapStore>,
    pub tile_progress: Arc<crate::ui::state::TileProgress>,
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
                Ok(cache) => Some(Arc::new(ParkMutex::new(cache))),
                Err(_) => None,
            }
        } else {
            None
        };

        // Create shared tile progress tracker for UI
        let tile_progress = Arc::new(crate::ui::state::TileProgress::new());

        let fs = {
            #[cfg(feature = "fuse")]
            {
                let mut builder = DdsFileSystem::builder(fetcher.clone(), &config.tile_provider)
                    .cache_entries(dds_cache_entries)
                    .custom_map(custom_map.clone())
                    .root(scenery_data_dir(&config.cache_dir))
                    .tile_progress(tile_progress.clone());

                if let Some(dc) = dds_cache.clone() {
                    builder = builder.disk_cache(dc);
                }

                Arc::new(builder.build())
            }
            #[cfg(not(feature = "fuse"))]
            {
                panic!("DdsFileSystem requires fuse feature");
            }
        };

        let stats = Arc::new(StatsStore::new());
        let tracker = Arc::new(DatarefTracker::new());

        Ok(Self {
            config: Arc::new(RwLock::new(config)),
            stats,
            tracker,
            fetcher,
            dds_cache,
            #[cfg(feature = "fuse")]
            fs,
            custom_map,
            tile_progress,
        })
    }
}

#[cfg(test)]
#[cfg(feature = "fuse")]
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
