# CI/CD Workflow Consolidation Plan

## Overview

Simplify the GitHub Actions workflows by:
1. Moving cross-platform tests into ci.yml (as dependent job)
2. Removing push triggers (all CI only runs on PRs)
3. Moving Windows installer into release.yml as a dependent job
4. Ensuring release only happens after all tests pass

## Final Flow

### PR Triggers (all run in parallel on PR to main)

- **ci.yml**: `check` → `test` → `cross-platform` (depends on `test`)
- **security.yml**: `audit` + `deny` (parallel)

### Release Flow (on push to main)

- **release-plz.yml**: push to main → version PR → merge → tag
- **release.yml** (on tag):
  - `plan` → `build-local-artifacts` → `build-global-artifacts` → `host` (creates GitHub Release)
  - `installer` (depends on `host`):
    1. Extract version from tag
    2. Download Windows binary from `build-local-artifacts` artifact
    3. Copy to `artifacts/autoortho.exe`
    4. Install WiX: `dotnet tool install --global wix`
    5. Build MSI: `wix build wix/main.wxs -o autoortho.msi -bv Version=$VERSION`
    6. Upload to existing release via `softprops/action-gh-release` (auto-updates existing release)

## Files to Modify

| File | Changes |
|------|---------|
| `ci.yml` | Add `cross-platform` job (matrix: ubuntu/macos/windows) depending on `test` |
| `cross-platform.yml` | **Delete** |
| `security.yml` | Remove push to main trigger |
| `release-plz.yml` | Remove workflow_run trigger |
| `release.yml` | Add `installer` job after `host`, downloads Windows binary, builds MSI, uploads to release |
| `installer.yml` | **Delete** (functionality moved to release.yml) |

## Technical Details

### Installer Job in release.yml

```yaml
installer:
  needs:
    - plan
    - host
  if: needs.host.result == 'success'
  runs-on: windows-latest
  env:
    GH_TOKEN: ${{ secrets.GITHUB_TOKEN }}
  steps:
    - uses: actions/checkout@v6
      with:
        persist-credentials: false

    - name: Extract version
      id: version
      run: |
        $tag = "${{ needs.plan.outputs.tag }}"
        $version = $tag -replace '^v', ''
        echo "version=$version" >> $env:GITHUB_OUTPUT

    - name: Download Windows binary
      uses: actions/download-artifact@v7
      with:
        pattern: artifacts-build-local-*
        path: target/
        merge-multiple: true

    - name: Setup binary for WiX
      run: |
        New-Item -ItemType Directory -Force -Path artifacts | Out-Null
        Get-ChildItem -Path target -Recurse -Filter "*autoortho*.exe" | Copy-Item -Destination "artifacts/autoortho.exe" -Force

    - name: Install WiX
      run: dotnet tool install --global wix

    - name: Build MSI
      run: |
        wix build wix/main.wxs -o autoortho-x86_64-pc-windows-msvc.msi -bv Version=${{ steps.version.outputs.version }}

    - name: Upload to release
      uses: softprops/action-gh-release@v2
      with:
        tag_name: ${{ needs.plan.outputs.tag }}
        files: autoortho-x86_64-pc-windows-msvc.msi
```

### Key Notes

1. **softprops/action-gh-release** automatically updates existing releases - no special config needed
2. **WiX v4** uses `-bv Version=$VERSION` to set version at build time
3. **Branch protection** should require ci.yml + security.yml to pass before merging to main

## Assumptions

- Branch protection rules require ci.yml + security.yml to pass before merging to main
- release-plz creates version bump PRs that get auto-merged
- Tag pushed after merge triggers the release flow
