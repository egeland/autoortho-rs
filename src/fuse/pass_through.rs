// SPDX-License-Identifier: Apache-2.0 OR GPL-3.0
// Copyright (c) 2026 the AutoOrtho contributors

//! Real filesystem pass-through operations.
//!
//! Handles attribute lookups and directory listings for files that exist
//! on the real filesystem (scenery root), as opposed to virtual DDS tiles.

use crate::fuse::filesystem::FileAttr;
use crate::fuse::{FuseError, VIRTUAL_DIRS};
use std::path::PathBuf;

/// Pass-through filesystem for real files in the scenery root.
///
/// When X-Plane requests a file that isn't a virtual DDS tile,
/// this module checks if it exists in the real scenery directory.
pub(crate) struct PassThroughFs {
    root: PathBuf,
}

impl PassThroughFs {
    pub fn new(root: PathBuf) -> Self {
        Self { root }
    }

    /// Get attributes for a real file or directory in the root.
    pub fn get_attr(&self, trimmed: &str) -> Result<FileAttr, FuseError> {
        let full_path = self.root.join(trimmed);
        if !full_path.exists() {
            return Err(FuseError::InvalidPath);
        }
        let meta = std::fs::metadata(&full_path).map_err(|e| FuseError::IoError(e.to_string()))?;
        if meta.is_dir() {
            Ok(FileAttr::directory())
        } else {
            Ok(FileAttr::file(meta.len()))
        }
    }

    /// List entries in a real directory within the root.
    pub fn list_dir(&self, trimmed: &str) -> Result<Vec<String>, FuseError> {
        let full_path = self.root.join(trimmed);
        if !full_path.is_dir() {
            return Err(FuseError::InvalidPath);
        }
        let mut entries = vec![".".to_string(), "..".to_string()];
        if let Ok(read_dir) = std::fs::read_dir(&full_path) {
            for entry in read_dir.flatten() {
                entries.push(entry.file_name().to_string_lossy().to_string());
            }
        }
        Ok(entries)
    }

    /// List root-level entries (virtual dirs + real files).
    pub fn root_entries(&self) -> Vec<String> {
        let mut entries = vec![".".to_string(), "..".to_string()];
        for dir in VIRTUAL_DIRS {
            entries.push(dir.to_string());
        }
        if let Ok(read_dir) = std::fs::read_dir(&self.root) {
            for entry in read_dir.flatten() {
                let name = entry.file_name().to_string_lossy().to_string();
                if !VIRTUAL_DIRS.contains(&name.as_str()) {
                    entries.push(name);
                }
            }
        }
        entries
    }

    /// Check if an entry in a directory path exists as a real directory.
    pub fn is_dir_in_root(&self, dir_path: &str, entry_name: &str) -> bool {
        let trimmed = dir_path.trim_start_matches('/');
        let full_path = if trimmed.is_empty() {
            self.root.join(entry_name)
        } else {
            self.root.join(trimmed).join(entry_name)
        };
        full_path.is_dir()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_pass_through() -> (PassThroughFs, tempfile::TempDir) {
        let tmp = tempfile::TempDir::new().unwrap();
        let fs = PassThroughFs::new(tmp.path().to_path_buf());
        (fs, tmp)
    }

    #[test]
    fn test_get_attr_real_file() {
        let (fs, tmp) = make_pass_through();
        std::fs::write(tmp.path().join("test.dsf"), b"hello").unwrap();

        let attr = fs.get_attr("test.dsf").unwrap();
        assert!(!attr.is_dir);
        assert_eq!(attr.size, 5);
    }

    #[test]
    fn test_get_attr_real_dir() {
        let (fs, tmp) = make_pass_through();
        std::fs::create_dir(tmp.path().join("scenery")).unwrap();

        let attr = fs.get_attr("scenery").unwrap();
        assert!(attr.is_dir);
    }

    #[test]
    fn test_get_attr_not_found() {
        let (fs, _tmp) = make_pass_through();
        assert!(fs.get_attr("nonexistent").is_err());
    }

    #[test]
    fn test_list_dir_real() {
        let (fs, tmp) = make_pass_through();
        std::fs::write(tmp.path().join("a.txt"), b"").unwrap();
        std::fs::create_dir(tmp.path().join("subdir")).unwrap();

        let entries = fs.list_dir("").unwrap();
        assert!(entries.contains(&"a.txt".to_string()));
        assert!(entries.contains(&"subdir".to_string()));
        assert!(entries.contains(&".".to_string()));
        assert!(entries.contains(&"..".to_string()));
    }

    #[test]
    fn test_list_dir_not_dir() {
        let (fs, tmp) = make_pass_through();
        std::fs::write(tmp.path().join("file.txt"), b"").unwrap();
        assert!(fs.list_dir("file.txt").is_err());
    }

    #[test]
    fn test_root_entries() {
        let (fs, tmp) = make_pass_through();
        std::fs::write(tmp.path().join("sa_info.json"), b"").unwrap();
        std::fs::create_dir(tmp.path().join("scenery")).unwrap();

        let entries = fs.root_entries();
        assert!(entries.contains(&"textures".to_string()));
        assert!(entries.contains(&"terrain".to_string()));
        assert!(entries.contains(&"sa_info.json".to_string()));
        assert!(entries.contains(&"scenery".to_string()));
    }

    #[test]
    fn test_root_entries_excludes_virtual_dirs_from_real() {
        let (fs, tmp) = make_pass_through();
        // If someone creates a real "textures" dir, it shouldn't appear twice
        std::fs::create_dir(tmp.path().join("textures")).unwrap();

        let entries = fs.root_entries();
        let texture_count = entries.iter().filter(|e| *e == "textures").count();
        assert_eq!(texture_count, 1, "virtual textures should appear once");
    }

    #[test]
    fn test_is_dir_in_root() {
        let (fs, tmp) = make_pass_through();
        std::fs::create_dir_all(tmp.path().join("scenery/z_ao")).unwrap();

        assert!(fs.is_dir_in_root("/", "scenery"));
        assert!(fs.is_dir_in_root("/scenery", "z_ao"));
        assert!(!fs.is_dir_in_root("/", "nonexistent"));
    }
}
