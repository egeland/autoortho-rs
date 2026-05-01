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
    #[error("Zip error: {0}")]
    ZipError(#[from] zip::result::ZipError),
}

/// Validate that an extracted path stays within the target directory.
/// This prevents ZIP slip attacks where a malicious zip contains paths like "../../../etc/passwd".
fn validate_extract_path(target_dir: &Path, entry_path: &Path) -> Result<PathBuf, DownloadError> {
    let canonical_target = target_dir
        .canonicalize()
        .map_err(|e| DownloadError::ExtractFailed(format!("Cannot resolve target: {}", e)))?;

    // Block absolute paths
    if entry_path.is_absolute() {
        return Err(DownloadError::ExtractFailed(format!(
            "Absolute path blocked: {}",
            entry_path.display()
        )));
    }

    // Check for parent directory traversal
    for component in entry_path.components() {
        if let std::path::Component::ParentDir = component {
            return Err(DownloadError::ExtractFailed(format!(
                "Path traversal attempt blocked: {}",
                entry_path.display()
            )));
        }
    }

    let out_path = target_dir.join(entry_path);

    // For existing paths, verify canonicalization
    if out_path.exists() {
        let canonical = out_path.canonicalize().map_err(|_| {
            DownloadError::ExtractFailed(format!(
                "Path traversal attempt blocked: {}",
                entry_path.display()
            ))
        })?;

        if !canonical.starts_with(&canonical_target) {
            return Err(DownloadError::ExtractFailed(format!(
                "Path traversal attempt blocked: {}",
                entry_path.display()
            )));
        }
    }

    Ok(out_path)
}

/// Scenery pack download manager
pub struct SceneryDownloader {
    download_dir: PathBuf,
}

