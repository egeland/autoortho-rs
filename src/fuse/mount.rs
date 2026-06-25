//! FUSE mount implementation using unifuse.
//!
//! unifuse provides a cross-platform async FUSE abstraction:
//! - Linux/macOS: uses rfuse3
//! - Windows: uses dokan via unifuse (requires Dokan runtime)

pub use self::unifuse_impl::mount;

mod unifuse_impl {
    use crate::fuse::filesystem::DdsFileSystem;
    use crate::fuse::{MARKER_FILE, VIRTUAL_DIRS, is_poison_path};
    use tracing::{debug, info, warn};
    use std::collections::HashMap;
    use std::path::{Path, PathBuf};
    use std::sync::{Arc, Mutex};
    use std::time::SystemTime;

    use unifuse::types::*;
    use unifuse::{
        DirEntry, FileAttr, FileHandle, FsError, MountOptions, OpenFlags, StatFs,
        UniFuseFilesystem, UniFuseHost,
    };

    const ROOT_INO: u64 = 1;
    const TEXTURES_INO: u64 = 2;
    const TERRAIN_INO: u64 = 3;
    const MARKER_INO: u64 = 4;
    const DYNAMIC_INO_START: u64 = 1000;

    pub struct AutoOrthoUniFuse {
        fs: Arc<DdsFileSystem>,
        path_to_inode: Mutex<HashMap<PathBuf, u64>>,
        next_inode: Mutex<u64>,
        open_files: Mutex<HashMap<FileHandle, PathBuf>>,
        next_file_handle: Mutex<FileHandle>,
    }

    impl AutoOrthoUniFuse {
        pub fn new(fs: Arc<DdsFileSystem>) -> Self {
            Self {
                fs,
                path_to_inode: Mutex::new(HashMap::new()),
                next_inode: Mutex::new(DYNAMIC_INO_START),
                open_files: Mutex::new(HashMap::new()),
                next_file_handle: Mutex::new(FileHandle(1)),
            }
        }

        fn path_to_inode(&self, path: &Path) -> u64 {
            if path == Path::new("/") {
                return ROOT_INO;
            }
            if path == Path::new("/textures") {
                return TEXTURES_INO;
            }
            if path == Path::new("/terrain") {
                return TERRAIN_INO;
            }

            let path_str = path.to_string_lossy().to_string();
            if path_str.ends_with(MARKER_FILE) {
                return MARKER_INO;
            }

            let mut p2i = self.path_to_inode.lock().unwrap();
            if let Some(&ino) = p2i.get(path) {
                return ino;
            }

            let mut next = self.next_inode.lock().unwrap();
            let ino = *next;
            *next += 1;

            p2i.insert(path.to_path_buf(), ino);

            ino
        }

        fn make_file_attr(size: u64) -> FileAttr {
            FileAttr {
                size,
                blocks: size.div_ceil(512),
                atime: SystemTime::now(),
                mtime: SystemTime::now(),
                ctime: SystemTime::now(),
                crtime: SystemTime::now(),
                kind: FileType::RegularFile,
                perm: 0o444,
                nlink: 1,
                uid: 0,
                gid: 0,
                rdev: 0,
                flags: 0,
            }
        }

        fn make_dir_attr() -> FileAttr {
            FileAttr {
                size: 4096,
                blocks: 1,
                atime: SystemTime::now(),
                mtime: SystemTime::now(),
                ctime: SystemTime::now(),
                crtime: SystemTime::now(),
                kind: FileType::Directory,
                perm: 0o555,
                nlink: 2,
                uid: 0,
                gid: 0,
                rdev: 0,
                flags: 0,
            }
        }
    }

    impl UniFuseFilesystem for AutoOrthoUniFuse {
        async fn getattr(&self, path: &Path) -> Result<FileAttr, FsError> {
            debug!("getattr: {:?}", path);

            let path_str = path.to_string_lossy();
            if is_poison_path(&path_str) {
                info!("Poison pill detected at {:?}. Shutting down.", path);
                return Err(FsError::NotFound);
            }

            let _ino = self.path_to_inode(path);

            match self.fs.get_attr(&path_str).await {
                Ok(attr) => {
                    let file_attr = if attr.is_dir {
                        Self::make_dir_attr()
                    } else {
                        Self::make_file_attr(attr.size)
                    };
                    Ok(file_attr)
                }
                Err(_) => Err(FsError::NotFound),
            }
        }

