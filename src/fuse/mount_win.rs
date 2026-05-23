//! Windows FUSE mount implementation using Dokan.
//!
//! Dokan provides Windows user-mode file system functionality, similar to FUSE on Unix.
//! This implementation uses the updated dokan-rust API (GitHub version).

pub use self::dokan_impl::mount;

mod dokan_impl {
    use crate::app_context::AppContext;
    use crate::fuse::filesystem::DdsFileSystem;
    use crate::fuse::{MARKER_FILE, VIRTUAL_DIRS, is_poison_path};
    use dokan::{
        CreateFileInfo, DiskSpaceInfo, FileInfo, FileSystemHandler, FileSystemMountError,
        FileSystemMounter, FindData, MountFlags, MountOptions, OperationInfo, OperationResult,
        VolumeInfo, init,
    };
    use log::{debug, error, info, warn};
    use std::collections::HashMap;
    use std::path::{Path, PathBuf};
    use std::sync::{Arc, Mutex, RwLock};
    use std::time::SystemTime;
    use widestring::U16CStr;
    use winapi::shared::ntstatus::*;
    use winapi::um::winnt;

    const ROOT_INO: u64 = 1;
    const TEXTURES_INO: u64 = 2;
    const TERRAIN_INO: u64 = 3;
    const MARKER_INO: u64 = 4;
    const DYNAMIC_INO_START: u64 = 1000;

    /// Context type for open files
    #[derive(Debug)]
    pub struct FileContext {
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
    }

    impl<'c, 'h: 'c> FileSystemHandler<'c, 'h> for AutoOrthoHandler {
        type Context = FileContext;

