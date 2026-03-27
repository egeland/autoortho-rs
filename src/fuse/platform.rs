//! Platform-specific FUSE mounting.
//!
//! Each platform has a different FUSE implementation:
//! - macOS: macFUSE via the `fuser` crate
//! - Linux: libfuse via the `fuser` crate (deferred)
//! - Windows: WinFsp (future — not yet implemented)
//!
//! The `mount::AutoOrthoFuse` struct implements the fuser Filesystem trait
//! and works on both macOS and Linux. Windows will need a separate
//! implementation using the `winfsp` crate when that's added.

/// Check if FUSE/virtual filesystem support is available on this platform.
pub fn is_fuse_available() -> bool {
    cfg!(feature = "fuse")
}

/// Platform name for logging.
pub fn platform_name() -> &'static str {
    if cfg!(target_os = "macos") {
        "macOS (macFUSE)"
    } else if cfg!(target_os = "linux") {
        "Linux (libfuse)"
    } else if cfg!(target_os = "windows") {
        "Windows (WinFsp — not yet implemented)"
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
