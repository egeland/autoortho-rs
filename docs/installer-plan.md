# Plan: Native Installer Packages

**Created:** 2026-03-29

## Overview

Create native installers for AutoOrtho Rust across all three platforms:
- **Windows**: NSIS installer (.exe) and ZIP
- **macOS**: DMG with app bundle and ZIP
- **Linux**: Tarball and AppImage

## Strategy

Use **cargo-dist** (formerly known as "cxldist") for cross-platform release automation. It provides:
- Unified configuration in `Cargo.toml`
- Builds for all platforms from a single config
- GitHub Actions integration
- Checksum generation
- Multiple output formats

Alternative: Use cargo-binstall for faster installs (optional future enhancement).

---

## Phase 1: Setup cargo-dist

### 1.1 Add cargo-dist to dev-dependencies

```toml
[dev-dependencies]
cargo-dist = "0.26"
```

### 1.2 Configure in Cargo.toml

```toml
[package.metadata.dist]
# The build workflow will create a GitHub Release with this name
dist-version = "0.2.0"

# Create a Windows .exe installer
targets = ["x86_64-pc-windows-msvc", "x86_64-unknown-linux-gnu", "aarch64-apple-darwin"]

# CI will upload artifacts to your GitHub Release
upload-github-release = true

# Generate checksums for all artifacts
generate-checksums = true
```

### 1.3 Create .github/workflows/release.yml

```yaml
name: Release

on:
  push:
    tags:
      - 'v*'

permissions:
  contents: write

jobs:
  release:
    uses: cargo-dist/gha-dist@v0.26.0
```

---

## Phase 2: Platform-Specific Build Requirements

### 2.1 Windows (NSIS Installer)

**Dependencies:**
- WinFsp (bundled or runtime check)
- Visual Studio Build Tools or MinGW

**Installers to produce:**
- `AutoOrtho_0.2.0_x64-setup.exe` (NSIS)
- `AutoOrtho_0.2.0_x64.zip` (portable)

**Features:**
- Start menu shortcuts
- Desktop shortcut (optional)
- Uninstaller
- WinFsp detection prompt

### 2.2 macOS (DMG + App Bundle)

**Dependencies:**
- macFUSE (user-installed, prompt during install)
- Code signing (optional but recommended)
- Notarization (optional)

**Artifacts:**
- `AutoOrtho_0.2.0_aarch64-apple-darwin.dmg`
- `AutoOrtho_0.2.0_aarch64-apple-darwin.tar.gz`

**Note:** Build on macos-latest (Apple Silicon) for arm64 binary.

### 2.3 Linux (Tarball + AppImage)

**Dependencies:**
- libfuse-dev / fuse3

**Artifacts:**
- `autoortho_0.2.0_amd64.tar.gz` (tarball)
- `autoortho_0.2.0_amd64.AppImage` (portable)

---

## Phase 3: FUSE Dependency Handling

### 3.1 Linux

The binary requires libfuse. Options:
1. **Static linking** (complex, not recommended for FUSE)
2. **Runtime dependency** - document in README
3. **AppImage** - bundles everything, most portable

**Recommendation:** Ship AppImage which includes FUSE libraries.

### 3.2 macOS

**macFUSE:** User must install separately. In installer:
- Check for `/Library/Filesystems/macfusefs.fs/Contents/Info.plist`
- Show helpful error if not found
- Provide link to Homebrew: `brew install macfuse`

### 3.3 Windows

WinFsp is required. In installer:
- Check for WinFsp DLL in PATH or program files
- Provide download link if not found

---

## Phase 4: Build Scripts

### 4.1 Local Build Commands

```bash
# Install dependencies
cargo install cargo-dist

# Build locally (all platforms)
cargo dist build

# Build for specific platform
cargo dist build --target x86_64-pc-windows-msvc

# Upload to GitHub
cargo dist upload
```

### 4.2 CI Configuration

```yaml
# .github/workflows/ci.yml
name: CI

on:
  push:
    branches: [main]
  pull_request:

jobs:
  test:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - uses: dtolnay/rust-action@stable
      - run: cargo test --all

  dist:
    needs: test
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - uses: dtolnay/rust-action@stable
      - uses: cargo-dist/gha-dist@v0.26.0
        with:
          # Generate a "plan" artifact for the release PR
          upload-plan: ${{ github.event_name == 'pull_request' }}
```

---

## Phase 5: Release Process

### 5.1 Version Tagging

```bash
# Create release tag
git tag -a v0.2.0 -m "Release v0.2.0"
git push origin v0.2.0
```

### 5.2 GitHub Release

cargo-dist will:
1. Build all targets
2. Generate checksums
3. Create GitHub Release
4. Upload artifacts
5. Render release notes

### 5.3 Manual Steps (if needed)

- Edit release notes
- Add screenshots/changelog
- Mark as "Latest" (or keep pre-release)

---

## Phase 6: Installer UX

### 6.1 Windows Installer (NSIS)

**Welcome Screen:**
- AutoOrtho v0.2.0
- Satellite imagery for X-Plane

**License:** GPL-3.0 / Apache-2.0 (display license file)

**WinFsp Check:**
```
WinFsp is required for AutoOrtho to work.
Download from: https://github.com/winfsp/winfsp
[Download] [Continue Anyway]
```

**Install Location:** Default `C:\Program Files\AutoOrtho`

**Finish:**
- Launch AutoOrtho
- Open documentation

### 6.2 macOS DMG

**Contents:**
- AutoOrtho.app
- Applications → Install shortcut
- Documentation folder

**Post-Install:**
- Prompt to install macFUSE if missing
- Gatekeeper bypass instructions (unsigned)

### 6.3 Linux

**AppImage:** Self-contained, no install needed
```bash
chmod +x AutoOrtho_*.AppImage
./AutoOrtho_*.AppImage
```

**Tarball:**
```bash
tar -xzf autoortho_*.tar.gz
cd autoortho
./bin/autoortho
```

---

## Files to Modify

| File | Changes |
|------|---------|
| `Cargo.toml` | Add cargo-dist config |
| `.github/workflows/release.yml` | New CI workflow |

## Files to Create

| File | Description |
|------|-------------|
| `.github/workflows/ci.yml` | CI test workflow |
| `installer/windows/` | NSIS scripts (optional) |
| `README.md` | Update with install instructions |

## Acceptance Criteria

- [ ] `cargo dist build` produces Windows .exe and .zip
- [ ] `cargo dist build` produces macOS .dmg and .tar.gz
- [ ] `cargo dist build` produces Linux .tar.gz and .AppImage
- [ ] GitHub Release created on tag push
- [ ] All artifacts have checksums
- [ ] Windows installer prompts for WinFsp
- [ ] macOS app checks for macFUSE

## Time Estimate

- Phase 1-2 (Setup): 2-3 hours
- Phase 3 (FUSE handling): 1-2 hours
- Phase 4-5 (CI/Process): 2-3 hours
- Phase 6 (UX): 2-3 hours

**Total:** ~10 hours

## References

- [cargo-dist Book](https://dist.cargo.rs/)
- [cargo-dist GitHub Action](https://github.com/cargo-dist/gha-dist)
- [NSIS Installer](https://nsis.sourceforge.io/)
- [AppImage](https://appimage.org/)
