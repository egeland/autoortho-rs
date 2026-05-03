# Installation Guide

This guide covers installation of AutoOrtho on macOS, Linux, and Windows.

## System Requirements

- **Operating System**: macOS 12+, Linux (Ubuntu 20.04+, Fedora 35+), Windows 10/11
- **X-Plane**: Version 11 or 12
- **Disk Space**: 2GB minimum for base installation, more for tile cache
- **RAM**: 4GB minimum, 8GB recommended
- **Network**: Broadband connection for tile downloading

## Pre-requisites

### Rust Toolchain

AutoOrtho is built with Rust. Install the Rust toolchain:

```bash
# Install Rust (if not already installed)
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# Verify installation
rustc --version
cargo --version
```

## macOS Installation

### 1. Install macFUSE

AutoOrtho requires macFUSE for the virtual filesystem:

```bash
# Using Homebrew
brew install macfuse

# Or download from: https://github.com/macfuse/macfuse/releases
```

After installation, you may need to approve the macFUSE extension in System Settings > Privacy & Security.

### 2. Build AutoOrtho

```bash
# Clone the repository
git clone https://github.com/egeland/autoortho-rs.git
cd autoortho-rs

# Build release version
cargo build --release

# The binary will be at target/release/autoortho
```

### 3. First Run

```bash
# Launch with GUI
./target/release/autoortho --gui

# Or configure via command line
./target/release/autoortho --xplane "/Applications/X-Plane 12"
```

### 4. X-Plane Configuration

1. Open AutoOrtho GUI
2. Browse to your X-Plane folder (e.g., `/Applications/X-Plane 12`)
3. Configure tile provider (default: ArcGIS)
4. Click "Start" to mount the filesystem and begin

## Linux Installation

### 1. Install FUSE Development Libraries

```bash
# Ubuntu/Debian
sudo apt update
sudo apt install libfuse2 libfuse-dev pkg-config

# Fedora
sudo dnf install fuse fuse-devel pkg-config

# Arch Linux
sudo pacman -S fuse2
```

### 2. Build AutoOrtho

```bash
# Clone the repository
git clone https://github.com/egeland/autoortho-rs.git
cd autoortho-rs

# Install build dependencies (Ubuntu/Debian example)
sudo apt install build-essential pkg-config libfuse-dev

# Build release version
cargo build --release

# The binary will be at target/release/autoortho
```

### 3. First Run

```bash
# Launch with GUI
./target/release/autoortho --gui

# Or configure via command line
./target/release/autoortho --xplane ~/X-Plane
```

### 4. Permissions

FUSE requires appropriate permissions. If you encounter permission errors:

```bash
# Add your user to the fuse group
sudo usermod -a -G fuse $USER

# Log out and back in for group changes to take effect
```

## Windows Installation

### 1. Install Dokan2

AutoOrtho uses Dokan2 for the virtual filesystem on Windows:

1. Download Dokan2 from: https://github.com/dokan-dev/dokan/releases
2. Run the installer (DokanSetup.exe)
3. Complete the installation wizard

### 2. Build AutoOrtho

```powershell
# Clone the repository
git clone https://github.com/egeland/autoortho-rs.git
cd autoortho-rs

# Build release version (use MSVC toolchain for best compatibility)
rustup default stable-msvc
cargo build --release

# The binary will be at target/release/autoortho.exe
```

### 3. First Run

```powershell
# Launch with GUI
.\target\release\autoortho.exe --gui

# Or configure via command line
.\target\release\autoortho.exe --xplane "C:\Program Files\xplane 12"
```

## Post-Installation

### Verify Installation

Run with verbose logging to verify everything works:

```bash
RUST_LOG=debug ./target/release/autoortho --gui
```

Look for these success messages:
- `Web UI server listening on http://...`
- `X-Plane tracker initialized`
- `FUSE filesystem mounted at ...`

### Configure AutoOrtho

1. **Tile Provider**: Choose from ArcGIS (default), Google Maps, Bing Maps, USGS, etc.
2. **Zoom Range**: Set min/max zoom levels (default: 10-18)
3. **Cache Location**: Configure where tiles are stored
4. **Night Exclusion**: Enable automatic night mode based on sun position

### X-Plane Settings

1. Set X-Plane to use the AutoOrtho folder as a scenery pack:
   - X-Plane Settings > Scenery > Settings
   - Add the AutoOrtho mount path to scenery configuration
   - Recommended: Place at top of scenery priority list

2. Configure X-Plane graphics settings:
   - Higher texture resolution = more detailed satellite imagery
   - AutoOrtho provides up to 4K textures

## Troubleshooting

### macOS: "System Extension Blocked"

If you see "System Extension Blocked" errors:
1. Go to System Settings > Privacy & Security
2. Scroll down to Security
3. Look for "System Extensions" or "Developer Tools"
4. Allow the macFUSE extension

### macOS: "Developer cannot be verified"

Right-click the binary and select "Open" to bypass Gatekeeper warnings.

### Linux: "fuse: permission denied"

```bash
# Check if /dev/fuse exists
ls -la /dev/fuse

# If not, create it
sudo mknod /dev/fuse 0 0
sudo chmod 666 /dev/fuse
```

### Windows: "Dokan2 not installed"

Ensure Dokan2 is installed and running. Check in Services:
1. Press Win+R, type `services.msc`
2. Look for "Dokan" service
3. Ensure it's started and set to Automatic

### All Platforms: "Port already in use"

Another application is using the web UI port (default 5847). Either:
1. Stop the other application
2. Configure AutoOrtho to use a different port in settings

## Uninstallation

### macOS/Linux

```bash
# Stop any running instances
pkill autoortho

# Remove the binary
rm /path/to/autoortho

# Remove configuration (optional)
rm -rf ~/.config/autoortho
rm -rf ~/Library/Application\ Support/autoortho  # macOS only
```

### Windows

```powershell
# Stop any running instances
taskkill /F /IM autoortho.exe

# Remove the binary
del C:\path\to\autoortho.exe

# Remove configuration (optional)
rmdir /s %APPDATA%\autoortho
```

To uninstall macFUSE/Dokan2, use the respective uninstaller or system settings.

## Updating

```bash
# Pull latest changes
git pull

# Rebuild
cargo build --release
```

## Getting Help

- **Issues**: https://github.com/egeland/autoortho-rs/issues
- **Discussions**: https://github.com/egeland/autoortho-rs/discussions
