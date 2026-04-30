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
        // Try to unmount using WinFsp's fspmount tool
        // This is similar to Python's winsetup.force_unmount()
        let status = std::process::Command::new("fspmount")
            .args(["-u", &mount_str])
            .status();
        match status {
            Ok(s) if s.success() => {
                log::info!("WinFsp unmount succeeded for {}", mount_str);
            }
            _ => {
                // Ignore errors - maybe not mounted
                log::debug!("WinFsp unmount failed or not mounted: {:?}", status);
            }
        }
        // Also try fsutil
        let _ = std::process::Command::new("fsutil")
            .args(["volume", "dismount", &mount_str])
            .status();
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
            .map_or(false, |s| s.success())
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
