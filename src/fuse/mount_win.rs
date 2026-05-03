//! Windows FUSE mount implementation using Dokan.
//!
//! Dokan provides Windows user-mode file system functionality, similar to FUSE on Unix.
//! This implementation uses the updated dokan-rust API (GitHub version).

pub use self::dokan_impl::mount;

mod dokan_impl {
    use crate::fuse::filesystem::DdsFileSystem;
    use crate::fuse::{MARKER_FILE, VIRTUAL_DIRS, is_poison_path};
    use dokan::{
        FileSystemMounter, MountFlags, MountOptions, OperationResult,
        data::{CreateFileInfo, DiskSpaceInfo, FileInfo, FindData, VolumeInfo},
        file_system_handler::FileSystemHandler,
        init, shutdown,
    };
    use log::{debug, error, info, warn};
    use std::collections::HashMap;
    use std::path::{Path, PathBuf};
    use std::sync::{Arc, Mutex, RwLock};
    use std::time::SystemTime;
    use widestring::{U16CString, U16String};

    const ROOT_INO: u64 = 1;
    const TEXTURES_INO: u64 = 2;
    const TERRAIN_INO: u64 = 3;
    const MARKER_INO: u64 = 4;
    const DYNAMIC_INO_START: u64 = 1000;

    /// Context type for open files
    #[derive(Debug)]
    struct FileContext {
        path: PathBuf,
        inode: u64,
    }

    pub struct AutoOrthoHandler {
        fs: Arc<DdsFileSystem>,
        runtime: tokio::runtime::Handle,
        path_to_inode: RwLock<HashMap<PathBuf, u64>>,
        next_inode: Mutex<u64>,
        open_files: RwLock<HashMap<u64, PathBuf>>,
        next_file_handle: Mutex<u64>,
    }