        async fn lookup(&self, parent: &Path, name: &std::ffi::OsStr) -> Result<FileAttr, FsError> {
            let name_str = name.to_string_lossy();
            let full_path = if parent == Path::new("/") {
                PathBuf::from(format!("/{}", name_str))
            } else {
                parent.join(name_str.as_ref())
            };

            debug!("lookup: {:?} / {:?} -> {:?}", parent, name, full_path);

            let full_path_str = full_path.to_string_lossy();

            if is_poison_path(&full_path_str) {
                info!("Poison pill detected at {}. Shutting down.", full_path_str);
                return Err(FsError::NotFound);
            }

            let _ino = self.path_to_inode(&full_path);

            match self.fs.get_attr(&full_path_str).await {
                Ok(attr) => {
                    let file_attr = if attr.is_dir {
                        Self::make_dir_attr()
                    } else {
                        Self::make_file_attr(attr.size)
                    };
                    Ok(file_attr)
                }
                Err(_) => Err(FsError::NotFound),
            }
        }

        async fn open(&self, path: &Path, _flags: OpenFlags) -> Result<FileHandle, FsError> {
            debug!("open: {:?}", path);

            let path_str = path.to_string_lossy();

            if is_poison_path(&path_str) {
                info!("Poison pill detected at {}. Shutting down.", path_str);
                return Err(FsError::NotFound);
            }

            match self.fs.get_attr(&path_str).await {
                Ok(_) => {
                    let mut fh_guard = self.next_file_handle.lock().unwrap();
                    let fh = *fh_guard;
                    *fh_guard = FileHandle(fh.0 + 1);

                    self.open_files
                        .lock()
                        .unwrap()
                        .insert(fh, path.to_path_buf());
                    Ok(fh)
                }
                Err(_) => Err(FsError::NotFound),
            }
        }

        async fn read(
            &self,
            path: &Path,
            _fh: FileHandle,
            offset: u64,
            size: u32,
        ) -> Result<Vec<u8>, FsError> {
            debug!("read: {:?} offset={} size={}", path, offset, size);

            let path_str = path.to_string_lossy();

            match self.fs.read_dds(&path_str, offset, size).await {
                Ok(data) => Ok(data),
                Err(e) => {
                    warn!("read error for {:?}: {:?}", path, e);
                    Err(FsError::Io(std::io::Error::other(e)))
                }
            }
        }

        async fn release(&self, path: &Path, fh: FileHandle) -> Result<(), FsError> {
            debug!("release: {:?} fh={:?}", path, fh);
            self.open_files.lock().unwrap().remove(&fh);
            Ok(())
        }

        async fn readdir(&self, path: &Path) -> Result<Vec<DirEntry>, FsError> {
            debug!("readdir: {:?}", path);

            let path_str = path.to_string_lossy();
            let mut entries = Vec::new();

            entries.push(DirEntry {
                name: ".".to_string(),
                kind: FileType::Directory,
            });

            entries.push(DirEntry {
                name: "..".to_string(),
                kind: FileType::Directory,
            });

            // Use DdsFileSystem.list_dir() for all directory listings
            // This gives us virtual dirs + pass-through entries from root
            match self.fs.list_dir(&path_str) {
                Ok(fs_entries) => {
                    for entry_name in fs_entries {
                        if entry_name == "." || entry_name == ".." {
                            continue;
                        }
                        let is_dir = VIRTUAL_DIRS.contains(&entry_name.as_str())
                            || self.fs.is_dir_in_root(&path_str, &entry_name);
                        entries.push(DirEntry {
                            name: entry_name,
                            kind: if is_dir {
                                FileType::Directory
                            } else {
                                FileType::RegularFile
                            },
                        });
                    }
                }
                Err(_) => return Err(FsError::NotFound),
            }

            Ok(entries)
        }

        async fn statfs(&self, _path: &Path) -> Result<StatFs, FsError> {
            Ok(StatFs {
                blocks: 1_000_000,
                bfree: 500_000,
                bavail: 500_000,
                files: 1_000_000,
                ffree: 500_000,
                bsize: 4096,
                namelen: 255,
            })
        }
    }

    pub fn mount(
        fs: Arc<DdsFileSystem>,
        mountpoint: &Path,
        runtime: tokio::runtime::Handle,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let unifuse_fs = AutoOrthoUniFuse::new(fs);

        info!(
            "Mounting AutoOrtho at {} using unifuse",
            mountpoint.display()
        );

        runtime.block_on(async {
            let host = UniFuseHost::new(unifuse_fs);
            host.mount(mountpoint, &MountOptions::default())
                .await
                .map_err(|e| Box::new(e) as Box<dyn std::error::Error>)
        })?;

        info!("AutoOrtho unmounted from {}", mountpoint.display());
        Ok(())
    }
}
