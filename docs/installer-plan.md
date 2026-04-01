# Plan: Native Installer Packages

**Created:** 2026-03-29
**Updated:** 2026-04-01

## Overview

Create native installers for AutoOrtho Rust across all three platforms:
- **Windows**: MSI installer (.msi) and ZIP (NSIS not supported by cargo-dist)
- **macOS**: ZIP (simplest), DMG (traditional, requires extra tooling), Homebrew
- **Linux**: Tarball and AppImage (AppImage not supported - requires third-party tool)

## Current Status: PARTIALLY COMPLETE 🔄

cargo-dist is configured and release automation is in place. Native installers build on tag push but platform-specific enhancements (FUSE dependency checks) are not yet implemented.

## Strategy

Using **cargo-dist** v0.30+ for cross-platform release automation.

---

## What's Done ✅

| Item | Status |
|------|--------|
| `Cargo.toml` — cargo-dist config (targets, installer suffix) | ✅ |
| `.github/workflows/release.yml` — cargo-dist release workflow | ✅ |
| `.github/workflows/release-plz.yml` — release-plz version bumps | ✅ |
| `.github/workflows/ci.yml` — CI test workflow | ✅ |
| `.github/workflows/cross-platform.yml` — Multi-platform builds | ✅ |
| `.github/workflows/security.yml` — cargo-audit + cargo-deny | ✅ |
| Release profile (LTO thin, strip, panic abort) | ✅ |
| Three target triples configured | ✅ |
| Windows MSI installer | ✅ (with `allow-dirty = ["msi"]`) |

## What's NOT Done ❌

| Item | Status | Complexity |
|------|--------|------------|
| macOS ZIP (change archive format) | ❌ | SIMPLE - one config change |
| macOS DMG bundle | ❌ | COMPLEX - needs app bundle + GitHub Action |
| Linux AppImage | ❌ | COMPLEX - needs third-party tool |
| Windows installer prompts for WinFsp | ❌ | APP CODE - runtime check |
| macOS app checks for macFUSE | ❌ | APP CODE - runtime check |

## Native Installers - Technical Details

### Current State
cargo-dist currently produces:
- **Windows**: `.zip` archives + `.msi` (enabled via `installers = ["msi"]`)
- **macOS**: `.tar.xz` archives
- **Linux**: `.tar.xz` archives

**Note:** MSI requires `allow-dirty = ["msi"]` in dist-workspace.toml because cargo-dist requires WiX template regeneration via `dist init`, which we skip with dirty allow.

### Windows MSI
- MSI is the only Windows bundling installer available in cargo-dist
- NSIS is **not supported** by cargo-dist
- Requires WiX toolset on Windows runners (auto-installed by cargo-dist)
- Configured with `installers = ["msi"]` in Cargo.toml

### macOS Installers

**Option 1: ZIP (Simplest - Recommended First Step)**
- Change `unix-archive` in Cargo.toml to `.zip`
- Already produces `.tar.xz` - switching to ZIP is trivial
- User extracts, drags `.app` to Applications

**Option 2: DMG (Traditional macOS)**
- Requires additional tooling: GitHub Action like `L-Super/create-dmg-actions`
- Requires packaging binary as `.app` bundle
- Needs `Info.plist` file for app metadata
- Optionally: code signing for Gatekeeper

**Option 3: Homebrew Tap**
- cargo-dist supports this natively with `tap` config
- Users install via `brew install`
- Good for power users

#### What's Needed for macOS .app Bundle
To create a proper macOS app bundle (.app), you need:
1. An `Info.plist` file with app metadata
2. App bundle structure: `YourApp.app/Contents/MacOS/yourbinary`
3. Optionally: code signing and notarization

cargo-dist does NOT create app bundles - this would need a custom build step or using a crate like `cargo-bundle`.

### Linux AppImage
- **Not supported** by cargo-dist
- Would require manual build script or third-party tool (e.g., `appimage-builder`)

### Alternative: Shell/PowerShell Installers (Fetching)
These are supported and provide `curl | sh` / `irm | iex` installers:
```toml
[package.metadata.dist]
installers = ["shell", "powershell"]
```
- These fetch binaries from GitHub Releases at install time
- Cross-platform (single installer works on all OSes)
- Does NOT replace native bundling installers

### FUSE Dependency Prompts (Not cargo-dist features)
- These require runtime checks in the application code, not installer config
- Would need to be implemented in the Rust code itself
- Check for WinFsp/macFUSE presence on startup and show user-friendly message

## Acceptance Criteria

- [x] GitHub Release created on tag push (via cargo-dist + release-plz)
- [x] Release builds for Linux x86_64, macOS arm64, Windows x86_64
- [x] All artifacts have checksums (generate-checksums = true)
- [x] Windows .msi installer (enabled via `installers = ["msi"]` + `allow-dirty`)
- [ ] macOS .zip (change `unix-archive = ".zip"` in Cargo.toml) - SIMPLE
- [ ] macOS .dmg (requires app bundle + create-dmg-actions) - COMPLEX
- [ ] Linux .AppImage (NOT supported by cargo-dist - needs third-party tool)
- [ ] Windows installer prompts for WinFsp (requires app code, not installer config)
- [ ] macOS app checks for macFUSE (requires app code, not installer config)