        fn create_file(
            &'h self,
            file_name: &U16CStr,
            _security_context: &dokan_sys::DOKAN_IO_SECURITY_CONTEXT,
            _desired_access: u32,
            _file_attributes: u32,
            _share_access: u32,
            _create_disposition: u32,
            _create_options: u32,
            #[allow(unused_variables)] _info: &mut OperationInfo<'c, 'h, Self>,
        ) -> OperationResult<CreateFileInfo<Self::Context>> {
            let path_str = file_name.to_string_lossy();
            let path = Path::new(&path_str);
            // Store the converted string to avoid temporary value
            let path_str_converted = self.path_to_string(path);
            let clean_path = Path::new(&path_str_converted);

            debug!("dokan create_file: {:?}", path_str);

            if is_poison_path(&path_str) {
                info!("Poison pill detected at {}. Shutting down.", path_str);
                return Err(STATUS_FILE_IS_A_DIRECTORY);
            }

            let ino = self.path_to_inode(clean_path);
            let attr = self.runtime.block_on(self.fs.get_attr(&path_str_converted));

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

                    Ok(CreateFileInfo {
                        context,
                        is_dir: file_attr.is_dir,
                        new_file_created: false,
                    })
                }
                Err(_) => Err(STATUS_OBJECT_NAME_NOT_FOUND),
            }
        }

        fn cleanup(
            &self,
            _file_name: &U16CStr,
            _info: &OperationInfo<'c, 'h, Self>,
            _context: &'c Self::Context,
        ) {
            // Nothing special needed for cleanup
        }

        fn close_file(
            &self,
            _file_name: &U16CStr,
            _info: &OperationInfo<'c, 'h, Self>,
            context: &'c Self::Context,
        ) {
            debug!("dokan close_file: {:?}", context.path);
            self.open_files.write().unwrap().remove(&context.inode);
        }

        fn read_file(
            &self,
            _file_name: &U16CStr,
            offset: i64,
            buffer: &mut [u8],
            _info: &OperationInfo<'c, 'h, Self>,
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
                    Err(STATUS_UNSUCCESSFUL)
                }
            }
        }

        fn find_files(
            &self,
            _file_name: &U16CStr,
            mut fill_find_data: impl FnMut(&FindData) -> dokan::FillDataResult,
            _info: &OperationInfo<'c, 'h, Self>,
            context: &'c Self::Context,
        ) -> OperationResult<()> {
            let dir_path = &context.path;
            let dir_path_str = dir_path.to_string_lossy().replace('\\', "/");
            let is_root = dir_path_str == "/" || dir_path_str.is_empty();
            let is_textures = dir_path_str.ends_with("/textures");
            let is_terrain = dir_path_str.ends_with("/terrain");

            debug!(
                "dokan find_files: path={} is_root={} is_textures={} is_terrain={}",
                dir_path_str, is_root, is_textures, is_terrain
            );

            let now = SystemTime::now();

            // Add . and ..
            let dot = FindData {
                attributes: winnt::FILE_ATTRIBUTE_DIRECTORY,
                creation_time: now,
                last_access_time: now,
                last_write_time: now,
                file_size: 0,
                file_name: widestring::U16CString::from_str(".").unwrap(),
            };
            if fill_find_data(&dot).is_err() {
                debug!("dokan find_files: failed to add '.' entry");
                return Err(STATUS_UNSUCCESSFUL);
            }

            let dotdot = FindData {
                attributes: winnt::FILE_ATTRIBUTE_DIRECTORY,
                creation_time: now,
                last_access_time: now,
                last_write_time: now,
                file_size: 0,
                file_name: widestring::U16CString::from_str("..").unwrap(),
            };
            if fill_find_data(&dotdot).is_err() {
                debug!("dokan find_files: failed to add '..' entry");
                return Err(STATUS_UNSUCCESSFUL);
            }

            // Use DdsFileSystem.list_dir() for all directory listings
            // This gives us virtual dirs + pass-through entries from root
            match self.fs.list_dir(&dir_path_str) {
                Ok(entries) => {
                    for entry_name in entries {
                        // Skip . and .. (we already added them above)
                        if entry_name == "." || entry_name == ".." {
                            continue;
                        }

                        // Determine if entry is a directory
                        let is_dir = VIRTUAL_DIRS.contains(&entry_name.as_str())
                            || self.fs.is_dir_in_root(&dir_path_str, &entry_name);

                        let data = FindData {
                            attributes: if is_dir {
                                winnt::FILE_ATTRIBUTE_DIRECTORY
                            } else {
                                winnt::FILE_ATTRIBUTE_NORMAL
                            },
                            creation_time: now,
                            last_access_time: now,
                            last_write_time: now,
                            file_size: 0,
                            file_name: widestring::U16CString::from_str(&entry_name).unwrap(),
                        };
                        if fill_find_data(&data).is_err() {
                            debug!("dokan find_files: failed to add entry '{}'", entry_name);
                            return Err(STATUS_UNSUCCESSFUL);
                        }
                    }
                }
                Err(e) => {
                    debug!(
                        "dokan find_files: list_dir failed for {}: {:?}",
                        dir_path_str, e
                    );
                    // Path doesn't exist or is not a directory
                    return Err(STATUS_OBJECT_NAME_NOT_FOUND);
                }
            }

            debug!(
                "dokan find_files: completed successfully for path={}",
                dir_path_str
            );
            Ok(())
        }

        fn get_file_information(
            &self,
            _file_name: &U16CStr,
            _info: &OperationInfo<'c, 'h, Self>,
            context: &'c Self::Context,
        ) -> OperationResult<FileInfo> {
            let path_str = self.path_to_string(&context.path);

            if let Ok(attr) = self.runtime.block_on(self.fs.get_attr(&path_str)) {
                let now = SystemTime::now();

                return Ok(FileInfo {
                    attributes: if attr.is_dir {
                        winnt::FILE_ATTRIBUTE_DIRECTORY
                    } else {
                        winnt::FILE_ATTRIBUTE_NORMAL
                    },
                    creation_time: now,
                    last_access_time: now,
                    last_write_time: now,
                    file_size: attr.size,
                    number_of_links: 0,
                    file_index: context.inode,
                });
            }

            Err(STATUS_OBJECT_NAME_NOT_FOUND)
        }

        fn get_volume_information(
            &self,
            _info: &OperationInfo<'c, 'h, Self>,
        ) -> OperationResult<VolumeInfo> {
            // Use U16CString for owned string since both widestring 0.4 and 1.x support this
            Ok(VolumeInfo {
                name: widestring::U16CString::from_str("AutoOrtho").unwrap(),
                serial_number: 0x12345678,
                max_component_length: 256,
                fs_flags: 0,
                fs_name: widestring::U16CString::from_str("Dokan").unwrap(),
            })
        }

        fn get_disk_free_space(
            &self,
            _info: &OperationInfo<'c, 'h, Self>,
        ) -> OperationResult<DiskSpaceInfo> {
            Ok(DiskSpaceInfo {
                byte_count: 1_000_000_000_000,
                free_byte_count: 500_000_000_000,
                available_byte_count: 500_000_000_000,
            })
        }
    }

    pub fn mount(
        fs: Arc<DdsFileSystem>,
        mountpoint: &Path,
        runtime: tokio::runtime::Handle,
        app_context: Arc<AppContext>,
    ) -> Result<(), Box<dyn std::error::Error>> {
        // Initialize Dokan library
        init();

        let handler = AutoOrthoHandler::new(fs, runtime);
        let mount_str = mountpoint.to_string_lossy().to_string();
        let mount_cstr = widestring::U16CString::from_str(&mount_str)?;

        info!("Mounting AutoOrtho at {} using Dokan", mount_str);

        // Cleanup stale mount
        let _ = crate::fuse::platform::cleanup_mount(mountpoint);
        std::thread::sleep(std::time::Duration::from_secs(1));

        let options = MountOptions {
            // CURRENT_SESSION: only visible to the current user session
            // FILELOCK_USER_MODE: use user-mode locking instead of kernel
            flags: MountFlags::CURRENT_SESSION | MountFlags::FILELOCK_USER_MODE,
            ..Default::default()
        };

        // Create an owned copy for the mounter
        let mount_point: &widestring::U16CStr = mount_cstr.as_ucstr();
        let mut mounter = FileSystemMounter::new(&handler, mount_point, &options);

        /// Map Dokan mount errors to user-friendly messages
        fn describe_mount_error(err: &FileSystemMountError) -> &'static str {
            match err {
                FileSystemMountError::General => "General Dokan error",
                FileSystemMountError::DriveLetter => "Invalid drive letter",
                FileSystemMountError::DriverInstall => {
                    "Dokan driver not installed or failed to install"
                }
                FileSystemMountError::Start => "Dokan driver failed to start",
                FileSystemMountError::Mount => {
                    "Mount point already in use — collision with existing mount"
                }
                FileSystemMountError::MountPoint => {
                    "Mount point is invalid (path doesn't exist or is not a directory)"
                }
                FileSystemMountError::Version => "Dokan library version mismatch",
            }
        }

        // This blocks until unmounted
        match mounter.mount() {
            Ok(fs) => {
                info!("AutoOrtho mounted at {}", mount_str);
                app_context.set_dokan_mount(fs);
                Ok(())
            }
            Err(e) => {
                let desc = describe_mount_error(&e);
                error!("Failed to mount at {}: {} ({:?})", mount_str, desc, e);

                // Mount collision — clean up stale mount and retry
                if matches!(e, FileSystemMountError::Mount) {
                    warn!(
                        "Mount collision at {}, attempting cleanup and retry...",
                        mount_str
                    );
                    let _ = crate::fuse::platform::cleanup_mount(mountpoint);
                    std::thread::sleep(std::time::Duration::from_secs(2));

                    let mount_point: &widestring::U16CStr = mount_cstr.as_ucstr();
                    let mut mounter = FileSystemMounter::new(&handler, mount_point, &options);
                    match mounter.mount() {
                        Ok(fs) => {
                            info!("AutoOrtho mounted at {} (retry)", mount_str);
                            app_context.set_dokan_mount(fs);
                            Ok(())
                        }
                        Err(e) => {
                            let desc = describe_mount_error(&e);
                            error!("Retry failed at {}: {} ({:?})", mount_str, desc, e);
                            // Do NOT call shutdown() - cleanup_mount handles unmount
                            Err(Box::new(e))
                        }
                    }
                } else {
                    // Do NOT call shutdown() - cleanup_mount handles unmount
                    Err(Box::new(e))
                }
            }
        }
    }
}
