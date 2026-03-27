//! UI helper functions — disk space, formatting, icons.
//!
//! Icons use Nerd Font codepoints from the bundled FiraCode Nerd Font.
//! Reference: https://www.nerdfonts.com/cheat-sheet

use std::path::Path;

// Nerd Font icon codepoints
pub const ICON_DOWNLOAD: &str = "\u{f019}"; // nf-fa-download
pub const ICON_FOLDER: &str = "\u{f07b}"; // nf-fa-folder
pub const ICON_PLAY: &str = "\u{f04b}"; // nf-fa-play
pub const ICON_STOP: &str = "\u{f04d}"; // nf-fa-stop
pub const ICON_SETTINGS: &str = "\u{f013}"; // nf-fa-gear (cog)
pub const ICON_TRASH: &str = "\u{f1f8}"; // nf-fa-trash
pub const ICON_CHECK: &str = "\u{f00c}"; // nf-fa-check
pub const ICON_GLOBE: &str = "\u{f0ac}"; // nf-fa-globe
pub const ICON_PLANE: &str = "\u{f072}"; // nf-fa-plane
pub const ICON_MAP: &str = "\u{f279}"; // nf-fa-map
pub const ICON_REFRESH: &str = "\u{f021}"; // nf-fa-refresh
pub const ICON_INFO: &str = "\u{f05a}"; // nf-fa-info_circle
pub const ICON_WARNING: &str = "\u{f071}"; // nf-fa-warning
pub const ICON_DEV: &str = "\u{f121}"; // nf-fa-code
pub const ICON_TIMES: &str = "\u{f00d}"; // nf-fa-times (x/close)
pub const ICON_SEARCH: &str = "\u{f002}"; // nf-fa-search
pub const ICON_HOME: &str = "\u{f015}"; // nf-fa-home
pub const ICON_CLOUD_DL: &str = "\u{f0ed}"; // nf-fa-cloud_download

/// Get available disk space for a path, in bytes.
pub fn available_space(path: &str) -> Option<u64> {
    let p = Path::new(path);
    let check = if p.exists() {
        p.to_path_buf()
    } else if let Some(parent) = p.parent() {
        if parent.exists() {
            parent.to_path_buf()
        } else {
            return None;
        }
    } else {
        return None;
    };

    fs2::available_space(&check).ok()
}

/// Format bytes as human-readable string.
pub fn format_bytes(bytes: u64) -> String {
    if bytes < 1024 {
        format!("{} B", bytes)
    } else if bytes < 1024 * 1024 {
        format!("{:.1} KB", bytes as f64 / 1024.0)
    } else if bytes < 1024 * 1024 * 1024 {
        format!("{:.1} MB", bytes as f64 / (1024.0 * 1024.0))
    } else {
        format!("{:.1} GB", bytes as f64 / (1024.0 * 1024.0 * 1024.0))
    }
}

/// Format available disk space for display.
pub fn disk_space_label(path: &str) -> String {
    match available_space(path) {
        Some(bytes) => format!("{} free", format_bytes(bytes)),
        None => String::new(),
    }
}
