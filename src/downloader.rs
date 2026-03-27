// SPDX-License-Identifier: Apache-2.0 OR GPL-3.0
// Copyright (c) 2024 the AutoOrtho contributors

use std::path::{Path, PathBuf};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum DownloadError {
    #[error("Download failed: {0}")]
    DownloadFailed(String),
    #[error("Extract failed: {0}")]
    ExtractFailed(String),
    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),
}

/// Scenery pack download manager
pub struct SceneryDownloader {
    download_dir: PathBuf,
    _cache_dir: PathBuf,
}

impl SceneryDownloader {
    pub fn new(download_dir: PathBuf, cache_dir: PathBuf) -> Self {
        Self {
            download_dir,
            _cache_dir: cache_dir,
        }
    }

    /// Download a scenery pack from URL
    pub async fn download(&self, url: &str, name: &str) -> Result<PathBuf, DownloadError> {
        let filename = self.download_dir.join(format!("{}.zip", name));

        // Simulate download (in real implementation, use reqwest)
        if url.is_empty() {
            return Err(DownloadError::DownloadFailed("Empty URL".to_string()));
        }

        Ok(filename)
    }

    /// Extract zip/7z file
    pub fn extract(&self, archive: &Path, extract_to: &Path) -> Result<(), DownloadError> {
        if !archive.exists() {
            return Err(DownloadError::ExtractFailed("File not found".to_string()));
        }

        // Check file extension
        let ext = archive.extension().and_then(|s| s.to_str()).unwrap_or("");

        match ext {
            "zip" => self.extract_zip(archive, extract_to),
            "7z" => self.extract_7z(archive, extract_to),
            _ => Err(DownloadError::ExtractFailed(
                "Unsupported format".to_string(),
            )),
        }
    }

    fn extract_zip(&self, _archive: &Path, _extract_to: &Path) -> Result<(), DownloadError> {
        // TODO: Implement with zip crate
        Ok(())
    }

    fn extract_7z(&self, _archive: &Path, _extract_to: &Path) -> Result<(), DownloadError> {
        // TODO: Implement with subprocess to 7zz
        Ok(())
    }

    /// Get download progress callback
    pub fn set_progress_callback<F>(&self, _callback: F)
    where
        F: Fn(u64, u64) + 'static,
    {
        // Store callback for progress updates
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_downloader_creation() {
        let tmp = TempDir::new().unwrap();
        let dl = SceneryDownloader::new(tmp.path().to_path_buf(), tmp.path().to_path_buf());

        assert_eq!(dl.download_dir, tmp.path().to_path_buf());
    }

    #[tokio::test]
    async fn test_download_empty_url() {
        let tmp = TempDir::new().unwrap();
        let dl = SceneryDownloader::new(tmp.path().to_path_buf(), tmp.path().to_path_buf());

        let result = dl.download("", "test").await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_download_creates_path() {
        let tmp = TempDir::new().unwrap();
        let dl = SceneryDownloader::new(tmp.path().to_path_buf(), tmp.path().to_path_buf());

        let result = dl.download("http://example.com/pack.zip", "test").await;
        assert!(result.is_ok());

        let path = result.unwrap();
        assert!(path.to_string_lossy().contains("test.zip"));
    }

    #[test]
    fn test_extract_missing_file() {
        let tmp = TempDir::new().unwrap();
        let dl = SceneryDownloader::new(tmp.path().to_path_buf(), tmp.path().to_path_buf());

        let missing = tmp.path().join("missing.zip").to_path_buf();
        let result = dl.extract(&missing, &tmp.path().to_path_buf());

        assert!(result.is_err());
    }

    #[test]
    fn test_extract_unsupported_format() {
        let tmp = TempDir::new().unwrap();
        let dl = SceneryDownloader::new(tmp.path().to_path_buf(), tmp.path().to_path_buf());

        // Create a dummy .rar file
        let file = tmp.path().join("test.rar").to_path_buf();
        std::fs::write(&file, b"dummy").unwrap();

        let result = dl.extract(&file, &tmp.path().to_path_buf());
        assert!(result.is_err());
    }
}
