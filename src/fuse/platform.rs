//! Platform-specific FUSE mounting.
//!
//! Each platform has a different FUSE implementation:
//! - macOS: macFUSE via the `fuser` crate
//! - Linux: libfuse via the `fuser` crate (deferred)
//! - Windows: Dokan via the `dokan` crate
//!
//! The `mount::AutoOrthoFuse` struct implements the fuser Filesystem trait
//! and works on both macOS and Linux. Windows uses a separate
//! implementation using the `dokan` crate.

/// Check if FUSE/virtual filesystem support is available on this platform.
pub fn is_fuse_available() -> bool {
    #[cfg(windows)]
    {
        // Check if Dokan is installed by checking the version
        // DokanVersion returns 0 if not installed
        let version = unsafe { dokan_sys::DokanVersion() };
        version != 0
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
        "Windows (Dokan)"
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
        // Dokan uses 'dokan unmount' or we can let the mount handle cleanup
        // Try to remove stale Dokan mount point via command line
        if std::process::Command::new("dokan")
            .args(["unmount", &mount_str])
            .status()
            .is_ok_and(|s| s.success())
        {
            log::info!(
                "Successfully removed stale Dokan mount point: {}",
                mount_str
            );
        } else {
            log::debug!("Dokan unmount failed or not mounted");
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
