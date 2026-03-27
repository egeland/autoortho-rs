//! FUSE mount implementation using the `fuser` crate.
//!
//! This module is only compiled when the `fuse` feature is enabled,
//! which requires macFUSE (macOS), libfuse (Linux), or WinFsp (Windows)
//! to be installed.
//!
//! It translates between the fuser trait interface and our DdsFileSystem.

use crate::fuse::filesystem::DdsFileSystem;
use crate::fuse::{MARKER_FILE, VIRTUAL_DIRS, is_poison_path};
use fuser::{
    FileType, Filesystem, MountOption, ReplyAttr, ReplyData, ReplyDirectory, ReplyEntry, Request,
};
use libc::ENOENT;
use log::{debug, info, warn};
use std::collections::HashMap;
use std::ffi::OsStr;
use std::path::Path;
use std::sync::{Arc, Mutex};
use std::time::{Duration, SystemTime};

const TTL: Duration = Duration::from_secs(1);

/// Inode allocation for the virtual filesystem.
/// FUSE works with inode numbers, not paths. We map paths → inodes dynamically.
const ROOT_INO: u64 = 1;
const TEXTURES_INO: u64 = 2;
const TERRAIN_INO: u64 = 3;
const MARKER_INO: u64 = 4;
// Dynamic inodes for DDS files start here
const DYNAMIC_INO_START: u64 = 1000;

/// The fuser Filesystem implementation that wraps our DdsFileSystem.
pub struct AutoOrthoFuse {
    fs: Arc<DdsFileSystem>,
    runtime: tokio::runtime::Handle,
    /// Map inode → path for dynamic DDS files
    inode_to_path: Mutex<HashMap<u64, String>>,
    /// Map path → inode
    path_to_inode: Mutex<HashMap<String, u64>>,
    /// Next available inode number
    next_inode: Mutex<u64>,
}

impl AutoOrthoFuse {
    pub fn new(fs: Arc<DdsFileSystem>, runtime: tokio::runtime::Handle) -> Self {
        Self {
            fs,
            runtime,
            inode_to_path: Mutex::new(HashMap::new()),
            path_to_inode: Mutex::new(HashMap::new()),
            next_inode: Mutex::new(DYNAMIC_INO_START),
        }
    }

    /// Allocate or retrieve an inode for a path.
    fn get_or_create_inode(&self, path: &str) -> u64 {
        // Check well-known inodes
        match path {
            "/" => return ROOT_INO,
            "/textures" => return TEXTURES_INO,
            "/terrain" => return TERRAIN_INO,
            _ => {}
        }

        // Check if path ends with marker file
        if path.ends_with(MARKER_FILE) {
            return MARKER_INO;
        }

        // Check existing mapping
        let mut p2i = self
            .path_to_inode
            .lock()
            .expect("fuse inode mutex poisoned");
        if let Some(&ino) = p2i.get(path) {
            return ino;
        }

        // Allocate new inode
        let mut next = self.next_inode.lock().expect("fuse inode mutex poisoned");
        let ino = *next;
        *next += 1;

        p2i.insert(path.to_string(), ino);
        self.inode_to_path
            .lock()
            .expect("fuse inode mutex poisoned")
            .insert(ino, path.to_string());

        ino
    }

    /// Look up the path for an inode.
    fn inode_path(&self, ino: u64) -> Option<String> {
        match ino {
            ROOT_INO => Some("/".to_string()),
            TEXTURES_INO => Some("/textures".to_string()),
            TERRAIN_INO => Some("/terrain".to_string()),
            _ => self
                .inode_to_path
                .lock()
                .expect("fuse inode mutex poisoned")
                .get(&ino)
                .cloned(),
        }
    }

    fn now() -> SystemTime {
        SystemTime::now()
    }

    fn make_dir_attr(ino: u64) -> fuser::FileAttr {
        fuser::FileAttr {
            ino,
            size: 4096,
            blocks: 8,
            atime: Self::now(),
            mtime: Self::now(),
            ctime: Self::now(),
            crtime: Self::now(),
            kind: FileType::Directory,
            perm: 0o755,
            nlink: 2,
            uid: unsafe { libc::getuid() },
            gid: unsafe { libc::getgid() },
            rdev: 0,
            blksize: 4096,
            flags: 0,
        }
    }

