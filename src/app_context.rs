// SPDX-License-Identifier: Apache-2.0 OR GPL-3.0
// Copyright (c) 2026 the AutoOrtho contributors

use parking_lot::{Mutex as ParkMutex, RwLock};
use std::error::Error;
use std::path::PathBuf;
use std::sync::Arc;
#[cfg(all(target_os = "windows", feature = "fuse"))]
use std::sync::Mutex;

use crate::config::AutoOrthoConfig;
#[cfg(feature = "fuse")]
use crate::fuse::filesystem::DdsFileSystem;
#[cfg(all(target_os = "windows", feature = "fuse"))]
use crate::fuse::mount_win::AutoOrthoHandler;
use crate::pipeline::cache::DdsCache;
use crate::stats::StatsStore;
use crate::tiles::fetcher::TileFetcher;
use crate::tiles::provider::ProviderFactory;
use crate::webui::custommap::CustomMapStore;
use crate::xplane::dataref::DatarefTracker;

/// Opaque handle type for storing Dokan filesystem on Windows
/// This prevents generic parameter pollution while keeping the mount alive
#[cfg(all(target_os = "windows", feature = "fuse"))]
pub type DokanMountHandle =
    Arc<Mutex<Option<dokan::FileSystem<'static, 'static, AutoOrthoHandler>>>>;

#[derive(Clone)]
pub struct AppContext {
    pub config: Arc<RwLock<AutoOrthoConfig>>,
    pub stats: Arc<StatsStore>,
    pub tracker: Arc<DatarefTracker>,
    pub fetcher: Arc<TileFetcher>,
    pub dds_cache: Option<Arc<ParkMutex<DdsCache>>>,
    #[cfg(feature = "fuse")]
    pub fs: Arc<DdsFileSystem>,
    #[cfg(all(target_os = "windows", feature = "fuse"))]
    /// Dokan filesystem handle - must be kept alive to maintain mount
    pub dokan_mount: DokanMountHandle,
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
                Ok(cache) => Some(Arc::new(ParkMutex::new(cache))),
                Err(_) => None,
            }
        } else {
            None
        };

        let fs = {
            #[cfg(feature = "fuse")]
            {
                let mut builder = DdsFileSystem::builder(fetcher.clone(), &config.tile_provider)
                    .cache_entries(dds_cache_entries)
                    .custom_map(custom_map.clone())
                    .root(config.scenery_data_dir());

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
            #[cfg(all(target_os = "windows", feature = "fuse"))]
            dokan_mount: Arc::new(Mutex::new(None)),
            custom_map,
        })
    }

    /// Set the Dokan filesystem handle after successful mount (Windows only)
    /// This keeps the mount alive for the duration of the application
    #[cfg(all(target_os = "windows", feature = "fuse"))]
    pub fn set_dokan_mount(&self, fs: dokan::FileSystem<'static, 'static, AutoOrthoHandler>) {
        if let Some(handle) = self.dokan_mount.lock().ok() {
            *handle = Some(fs);
        }
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
