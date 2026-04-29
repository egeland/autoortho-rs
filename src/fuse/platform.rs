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
        return false;
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