    fn make_file_attr(ino: u64, size: u64) -> fuser::FileAttr {
        fuser::FileAttr {
            ino,
            size,
            blocks: size.div_ceil(4096) * 8,
            atime: Self::now(),
            mtime: Self::now(),
            ctime: Self::now(),
            crtime: Self::now(),
            kind: FileType::RegularFile,
            perm: 0o644,
            nlink: 1,
            uid: unsafe { libc::getuid() },
            gid: unsafe { libc::getgid() },
            rdev: 0,
            blksize: 4096,
            flags: 0,
        }
    }
}

impl Filesystem for AutoOrthoFuse {
    fn lookup(&mut self, _req: &Request, parent: u64, name: &OsStr, reply: ReplyEntry) {
        let name_str = name.to_string_lossy();
        let parent_path = match self.inode_path(parent) {
            Some(p) => p,
            None => {
                reply.error(ENOENT);
                return;
            }
        };

        let full_path = if parent_path == "/" {
            format!("/{}", name_str)
        } else {
            format!("{}/{}", parent_path, name_str)
        };

        debug!(
            "lookup: parent={} name={} → {}",
            parent, name_str, full_path
        );

        // Poison pill check
        if is_poison_path(&full_path) {
            info!("Poison pill detected at {}. Shutting down.", full_path);
            reply.error(ENOENT);
            // TODO: trigger fuse_exit
            return;
        }

        // Use the async filesystem to get attributes
        let result = self.runtime.block_on(self.fs.get_attr(&full_path));

        match result {
            Ok(attr) => {
                let ino = self.get_or_create_inode(&full_path);
                let fuse_attr = if attr.is_dir {
                    Self::make_dir_attr(ino)
                } else {
                    Self::make_file_attr(ino, attr.size)
                };
                reply.entry(&TTL, &fuse_attr, 0);
            }
            Err(_) => {
                reply.error(ENOENT);
            }
        }
    }

    fn getattr(&mut self, _req: &Request, ino: u64, reply: ReplyAttr) {
        let path = match self.inode_path(ino) {
            Some(p) => p,
            None => {
                reply.error(ENOENT);
                return;
            }
        };

        let result = self.runtime.block_on(self.fs.get_attr(&path));

        match result {
            Ok(attr) => {
                let fuse_attr = if attr.is_dir {
                    Self::make_dir_attr(ino)
                } else {
                    Self::make_file_attr(ino, attr.size)
                };
                reply.attr(&TTL, &fuse_attr);
            }
            Err(_) => {
                reply.error(ENOENT);
            }
        }
    }

    fn read(
        &mut self,
        _req: &Request,
        ino: u64,
        _fh: u64,
        offset: i64,
        size: u32,
        _flags: i32,
        _lock_owner: Option<u64>,
        reply: ReplyData,
    ) {
        let path = match self.inode_path(ino) {
            Some(p) => p,
            None => {
                reply.error(ENOENT);
                return;
            }
        };

        debug!(
            "read: ino={} path={} offset={} size={}",
            ino, path, offset, size
        );

        let result = self
            .runtime
            .block_on(self.fs.read_dds(&path, offset as u64, size));

        match result {
            Ok(data) => reply.data(&data),
            Err(e) => {
                warn!("read error for {}: {}", path, e);
                reply.error(libc::EIO);
            }
        }
    }

    fn readdir(
        &mut self,
        _req: &Request,
        ino: u64,
        _fh: u64,
        offset: i64,
        mut reply: ReplyDirectory,
    ) {
        let path = match self.inode_path(ino) {
            Some(p) => p,
            None => {
                reply.error(ENOENT);
                return;
            }
        };

        let entries = match self.fs.list_dir(&path) {
            Ok(e) => e,
            Err(_) => {
                reply.error(ENOENT);
                return;
            }
        };

        for (i, name) in entries.iter().enumerate().skip(offset as usize) {
            let entry_path = if path == "/" {
                format!("/{}", name)
            } else {
                format!("{}/{}", path, name)
            };

            let entry_ino = self.get_or_create_inode(&entry_path);
            let file_type = if name == "." || name == ".." || VIRTUAL_DIRS.contains(&name.as_str())
            {
                FileType::Directory
            } else {
                FileType::RegularFile
            };

            // reply.add returns true when buffer is full
            if reply.add(entry_ino, (i + 1) as i64, file_type, name) {
                break;
            }
        }

        reply.ok();
    }