    impl AutoOrthoHandler {
        pub fn new(fs: Arc<DdsFileSystem>, runtime: tokio::runtime::Handle) -> Self {
            Self {
                fs,
                runtime,
                path_to_inode: RwLock::new(HashMap::new()),
                next_inode: Mutex::new(DYNAMIC_INO_START),
                open_files: RwLock::new(HashMap::new()),
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

            let mut p2i = self.path_to_inode.write().unwrap();
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

        fn system_time_to_file_time(time: SystemTime) -> u64 {
            // Windows FILETIME: 100-nanosecond intervals since 1601-01-01
            const EPOCH_DIFF: u64 = 11_644_473_600;
            let duration = time
                .duration_since(SystemTime::UNIX_EPOCH)
                .unwrap_or_default();
            (duration.as_secs() + EPOCH_DIFF) * 10_000_000
                + u64::from(duration.subsec_nanos()) / 100
        }
    }

    impl<'c, 'h: 'c> FileSystemHandler<'c, 'h> for AutoOrthoHandler
    where
        'h: 'c,
    {
        type Context = FileContext;

        fn create_file(
            &'h self,
            file_name: &widestring::U16CStr,
            _security_context: &dokan_sys::DOKAN_IO_SECURITY_CONTEXT,
            _desired_access: u32,
            _file_attributes: u32,
            _share_access: u32,
            _create_disposition: u32,
            _create_options: u32,
            _info: &mut dokan::OperationInfo<'c, 'h, Self>,
        ) -> OperationResult<CreateFileInfo<Self::Context>> {
            let path_str = file_name.to_string_lossy();
            let path = Path::new(path_str.as_ref());
            let clean_path = Path::new(&self.path_to_string(path));

            debug!("dokan create_file: {:?}", path_str);

            if is_poison_path(&path_str) {
                info!("Poison pill detected at {}. Shutting down.", path_str);
                return Err(dokan_sys::STATUS_FILE_IS_A_DIRECTORY);
            }

            let ino = self.path_to_inode(clean_path);
            let attr = self.runtime.block_on(self.fs.get_attr(&path_str));

            match attr {
                Ok(file_attr) => {
                    let mut next_fh = self.next_file_handle.lock().unwrap();
                    let fh = *next_fh;
                    *next_fh += 1;

                    self.open_files
                        .write()
                        .unwrap()
                        .insert(fh, clean_path.to_path_buf());

                    let context = FileContext {
                        path: clean_path.to_path_buf(),
                        inode: ino,
                    };

                    let now = SystemTime::now();
                    let file_info = FileInfo {
                        attributes: if file_attr.is_dir {
                            winapi::um::winnt::FILE_ATTRIBUTE_DIRECTORY
                        } else {
                            winapi::um::winnt::FILE_ATTRIBUTE_NORMAL
                        },
                        creation_time: Self::system_time_to_file_time(now),
                        last_access_time: Self::system_time_to_file_time(now),
                        last_write_time: Self::system_time_to_file_time(now),
                        change_time: Self::system_time_to_file_time(now),
                        file_size: file_attr.size,
                        allocation_size: if file_attr.is_dir {
                            0
                        } else {
                            (file_attr.size + 4095) / 4096 * 4096
                        },
                        index_number: ino,
                    };

                    Ok(CreateFileInfo {
                        context,
                        file_info,
                        is_dir: file_attr.is_dir,
                    })
                }
                Err(_) => Err(dokan_sys::STATUS_OBJECT_NAME_NOT_FOUND),
            }
        }

        fn cleanup(
            &'h self,
            _file_name: &widestring::U16CStr,
            _info: &dokan::OperationInfo<'c, 'h, Self>,
            _context: &'c Self::Context,
        ) {
            // Nothing special needed for cleanup
        }

        fn close_file(
            &'h self,
            _file_name: &widestring::U16CStr,
            _info: &dokan::OperationInfo<'c, 'h, Self>,
            context: &'c Self::Context,
        ) {
            debug!("dokan close_file: {:?}", context.path);
            self.open_files.write().unwrap().remove(&context.inode);
        }

        fn read_file(
            &'h self,
            _file_name: &widestring::U16CStr,
            offset: i64,
            buffer: &mut [u8],
            _info: &dokan::OperationInfo<'c, 'h, Self>,
            context: &'c Self::Context,
        ) -> OperationResult<u32> {
            let path_str = self.path_to_string(&context.path);
            let offset_u64 = offset as u64;

            debug!(
                "dokan read_file: path={} offset={} size={}",
                path_str,
                offset,
                buffer.len()
            );

            let size = buffer.len() as u32;
            match self
                .runtime
                .block_on(self.fs.read_dds(&path_str, offset_u64, size))
            {
                Ok(data) => {
                    let len = data.len().min(buffer.len());
                    buffer[..len].copy_from_slice(&data[..len]);
                    Ok(len as u32)
                }
                Err(e) => {
                    warn!("dokan read_file error for {:?}: {:?}", context.path, e);
                    Err(dokan_sys::STATUS_UNSUCCESSFUL)
                }
            }
        }

        fn find_files(
            &'h self,
            _file_name: &widestring::U16CStr,
            mut fill_find_data: impl FnMut(&FindData) -> dokan::FillDataResult,
            _info: &dokan::OperationInfo<'c, 'h, Self>,
            context: &'c Self::Context,
        ) -> OperationResult<()> {
            let dir_path = &context.path;
            let dir_path_str = dir_path.to_string_lossy().replace('\\', "/");
            let is_root = dir_path_str == "/" || dir_path_str.is_empty();
            let is_textures = dir_path_str.ends_with("/textures");
            let is_terrain = dir_path_str.ends_with("/terrain");

            let now = SystemTime::now();
            let file_time = Self::system_time_to_file_time(now);

            // Add . and ..
            let dot = FindData {
                file_name: U16String::from_str(".").unwrap(),
                attributes: winapi::um::winnt::FILE_ATTRIBUTE_DIRECTORY,
                creation_time: file_time,
                last_access_time: file_time,
                last_write_time: file_time,
                file_size: 0,
                allocation_size: 0,
            };
            if fill_find_data(&dot).is_err() {
                return Err(dokan_sys::STATUS_UNSUCCESSFUL);
            }

            let dotdot = FindData {
                file_name: U16String::from_str("..").unwrap(),
                attributes: winapi::um::winnt::FILE_ATTRIBUTE_DIRECTORY,
                creation_time: file_time,
                last_access_time: file_time,
                last_write_time: file_time,
                file_size: 0,
                allocation_size: 0,
            };
            if fill_find_data(&dotdot).is_err() {
                return Err(dokan_sys::STATUS_UNSUCCESSFUL);
            }

            if is_root {
                for dir in VIRTUAL_DIRS {
                    let data = FindData {
                        file_name: U16String::from_str(dir).unwrap(),
                        attributes: winapi::um::winnt::FILE_ATTRIBUTE_DIRECTORY,
                        creation_time: file_time,
                        last_access_time: file_time,
                        last_write_time: file_time,
                        file_size: 0,
                        allocation_size: 4096,
                    };
                    if fill_find_data(&data).is_err() {
                        return Err(dokan_sys::STATUS_UNSUCCESSFUL);
                    }
                }
            } else if is_textures || is_terrain {
                let data = FindData {
                    file_name: U16String::from_str(MARKER_FILE).unwrap(),
                    attributes: winapi::um::winnt::FILE_ATTRIBUTE_NORMAL,
                    creation_time: file_time,
                    last_access_time: file_time,
                    last_write_time: file_time,
                    file_size: 0,
                    allocation_size: 0,
                };
                if fill_find_data(&data).is_err() {
                    return Err(dokan_sys::STATUS_UNSUCCESSFUL);
                }
            }

            Ok(())
        }

        fn get_file_information(
            &'h self,
            _file_name: &widestring::U16CStr,
            _info: &dokan::OperationInfo<'c, 'h, Self>,
            context: &'c Self::Context,
        ) -> OperationResult<FileInfo> {
            let path_str = self.path_to_string(&context.path);

            if let Ok(attr) = self.runtime.block_on(self.fs.get_attr(&path_str)) {
                let now = SystemTime::now();
                let file_time = Self::system_time_to_file_time(now);

                return Ok(FileInfo {
                    attributes: if attr.is_dir {
                        winapi::um::winnt::FILE_ATTRIBUTE_DIRECTORY
                    } else {
                        winapi::um::winnt::FILE_ATTRIBUTE_NORMAL
                    },
                    creation_time: file_time,
                    last_access_time: file_time,
                    last_write_time: file_time,
                    change_time: file_time,
                    file_size: attr.size,
                    allocation_size: if attr.is_dir {
                        0
                    } else {
                        (attr.size + 4095) / 4096 * 4096
                    },
                    index_number: context.inode,
                });
            }

            Err(dokan_sys::STATUS_OBJECT_NAME_NOT_FOUND)
        }

        fn get_volume_information(
            &'h self,
            _info: &dokan::OperationInfo<'c, 'h, Self>,
        ) -> OperationResult<VolumeInfo> {
            Ok(VolumeInfo {
                volume_label: U16String::from_str("AutoOrtho").unwrap(),
                file_system_name: U16String::from_str("Dokan").unwrap(),
                serial_number: 0x12345678,
                max_component_length: 256,
                characteristics: 0,
            })
        }

        fn get_disk_free_space(
            &'h self,
            _info: &dokan::OperationInfo<'c, 'h, Self>,
        ) -> OperationResult<DiskSpaceInfo> {
            Ok(DiskSpaceInfo {
                total_size: 1_000_000_000_000,
                free_size: 500_000_000_000,
                available_size: 500_000_000_000,
            })
        }
    }

    pub fn mount(
        fs: Arc<DdsFileSystem>,
        mountpoint: &Path,
        runtime: tokio::runtime::Handle,
    ) -> Result<(), Box<dyn std::error::Error>> {
        // Initialize Dokan library
        init();

        let handler = AutoOrthoHandler::new(fs, runtime);
        let mount_str = mountpoint.to_string_lossy().to_string();
        let mount_cstr = U16CString::from_str(&mount_str)?;

        info!("Mounting AutoOrtho at {} using Dokan", mount_str);

        // Cleanup stale mount
        let _ = crate::fuse::platform::cleanup_mount(mountpoint);
        std::thread::sleep(std::time::Duration::from_secs(1));

        let options = MountOptions {
            flags: MountFlags::empty(),
            ..Default::default()
        };

        let mut mounter = FileSystemMounter::new(&handler, &mount_cstr, &options);

        // This blocks until unmounted
        match mounter.mount() {
            Ok(_filesystem) => {
                info!("AutoOrtho mounted at {}", mount_str);
                shutdown();
                Ok(())
            }
            Err(e) => {
                error!("Failed to mount: {:?}", e);
                // Check for mount collision
                if format!("{:?}", e).contains("mount") {
                    warn!("Mount collision, retrying...");
                    let _ = crate::fuse::platform::cleanup_mount(mountpoint);
                    std::thread::sleep(std::time::Duration::from_secs(2));
                    // Try again
                    let mut mounter = FileSystemMounter::new(&handler, &mount_cstr, &options);
                    match mounter.mount() {
                        Ok(_filesystem) => {
                            info!("AutoOrtho mounted at {} (retry)", mount_str);
                            shutdown();
                            Ok(())
                        }
                        Err(e) => {
                            shutdown();
                            Err(Box::new(e))
                        }
                    }
                } else {
                    shutdown();
                    Err(Box::new(e))
                }
            }
        }
    }
}
