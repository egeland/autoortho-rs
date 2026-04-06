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

    use winfsp::filesystem::{
        DirBuffer, DirInfo, DirMarker, FileInfo, FileSecurity, FileSystemContext, OpenFileInfo,
        VolumeInfo, WideNameInfo,
    };
    use winfsp::host::{FileSystemHost, VolumeParams};

    // Raw NTSTATUS values (avoids direct dependency on `windows` crate)
    const STATUS_OBJECT_NAME_NOT_FOUND: i32 = 0xC0000034_u32 as i32;
    const STATUS_INVALID_HANDLE: i32 = 0xC0000008_u32 as i32;
    const STATUS_UNSUCCESSFUL: i32 = 0xC0000001_u32 as i32;

    const ROOT_INO: u64 = 1;
    const TEXTURES_INO: u64 = 2;
    const TERRAIN_INO: u64 = 3;
    const MARKER_INO: u64 = 4;
    const DYNAMIC_INO_START: u64 = 1000;

    pub struct AutoOrthoWinFsp {
        fs: Arc<DdsFileSystem>,
        runtime: tokio::runtime::Handle,
        path_to_inode: Mutex<HashMap<PathBuf, u64>>,
        next_inode: Mutex<u64>,
        open_files: Mutex<HashMap<u64, PathBuf>>,
        next_file_handle: Mutex<u64>,
        dir_buffer: DirBuffer,
    }

    impl AutoOrthoWinFsp {
        pub fn new(fs: Arc<DdsFileSystem>, runtime: tokio::runtime::Handle) -> Self {
            Self {
                fs,
                runtime,
                path_to_inode: Mutex::new(HashMap::new()),
                next_inode: Mutex::new(DYNAMIC_INO_START),
                open_files: Mutex::new(HashMap::new()),
                next_file_handle: Mutex::new(1),
                dir_buffer: DirBuffer::new(),
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

        fn system_time_to_u64(time: SystemTime) -> u64 {
            // Windows FILETIME: 100-nanosecond intervals since 1601-01-01
            // Unix epoch offset from Windows epoch: 11644473600 seconds
            const EPOCH_DIFF: u64 = 11_644_473_600;
            let duration = time
                .duration_since(SystemTime::UNIX_EPOCH)
                .unwrap_or_default();
            (duration.as_secs() + EPOCH_DIFF) * 10_000_000
                + u64::from(duration.subsec_nanos()) / 100
        }

        fn fill_file_info(file_info: &mut FileInfo, is_dir: bool, size: u64, ino: u64) {
            let now = Self::system_time_to_u64(SystemTime::now());
            file_info.file_attributes = if is_dir { 0x10 } else { 0x80 };
            file_info.file_size = size;
            file_info.allocation_size = (size + 4095) / 4096 * 4096;
            file_info.creation_time = now;
            file_info.last_access_time = now;
            file_info.last_write_time = now;
            file_info.change_time = now;
            file_info.index_number = ino;
            file_info.hard_links = 0;
        }
    }

    impl FileSystemContext for AutoOrthoWinFsp {
        type FileContext = u64;

        fn get_security_by_name(
            &self,
            file_name: &winfsp::U16CStr,
            _security_descriptor: Option<&mut [std::ffi::c_void]>,
            _reparse_point_resolver: impl FnOnce(&winfsp::U16CStr) -> Option<FileSecurity>,
        ) -> winfsp::Result<FileSecurity> {
            let path = file_name.to_string_lossy();
            debug!("get_security_by_name: {:?}", path);

            let path_str = self.path_to_string(&Path::new(&path));
            if is_poison_path(&path_str) {
                info!("Poison pill detected at {}. Shutting down.", path_str);
                return Err(winfsp::FspError::NTSTATUS(STATUS_OBJECT_NAME_NOT_FOUND));
            }

            Ok(FileSecurity {
                attributes: 0x80, // FILE_ATTRIBUTE_NORMAL
                reparse: false,
                sz_security_descriptor: 0,
            })
        }

        fn open(
            &self,
            file_name: &winfsp::U16CStr,
            _create_options: u32,
            _granted_access: u32,
            file_info: &mut OpenFileInfo,
        ) -> winfsp::Result<Self::FileContext> {
            let path = file_name.to_string_lossy();
            debug!("open: {:?}", path);

            let path_str = self.path_to_string(&Path::new(&path));

            if is_poison_path(&path_str) {
                info!("Poison pill detected at {}. Shutting down.", path_str);
                return Err(winfsp::FspError::NTSTATUS(STATUS_OBJECT_NAME_NOT_FOUND));
            }

            let ino = self.path_to_inode(Path::new(&path));

            let attr = self.runtime.block_on(self.fs.get_attr(&path_str));

            match attr {
                Ok(file_attr) => {
                    let mut fh_guard = self.next_file_handle.lock().unwrap();
                    let fh = *fh_guard;
                    *fh_guard += 1;

                    self.open_files
                        .lock()
                        .unwrap()
                        .insert(fh, PathBuf::from(&path));

                    let fi: &mut FileInfo = file_info.as_mut();
                    Self::fill_file_info(fi, file_attr.is_dir, file_attr.size, ino);

                    Ok(fh)
                }
                Err(_) => Err(winfsp::FspError::NTSTATUS(STATUS_OBJECT_NAME_NOT_FOUND)),
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
        ) -> winfsp::Result<()> {
            let path = self.open_files.lock().unwrap().get(context).cloned();
            if let Some(path) = path {
                let path_str = self.path_to_string(&path);
                let ino = self.path_to_inode(&path);

                if let Ok(attr) = self.runtime.block_on(self.fs.get_attr(&path_str)) {
                    Self::fill_file_info(file_info, attr.is_dir, attr.size, ino);
                    return Ok(());
                }
            }
            Err(winfsp::FspError::NTSTATUS(STATUS_OBJECT_NAME_NOT_FOUND))
        }

        fn read(
            &self,
            context: &Self::FileContext,
            buffer: &mut [u8],
            offset: u64,
        ) -> winfsp::Result<u32> {
            let path = self
                .open_files
                .lock()
                .unwrap()
                .get(context)
                .cloned()
                .ok_or(winfsp::FspError::NTSTATUS(STATUS_INVALID_HANDLE))?;
            let path_str = self.path_to_string(&path);

            debug!(
                "read: fh={} offset={} size={}",
                context,
                offset,
                buffer.len()
            );

            let size = buffer.len() as u32;
            match self
                .runtime
                .block_on(self.fs.read_dds(&path_str, offset, size))
            {
                Ok(data) => {
                    let len = data.len().min(buffer.len());
                    buffer[..len].copy_from_slice(&data[..len]);
                    Ok(len as u32)
                }
                Err(e) => {
                    warn!("read error for {:?}: {:?}", path, e);
                    Err(winfsp::FspError::NTSTATUS(STATUS_UNSUCCESSFUL))
                }
            }
        }

        fn read_directory(
            &self,
            _context: &Self::FileContext,
            _pattern: Option<&winfsp::U16CStr>,
            marker: DirMarker,
            buffer: &mut [u8],
        ) -> winfsp::Result<u32> {
            debug!("read_directory");

            if marker.is_none() {
                let lock = self.dir_buffer.acquire(true, None)?;

                let add_entry = |name: &str, is_dir: bool, size: u64| -> winfsp::Result<()> {
                    let mut dir_info = DirInfo::<255>::new();
                    dir_info.set_name(name)?;
                    let fi = dir_info.file_info_mut();
                    fi.file_attributes = if is_dir { 0x10 } else { 0x80 };
                    fi.file_size = size;
                    fi.allocation_size = (size + 4095) / 4096 * 4096;
                    lock.write(&mut dir_info)?;
                    Ok(())
                };

                let _ = add_entry(".", true, 4096);
                let _ = add_entry("..", true, 4096);

                for dir in VIRTUAL_DIRS {
                    let _ = add_entry(dir, true, 4096);
                }
                // lock is dropped here, releasing the buffer
            }

            let bytes_read = self.dir_buffer.read(marker, buffer);
            Ok(bytes_read)
        }

        fn get_volume_info(&self, out_volume_info: &mut VolumeInfo) -> winfsp::Result<()> {
            out_volume_info.total_size = 1_000_000_000_000;
            out_volume_info.free_size = 500_000_000_000;
            out_volume_info.set_volume_label("AutoOrtho");

            Ok(())
        }
    }

    pub fn mount(
        fs: Arc<DdsFileSystem>,
        mountpoint: &Path,
        runtime: tokio::runtime::Handle,
    ) -> Result<(), Box<dyn std::error::Error>> {
        // Initialize WinFSP library (must be called before creating FileSystemHost)
        let _init = winfsp::winfsp_init_or_die();

        let winfsp_fs = AutoOrthoWinFsp::new(fs, runtime);

        info!(
            "Mounting AutoOrtho at {} using winfsp",
            mountpoint.display()
        );

        let mut volume_params = VolumeParams::new();
        volume_params
            .sector_size(4096)
            .sectors_per_allocation_unit(1)
            .volume_serial_number(0x20260328)
            .file_info_timeout(1000)
            .case_sensitive_search(false)
            .case_preserved_names(true)
            .unicode_on_disk(true)
            .persistent_acls(true)
            .post_cleanup_when_modified_only(true)
            .filesystem_name("AutoOrtho");

        let mut host = FileSystemHost::new(volume_params, winfsp_fs)?;
        let mount_str = mountpoint.to_string_lossy().to_string();
        host.mount(&mount_str)?;

        info!("AutoOrtho mounted at {}", mountpoint.display());

        // Block until unmounted (WinFSP handles this internally via the service loop)
        // The mount call returns when the filesystem is unmounted.

        info!("AutoOrtho unmounted from {}", mountpoint.display());
        Ok(())
    }
}
