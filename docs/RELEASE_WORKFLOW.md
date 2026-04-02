# Release Workflow

## Overview

Haptic Harmony Simulator's canonical release pipeline lives in `.github/workflows/release.yml` and follows the same **validate first, publish last** pattern used across Gestura repos. Downstream package-manager publishing lives in `.github/workflows/package-managers.yml` and runs separately after a GitHub release is published.

## Triggers

### Release build and publish

- `push` to a tag matching `v*`
- `workflow_dispatch` with a required `tag` input such as `v0.1.0`

### Package-manager publishing

- `release.published`
- `workflow_dispatch` with a required `tag` input such as `v0.1.0`

## Version and tag rules

The release workflow validates these files before any platform packaging starts:

- `Cargo.toml` → `package.version`
- `tauri.conf.json` → `version`
- `ui/package.json` → `version`

The workflow fails immediately if:

- any version differs
- the version is not valid semver
- a tag-triggered run is not for `v<version>`

## Release job layout

1. **Prepare**
   - resolves the release version and prerelease status
   - validates tag/version consistency
2. **Validate**
   - runs cross-platform checks before packaging
   - runs the Linux all-features surface in CI
3. **Package**
   - builds signed macOS and Windows bundles when secrets are present
   - builds Linux bundles and CLI archives
   - normalizes release asset names
4. **Publish GitHub release**
   - downloads packaged artifacts
   - generates a unified SHA256 manifest
   - creates or updates the GitHub release

## Canonical release assets

The release workflow publishes normalized installer artifacts and companion CLI archives:

- macOS
  - `haptic-harmony-simulation-macos-x64.dmg`
  - `haptic-harmony-simulation-macos-arm64.dmg`
- Linux
  - `haptic-harmony-simulation-linux-x64.AppImage`
  - `haptic-harmony-simulation-linux-x64.deb`
  - `haptic-harmony-simulation-linux-x64.rpm` when available
- Windows
  - `haptic-harmony-simulation-windows-x64.exe`
  - `haptic-harmony-simulation-windows-x64.msi`
- Checksums
  - `haptic-harmony-simulation-SHA256SUMS.txt`

## Required secrets for signed releases

### macOS

- `APPLE_CERTIFICATE`
- `APPLE_CERTIFICATE_PASSWORD`
- `APPLE_SIGNING_IDENTITY`
- `APPLE_ID`
- `APPLE_PASSWORD`
- `APPLE_TEAM_ID`

### Windows

- `WINDOWS_CERTIFICATE`
- `WINDOWS_CERTIFICATE_PASSWORD`

If these signing secrets are incomplete, the workflow fails closed for the affected signed platform.

## Optional downstream publishing secrets

- `HOMEBREW_TAP_TOKEN`
- `CHOCOLATEY_API_KEY`
- `WINGET_TOKEN`
- `SNAPCRAFT_STORE_CREDENTIALS`

If these are absent, `.github/workflows/package-managers.yml` skips the corresponding publishing job instead of blocking installer release creation.

## Recommended release process

1. Update the version consistently:
   - `just set-version X.Y.Z`
2. Run the canonical local validation workflow:
   - `just validate`
   - `just verify-github-actions`
   - `just show-version`
3. Merge the release commit
4. Push the release tag:
   - `git tag vX.Y.Z`
   - `git push origin vX.Y.Z`
5. Monitor the `Release` workflow and verify uploaded assets
6. If package-manager tokens are configured, monitor the `Package Managers` workflow

## Manual dry-run examples

```bash
gh workflow run release.yml -f tag=v0.1.0
gh workflow run package-managers.yml -f tag=v0.1.0
```

## Notes

- Linux CI validates the broadest feature surface for this repo.
- macOS and Windows signed release behavior depends on configured secrets.
- Release publication is intentionally separate from downstream package-manager publishing so installer delivery is not blocked by optional publishing jobs.