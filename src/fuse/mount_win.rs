//! Windows FUSE mount implementation using Dokan.
//!
//! Dokan provides Windows user-mode file system functionality, similar to FUSE on Unix.

pub use self::dokan_impl::mount;

mod dokan_impl {
    use crate::fuse::filesystem::DdsFileSystem;
    use crate::fuse::{MARKER_FILE, VIRTUAL_DIRS, is_poison_path};
    use log::{debug, info, warn};
    use std::collections::HashMap;
    use std::path::{Path, PathBuf};
    use std::sync::{Arc, Mutex};
    use std::time::SystemTime;

    use dokan::filesystem::{
        FileContext, FileInfo, FileSystem, VolumeInfo,
    };
    use dokan::mount::{MountOptions, mount as dokan_mount};

    const ROOT_INO: u64 = 1;
    const TEXTURES_INO: u64 = 2;
    const TERRAIN_INO: u64 = 3;
    const MARKER_INO: u64 = 4;
    const DYNAMIC_INO_START: u64 = 1000;

    pub struct AutoOrthoDokan {
        fs: Arc<DdsFileSystem>,
        runtime: tokio::runtime::Handle,
        path_to_inode: Mutex<HashMap<PathBuf, u64>>,
        next_inode: Mutex<u64>,
        open_files: Mutex<HashMap<u64, PathBuf>>,
        next_file_handle: Mutex<u64>,
    }

    impl AutoOrthoDokan {
        pub fn new(fs: Arc<DdsFileSystem>, runtime: tokio::runtime::Handle) -> Self {
            Self {
                fs,
                runtime,
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

        fn system_time_to_dokan_time(time: SystemTime) -> u64 {
            // Dokan uses Windows FILETIME: 100-nanosecond intervals since 1601-01-01
            const EPOCH_DIFF: u64 = 11_644_473_600;
            let duration = time
                .duration_since(SystemTime::UNIX_EPOCH)
                .unwrap_or_default();
            (duration.as_secs() + EPOCH_DIFF) * 10_000_000
                + u64::from(duration.subsec_nanos()) / 100
        }

        fn fill_file_info(file_info: &mut FileInfo, is_dir: bool, size: u64, ino: u64) {
            let now = Self::system_time_to_dokan_time(SystemTime::now());
            file_info.attributes = if is_dir { 0x10 } else { 0x80 };
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

    impl FileSystem for AutoOrthoDokan {
        type FileContext = u64;

        fn open(
            &self,
            path: &Path,
            _flags: u32,
            file_info: &mut FileInfo,
        ) -> dokan::Result<Self::FileContext> {
            let path_str = self.path_to_string(path);
            debug!("dokan open: {:?}", path_str);

            if is_poison_path(&path_str) {
                info!("Poison pill detected at {}. Shutting down.", path_str);
                return Err(dokan::Error::FileNotFound);
            }

            let ino = self.path_to_inode(path);
            let attr = self.runtime.block_on(self.fs.get_attr(&path_str));

            match attr {
                Ok(file_attr) => {
                    let mut fh_guard = self.next_file_handle.lock().unwrap();
                    let fh = *fh_guard;
                    *fh_guard += 1;

                    self.open_files
                        .lock()
                        .unwrap()
                        .insert(fh, path.to_path_buf());

                    Self::fill_file_info(file_info, file_attr.is_dir, file_attr.size, ino);
                    Ok(fh)
                }
                Err(_) => Err(dokan::Error::FileNotFound),
            }
        }

        fn close(&self, context: Self::FileContext) {
            debug!("dokan close: fh={}", context);
            self.open_files.lock().unwrap().remove(&context);
        }

        fn read(
            &self,
            context: &Self::FileContext,
            buffer: &mut [u8],
            offset: u64,
        ) -> dokan::Result<u32> {
            let path = self
                .open_files
                .lock()
                .unwrap()
                .get(context)
                .cloned()
                .ok_or(dokan::Error::InvalidHandle)?;
            let path_str = self.path_to_string(&path);

            debug!(
                "dokan read: fh={} offset={} size={}",
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
                    warn!("dokan read error for {:?}: {:?}", path, e);
                    Err(dokan::Error::Unsuccessful)
                }
            }
        }

        fn read_directory(
            &self,
            context: &Self::FileContext,
            _pattern: Option<&str>,
            buffer: &mut dokan::DirBuffer,
        ) -> dokan::Result<()> {
            debug!("dokan read_directory: context={}", context);

            let dir_path = self
                .open_files
                .lock()
                .unwrap()
                .get(context)
                .cloned()
                .unwrap_or_else(|| PathBuf::from("/"));
            let dir_path_str = dir_path.to_string_lossy().replace('\\', "/");
            let is_root = dir_path_str == "/" || dir_path_str.is_empty();
            let is_textures = dir_path_str.ends_with("/textures");
            let is_terrain = dir_path_str.ends_with("/terrain");

            buffer.add_entry(".", 0x10, 4096)?;
            buffer.add_entry("..", 0x10, 4096)?;

            if is_root {
                for dir in VIRTUAL_DIRS {
                    buffer.add_entry(dir, 0x10, 4096)?;
                }
            } else if is_textures || is_terrain {
                buffer.add_entry(MARKER_FILE, 0x80, 0)?;
            }

            Ok(())
        }

        fn get_file_info(
            &self,
            context: &Self::FileContext,
            file_info: &mut FileInfo,
        ) -> dokan::Result<()> {
            let path = self.open_files.lock().unwrap().get(context).cloned();
            if let Some(path) = path {
                let path_str = self.path_to_string(&path);
                let ino = self.path_to_inode(&path);

                if let Ok(attr) = self.runtime.block_on(self.fs.get_attr(&path_str)) {
                    Self::fill_file_info(file_info, attr.is_dir, attr.size, ino);
                    return Ok(());
                }
            }
            Err(dokan::Error::FileNotFound)
        }

        fn get_volume_info(&self, volume_info: &mut VolumeInfo) -> dokan::Result<()> {
            volume_info.total_size = 1_000_000_000_000;
            volume_info.free_size = 500_000_000_000;
            volume_info.volume_label = "AutoOrtho".to_string();
            Ok(())
        }
    }

    pub fn mount(
        fs: Arc<DdsFileSystem>,
        mountpoint: &Path,
        runtime: tokio::runtime::Handle,
    ) -> Result<(), Box<dyn std::error::Error>> {
        // Initialize Dokan library
        dokan::DokanInit()?;

        let dokan_fs = AutoOrthoDokan::new(fs, runtime);
        let mount_str = mountpoint.to_string_lossy().to_string();
        info!("Mounting AutoOrtho at {} using Dokan", mount_str);

        // Cleanup stale mount
        let _ = crate::fuse::platform::cleanup_mount(mountpoint);
        std::thread::sleep(std::time::Duration::from_secs(1));

        let options = MountOptions::new()
            .mount_point(&mount_str)
            .volume_label("AutoOrtho")
            .sector_size(4096);

        let result = dokan_mount(&options, dokan_fs);
        match result {
            Ok(_) => {
                info!("AutoOrtho mounted at {}", mount_str);
                Ok(())
            }
            Err(e) => {
                // Check for mount collision
                if e.to_string().contains("already exists") {
                    warn!("Mount collision, retrying...");
                    let _ = crate::fuse::platform::cleanup_mount(mountpoint);
                    std::thread::sleep(std::time::Duration::from_secs(2));
                    dokan_mount(&options, dokan_fs)?;
                    Ok(())
                } else {
                    Err(Box::new(e))
                }
            }
        }
    }
}