impl SceneryDownloader {
    pub fn new(download_dir: PathBuf) -> Self {
        Self { download_dir }
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

    fn extract_zip(&self, archive: &Path, extract_to: &Path) -> Result<(), DownloadError> {
        use std::fs::File;
        use zip::ZipArchive;

        let file = File::open(archive)?;
        let mut archive = ZipArchive::new(file)?;

        for i in 0..archive.len() {
            let mut file = archive.by_index(i)?;
            let entry_name = file.name();

            // Validate the entry path (blocks path traversal)
            let outpath = validate_extract_path(extract_to, Path::new(entry_name))?;

            if file.name().ends_with('/') {
                std::fs::create_dir_all(&outpath)?;
            } else {
                if let Some(p) = outpath.parent()
                    && !p.exists()
                {
                    std::fs::create_dir_all(p)?;
                }
                let mut outfile = File::create(&outpath)?;
                std::io::copy(&mut file, &mut outfile)?;
            }
        }

        Ok(())
    }

    #[allow(dead_code)]
    fn extract_zip_from_memory(&self, data: &[u8], extract_to: &Path) -> Result<(), DownloadError> {
        use std::io::Cursor;
        use zip::ZipArchive;

        let cursor = Cursor::new(data);
        let mut archive = ZipArchive::new(cursor)?;

        for i in 0..archive.len() {
            let mut file = archive.by_index(i)?;
            let entry_name = file.name();

            // Validate the entry path
            let outpath = validate_extract_path(extract_to, Path::new(entry_name))?;

            if file.name().ends_with('/') {
                std::fs::create_dir_all(&outpath)?;
            } else {
                if let Some(p) = outpath.parent()
                    && !p.exists()
                {
                    std::fs::create_dir_all(p)?;
                }
                let mut outfile = std::fs::File::create(&outpath)?;
                std::io::copy(&mut file, &mut outfile)?;
            }
        }

        Ok(())
    }

    fn extract_7z(&self, archive: &Path, extract_to: &Path) -> Result<(), DownloadError> {
        use sevenz_rust::{
            Error as SevenZError, SevenZArchiveEntry, decompress_file_with_extract_fn,
        };
        use std::fs::File;
        use std::io::{Read, copy};
        use std::path::Path;

        let extract_to_owned = extract_to.to_path_buf();

        let extract_fn = |entry: &SevenZArchiveEntry, reader: &mut dyn Read, _dest: &PathBuf| {
            let entry_path = Path::new(entry.name());
            let validated_path =
                validate_extract_path(&extract_to_owned, entry_path).map_err(|e| {
                    let io_err = std::io::Error::other(e);
                    SevenZError::Io(io_err, "path validation failed".into())
                })?;

            if entry.is_directory() {
                std::fs::create_dir_all(&validated_path)
                    .map_err(|e| SevenZError::Io(e, "failed to create directory".into()))?;
            } else {
                if let Some(parent) = validated_path.parent() {
                    std::fs::create_dir_all(parent).map_err(|e| {
                        SevenZError::Io(e, "failed to create parent directory".into())
                    })?;
                }
                let mut outfile = File::create(&validated_path)
                    .map_err(|e| SevenZError::Io(e, "failed to create output file".into()))?;
                copy(reader, &mut outfile)
                    .map_err(|e| SevenZError::Io(e, "failed to copy content".into()))?;
            }
            Ok(true)
        };

        decompress_file_with_extract_fn(archive, extract_to, extract_fn)
            .map_err(|e| DownloadError::ExtractFailed(e.to_string()))?;

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
        let dl = SceneryDownloader::new(tmp.path().to_path_buf());

        assert_eq!(dl.download_dir, tmp.path().to_path_buf());
    }

    #[tokio::test]
    async fn test_download_empty_url() {
        let tmp = TempDir::new().unwrap();
        let dl = SceneryDownloader::new(tmp.path().to_path_buf());

        let result = dl.download("", "test").await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_download_creates_path() {
        let tmp = TempDir::new().unwrap();
        let dl = SceneryDownloader::new(tmp.path().to_path_buf());

        let result = dl.download("http://example.com/pack.zip", "test").await;
        assert!(result.is_ok());

        let path = result.unwrap();
        assert!(path.to_string_lossy().contains("test.zip"));
    }

    #[test]
    fn test_extract_missing_file() {
        let tmp = TempDir::new().unwrap();
        let dl = SceneryDownloader::new(tmp.path().to_path_buf());

        let missing = tmp.path().join("missing.zip").to_path_buf();
        let result = dl.extract(&missing, &tmp.path().to_path_buf());

        assert!(result.is_err());
    }

    #[test]
    fn test_extract_unsupported_format() {
        let tmp = TempDir::new().unwrap();
        let dl = SceneryDownloader::new(tmp.path().to_path_buf());

        // Create a dummy .rar file
        let file = tmp.path().join("test.rar").to_path_buf();
        std::fs::write(&file, b"dummy").unwrap();

        let result = dl.extract(&file, &tmp.path().to_path_buf());
        assert!(result.is_err());
    }

    #[test]
    fn test_extract_zip_blocks_path_traversal() {
        use std::io::Write;
        use zip::ZipWriter;
        use zip::write::SimpleFileOptions;

        let tmp = TempDir::new().unwrap();
        let dl = SceneryDownloader::new(tmp.path().to_path_buf());

        let zip_path = tmp.path().join("malicious.zip");

        // Create a zip with path traversal
        let file = std::fs::File::create(&zip_path).unwrap();
        let mut zip = ZipWriter::new(file);
        let options =
            SimpleFileOptions::default().compression_method(zip::CompressionMethod::Deflated);

        zip.start_file("../../../etc/evil.txt", options).unwrap();
        zip.write_all(b"malicious").unwrap();
        zip.finish().unwrap();

        // Extraction should fail
        let result = dl.extract(&zip_path, tmp.path());
        assert!(result.is_err(), "Path traversal should be blocked");
    }

    #[test]
    fn test_extract_zip_normal_file() {
        use std::io::Write;
        use zip::ZipWriter;
        use zip::write::SimpleFileOptions;

        let tmp = TempDir::new().unwrap();
        let dl = SceneryDownloader::new(tmp.path().to_path_buf());

        let zip_path = tmp.path().join("normal.zip");

        // Create a normal zip
        let file = std::fs::File::create(&zip_path).unwrap();
        let mut zip = ZipWriter::new(file);
        let options =
            SimpleFileOptions::default().compression_method(zip::CompressionMethod::Deflated);

        // Add directory first
        zip.add_directory("subdir", options).unwrap();
        zip.start_file("subdir/test.txt", options).unwrap();
        zip.write_all(b"hello").unwrap();
        zip.finish().unwrap();

        // Extraction should succeed
        let result = dl.extract(&zip_path, tmp.path());
        if let Err(e) = result {
            panic!("Extract failed: {}", e);
        }
        assert!(tmp.path().join("subdir/test.txt").exists());
    }
}