    fn statfs(&mut self, _req: &Request, _ino: u64, reply: fuser::ReplyStatfs) {
        // Return generous fake stats (matching Python implementation)
        reply.statfs(
            124_699_647, // blocks
            47_602_498,  // bfree
            47_602_498,  // bavail
            1_000_000,   // files
            1_000_000,   // ffree
            4096,        // bsize
            256,         // namelen
            4096,        // frsize
        );
    }
}

/// Mount the AutoOrtho FUSE filesystem.
///
/// This blocks until the filesystem is unmounted (via poison pill or signal).
///
/// # Arguments
/// * `fs` - The DdsFileSystem instance
/// * `mountpoint` - Path to mount at
/// * `runtime` - Tokio runtime handle for async operations
pub fn mount(
    fs: Arc<DdsFileSystem>,
    mountpoint: &Path,
    runtime: tokio::runtime::Handle,
) -> Result<(), Box<dyn std::error::Error>> {
    let fuse_fs = AutoOrthoFuse::new(fs, runtime);

    info!("Mounting AutoOrtho FUSE at {}", mountpoint.display());

    let options = vec![
        MountOption::RO,
        MountOption::FSName("autoortho".to_string()),
        MountOption::AllowOther,
        MountOption::AutoUnmount,
    ];

    fuser::mount2(fuse_fs, mountpoint, &options)?;

    info!("FUSE unmounted from {}", mountpoint.display());
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_well_known_inodes() {
        let provider = Arc::new(MockProvider);
        let fetcher = crate::tiles::fetcher::TileFetcher::new(provider);
        let fs = Arc::new(DdsFileSystem::new(Arc::new(fetcher)));
        let rt = tokio::runtime::Runtime::new().unwrap();
        let fuse = AutoOrthoFuse::new(fs, rt.handle().clone());

        assert_eq!(fuse.get_or_create_inode("/"), ROOT_INO);
        assert_eq!(fuse.get_or_create_inode("/textures"), TEXTURES_INO);
        assert_eq!(fuse.get_or_create_inode("/terrain"), TERRAIN_INO);
    }

    #[test]
    fn test_dynamic_inode_allocation() {
        let provider = Arc::new(MockProvider);
        let fetcher = crate::tiles::fetcher::TileFetcher::new(provider);
        let fs = Arc::new(DdsFileSystem::new(Arc::new(fetcher)));
        let rt = tokio::runtime::Runtime::new().unwrap();
        let fuse = AutoOrthoFuse::new(fs, rt.handle().clone());

        let ino1 = fuse.get_or_create_inode("/textures/100_200_BI16.dds");
        let ino2 = fuse.get_or_create_inode("/textures/300_400_GO2_18.dds");
        assert!(ino1 >= DYNAMIC_INO_START);
        assert_ne!(ino1, ino2);

        // Same path should return same inode
        let ino1b = fuse.get_or_create_inode("/textures/100_200_BI16.dds");
        assert_eq!(ino1, ino1b);
    }

    #[test]
    fn test_inode_path_roundtrip() {
        let provider = Arc::new(MockProvider);
        let fetcher = crate::tiles::fetcher::TileFetcher::new(provider);
        let fs = Arc::new(DdsFileSystem::new(Arc::new(fetcher)));
        let rt = tokio::runtime::Runtime::new().unwrap();
        let fuse = AutoOrthoFuse::new(fs, rt.handle().clone());

        let path = "/textures/100_200_BI16.dds";
        let ino = fuse.get_or_create_inode(path);
        assert_eq!(fuse.inode_path(ino).unwrap(), path);
    }

    use std::future::Future;
    use std::pin::Pin;

    struct MockProvider;

    impl crate::tiles::provider::TileProvider for MockProvider {
        fn fetch(
            &self,
            _row: u32,
            _col: u32,
            _zoom: u32,
        ) -> Pin<
            Box<
                dyn Future<Output = Result<Vec<u8>, crate::tiles::provider::TileProviderError>>
                    + Send
                    + '_,
            >,
        > {
            Box::pin(async { Ok(vec![0xFF, 0xD8, 0xFF, 0xD9]) })
        }

        fn name(&self) -> &str {
            "Mock"
        }
    }
}
