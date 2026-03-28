//! Windows FUSE mount implementation using winfsp.
//!
//! winfsp provides Windows File System Proxy functionality, similar to FUSE on Unix.

pub use self::winfsp_impl::mount;

mod winfsp_impl {
    use crate::fuse::filesystem::DdsFileSystem;
    use crate::fuse::{MARKER_FILE, VIRTUAL_DIRS, is_poison_path};
    use log::{debug, info, warn};
    use std::collections::HashMap;
    use std::path::{Path, PathBuf};
    use std::sync::{Arc, Mutex};
    use std::time::SystemTime;

    use winfsp::WCHAR;
    use winfsp::filesystem::{DirInfo, FileInfo, FileSecurity, FileType, OpenFileInfo, VolInfo};
    use winfsp::host::{FileSystemHost, MountOptions, VolumeParams};

    const ROOT_INO: u64 = 1;
    const TEXTURES_INO: u64 = 2;
    const TERRAIN_INO: u64 = 3;
    const MARKER_INO: u64 = 4;
    const DYNAMIC_INO_START: u64 = 1000;

    pub struct AutoOrthoWinFsp {
        fs: Arc<DdsFileSystem>,
        path_to_inode: Mutex<HashMap<PathBuf, u64>>,
        next_inode: Mutex<u64>,
        open_files: Mutex<HashMap<u64, PathBuf>>,
        next_file_handle: Mutex<u64>,
    }

    impl AutoOrthoWinFsp {
        pub fn new(fs: Arc<DdsFileSystem>) -> Self {
            Self {
                fs,
                path_to_inode: Mutex::new(HashMap::new()),
                next_inode: Mutex::new(DYNAMIC_INO_START),
                open_files: Mutex::new(HashMap::new()),
                next_file_handle: Mutex::new(1),
            }
        }

