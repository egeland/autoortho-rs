//! Platform-specific FUSE mounting.
//!
//! Each platform has a different FUSE implementation:
//! - macOS: macFUSE via the `fuser` crate
//! - Linux: libfuse via the `fuser` crate (deferred)
//! - Windows: WinFsp via the `winfsp` crate
//!
//! The `mount::AutoOrthoFuse` struct implements the fuser Filesystem trait
//! and works on both macOS and Linux. Windows uses a separate
//! implementation using the `winfsp` crate.

/// Check if FUSE/virtual filesystem support is available on this platform.
pub fn is_fuse_available() -> bool {
    #[cfg(windows)]
    {
        // Check if WinFsp is installed by trying to initialize it
        winfsp::winfsp_init().is_ok()
    }
    #[cfg(target_os = "linux")]
    {
        let fuse_device = std::path::Path::new("/dev/fuse");
        fuse_device.exists()
            && std::fs::OpenOptions::new()
                .read(true)
                .write(true)
                .open(fuse_device)
                .is_ok()
    }
    #[cfg(target_os = "macos")]
    {
        // macFUSE device naming has varied across versions/installations.
        // Treat FUSE as available if a known device node exists and is
        // accessible for read/write.
        for fuse_device in ["/dev/macfuse0", "/dev/osxfuse0"] {
            let fuse_device = std::path::Path::new(fuse_device);
            if fuse_device.exists()
                && std::fs::OpenOptions::new()
                    .read(true)
                    .write(true)
                    .open(fuse_device)
                    .is_ok()
            {
                return true;
            }
        }
        false
    }
    #[cfg(not(any(windows, target_os = "linux", target_os = "macos")))]
    {
        false
    }
}

/// Platform name for logging.
pub fn platform_name() -> &'static str {
    if cfg!(target_os = "macos") {
        "macOS (macFUSE)"
    } else if cfg!(target_os = "linux") {
        "Linux (libfuse)"
    } else if cfg!(target_os = "windows") {
        "Windows (WinFsp)"
    } else {
        "Unknown platform"
    }
}

/// Check if the platform requires a kernel extension or driver for FUSE.
pub fn requires_driver_install() -> bool {
    // macOS requires macFUSE kext, Windows requires WinFsp driver
    // Linux typically has FUSE built into the kernel
    cfg!(target_os = "macos") || cfg!(target_os = "windows")
}

/// Attempt to clean up any stale mount at the given path before mounting.
/// This is similar to Python's `setupmount()` which checks for existing mounts
/// and unmounts them before mounting again.
pub fn cleanup_mount(mountpoint: &std::path::Path) -> Result<(), Box<dyn std::error::Error>> {
    let mount_str = mountpoint.to_string_lossy().to_string();

    #[cfg(windows)]
    {
        // Use FspMountRemove from winfsp-sys to remove stale mount point
        use std::os::windows::ffi::OsStrExt;
        use winfsp_sys::{FSP_MOUNT_DESC, FspMountRemove};

        let mount_norm = mount_str.replace('/', "\\");

        // Convert path to wide string (null-terminated)
        let wide_path: Vec<u16> = std::ffi::OsStr::new(&mount_norm)
            .encode_wide()
            .chain(std::iter::once(0))
            .collect();

        // Create FSP_MOUNT_DESC with the mount point
        let mut desc = FSP_MOUNT_DESC {
            VolumeHandle: std::ptr::null_mut(),
            VolumeName: std::ptr::null_mut(),
            Security: std::ptr::null_mut(),
            Reserved: 0,
            MountPoint: wide_path.as_ptr() as *mut u16,
            MountHandle: std::ptr::null_mut(),
        };

        // Call FspMountRemove to remove the stale mount point
        let status = unsafe { FspMountRemove(&mut desc as *mut FSP_MOUNT_DESC) };

        if status == 0 {
            log::info!("Successfully removed stale mount point: {}", mount_norm);
        } else {
            // STATUS_OBJECT_NAME_NOT_FOUND (0xC0000034) means not mounted - that's OK
            const STATUS_OBJECT_NAME_NOT_FOUND: i32 = 0xC0000034_u32 as i32;
            if status == STATUS_OBJECT_NAME_NOT_FOUND {
                log::debug!("Mount point not found (not mounted): {}", mount_norm);
            } else {
                log::warn!(
                    "FspMountRemove returned NTSTATUS: {:#X} for {}",
                    status,
                    mount_norm
                );
            }
        }
    }

    #[cfg(target_os = "macos")]
    {
        let status = std::process::Command::new("diskutil")
            .args(["unmount", "force", &mount_str])
            .status();
        match status {
            Ok(s) if s.success() => {
                log::info!("macOS unmount succeeded for {}", mount_str);
            }
            _ => {
                log::debug!("macOS unmount failed or not mounted");
            }
        }
    }

    #[cfg(target_os = "linux")]
    {
        // Try fusermount -u
        if std::process::Command::new("fusermount")
            .args(["-u", "-z", &mount_str])
            .status()
            .is_ok_and(|s| s.success())
        {
            log::info!("Linux fusermount succeeded for {}", mount_str);
        } else {
            // Try umount -l
            let _ = std::process::Command::new("umount")
                .args(["-l", &mount_str])
                .status();
        }
    }

    // Give OS time to clean up
    std::thread::sleep(std::time::Duration::from_secs(1));

    Ok(())
}