        fn path_to_inode(&self, path: &Path) -> u64 {
            if path == Path::new("/") || path.to_string_lossy() == "\\" {
                return ROOT_INO;
            }
            if path == Path::new("/textures") || path.to_string_lossy() == "\\textures" {
                return TEXTURES_INO;
            }
            if path == Path::new("/terrain") || path.to_string_lossy() == "\\terrain" {
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

        fn path_to_string(&self, path: &Path) -> String {
            path.to_string_lossy().replace('\\', "/")
        }
    }

    impl winfsp::filesystem::FileSystemContext for AutoOrthoWinFsp {
        type FileContext = u64;

        fn get_security_by_name(
            &self,
            file_name: &winfsp::U16CStr,
            _security_descriptor: Option<&mut [std::ffi::c_void]>,
            _reparse_point_resolver: impl FnOnce(&winfsp::U16CStr) -> Option<FileSecurity>,
        ) -> Result<FileSecurity, winfsp::Result<()>> {
            let path = file_name.to_string_lossy();
            debug!("get_security_by_name: {:?}", path);

            let path_str = self.path_to_string(&Path::new(&path));
            if is_poison_path(&path_str) {
                info!("Poison pill detected at {}. Shutting down.", path_str);
                return Err(Err(()));
            }

            Ok(FileSecurity::from_access(0xFFFFFFFF).unwrap_or_default())
        }

        fn open(
            &self,
            file_name: &winfsp::U16CStr,
            _create_options: u32,
            _granted_access: u32,
            file_info: &mut OpenFileInfo,
        ) -> Result<Self::FileContext, winfsp::Result<()>> {
            let path = file_name.to_string_lossy();
            debug!("open: {:?}", path);

            let path_str = self.path_to_string(&Path::new(&path));

            if is_poison_path(&path_str) {
                info!("Poison pill detected at {}. Shutting down.", path_str);
                return Err(Err(()));
            }

            let ino = self.path_to_inode(Path::new(&path));

            let attr = tokio::runtime::Handle::current().block_on(self.fs.get_attr(&path_str));

            match attr {
                Ok(file_attr) => {
                    let mut fh_guard = self.next_file_handle.lock().unwrap();
                    let fh = *fh_guard;
                    *fh_guard += 1;

                    self.open_files
                        .lock()
                        .unwrap()
                        .insert(fh, PathBuf::from(&path));

                    file_info
                        .set_file_size(file_attr.size)
                        .set_allocation_size((file_attr.size + 4095) / 4096 * 4096)
                        .set_file_attributes(if file_attr.is_dir { 0x10 } else { 0x80 })
                        .set_creation_time(SystemTime::now())
                        .set_last_access_time(SystemTime::now())
                        .set_last_write_time(SystemTime::now())
                        .set_change_time(SystemTime::now())
                        .set_index_number(ino)
                        .set_hard_links(1);

                    Ok(fh)
                }
                Err(_) => Err(Err(())),
            }
        }

        fn close(&self, context: Self::FileContext) {
            debug!("close: fh={}", context);
            self.open_files.lock().unwrap().remove(&context);
        }

        fn get_file_info(
            &self,
            context: &Self::FileContext,
            file_info: &mut FileInfo,
        ) -> Result<(), winfsp::Result<()>> {
            let path = self.open_files.lock().unwrap().get(context).cloned();
            if let Some(path) = path {
                let path_str = self.path_to_string(&path);
                let ino = self.path_to_inode(&path);

                if let Ok(attr) =
                    tokio::runtime::Handle::current().block_on(self.fs.get_attr(&path_str))
                {
                    file_info
                        .set_file_size(attr.size)
                        .set_allocation_size((attr.size + 4095) / 4096 * 4096)
                        .set_file_attributes(if attr.is_dir { 0x10 } else { 0x80 })
                        .set_creation_time(SystemTime::now())
                        .set_last_access_time(SystemTime::now())
                        .set_last_write_time(SystemTime::now())
                        .set_change_time(SystemTime::now())
                        .set_index_number(ino)
                        .set_hard_links(1);

                    return Ok(());
                }
            }
            Err(Err(()))
        }

        fn read(
            &self,
            context: &Self::FileContext,
            buffer: &mut [u8],
            offset: u64,
        ) -> Result<u32, winfsp::Result<()>> {
            let path = self
                .open_files
                .lock()
                .unwrap()
                .get(context)
                .cloned()
                .ok_or(Err(()))?;
            let path_str = self.path_to_string(&path);

            debug!(
                "read: fh={} offset={} size={}",
                context,
                offset,
                buffer.len()
            );

            let size = buffer.len() as u32;
            match tokio::runtime::Handle::current()
                .block_on(self.fs.read_dds(&path_str, offset, size))
            {
                Ok(data) => {
                    let len = data.len().min(buffer.len());
                    buffer[..len].copy_from_slice(&data[..len]);
                    Ok(len as u32)
                }
                Err(e) => {
                    warn!("read error for {:?}: {:?}", path, e);
                    Err(Err(()))
                }
            }
        }

        fn read_directory(
            &self,
            _context: &Self::FileContext,
            _pattern: Option<&winfsp::U16CStr>,
            _marker: winfsp::filesystem::DirMarker,
            buffer: &mut [u8],
        ) -> Result<u32, winfsp::Result<()>> {
            debug!("read_directory");

            let mut cursor = 0u32;
            let mut add_entry = |name: &str, is_dir: bool, size: u64| -> bool {
                let dir_info =
                    DirInfo::create(name, 0, 0, size, 0, 0, 0, 0, 0, 0).map_err(|_| Err(()))?;
                let entry_size = dir_info.size() as u32;
                if cursor + entry_size > buffer.len() as u32 {
                    return false;
                }
                dir_info
                    .write_to_buffer(&mut buffer[cursor as usize..], &mut cursor)
                    .map_err(|_| Err(()))?;
                true
            };

            add_entry(".", true, 4096)?;
            add_entry("..", true, 4096)?;

            for dir in VIRTUAL_DIRS {
                if !add_entry(dir, true, 4096) {
                    return Err(Ok(cursor));
                }
            }

            Ok(cursor)
        }

        fn get_volume_info(&self, out_volume_info: &mut VolInfo) -> Result<(), winfsp::Result<()>> {
            out_volume_info
                .set_total_size(1_000_000_000_000)
                .set_free_size(500_000_000_000)
                .set_volume_label("AutoOrtho");

            Ok(())
        }
    }

    pub fn mount(
        fs: Arc<DdsFileSystem>,
        mountpoint: &Path,
        _runtime: tokio::runtime::Handle,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let winfsp_fs = AutoOrthoWinFsp::new(fs);

        info!(
            "Mounting AutoOrtho at {} using winfsp",
            mountpoint.display()
        );

        let mut volume_params = VolumeParams::new();
        volume_params
            .set_sector_size(4096)
            .set_sectors_per_allocation_unit(1)
            .set_volume_creation_time(SystemTime::now())
            .set_volume_serial_number(0x20260328)
            .set_file_info_timeout(1000)
            .set_case_sensitive_search(false)
            .set_case_preserved_names(true)
            .set_unicode_on_disk(true)
            .set_persistent_acls(true)
            .set_post_cleanup_when_modified_only(true)
            .set_file_system_name("AutoOrtho");

        let mut mount_options = MountOptions::new();
        mount_options.set_volfs(false);

        let mut host = FileSystemHost::new(winfsp_fs, volume_params);
        let mount_str = mountpoint.to_string_lossy();
        host.mount(&mount_str, &mount_options)?;

        info!("AutoOrtho unmounted from {}", mountpoint.display());
        Ok(())
    }
}
