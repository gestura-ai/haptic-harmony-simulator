# Release setup

## Tagged release flow

The simulator now publishes releases from `.github/workflows/release.yml` only when:

- a tag matching `v*` is pushed, or
- the workflow is launched manually with a matching tag input.

The workflow is intentionally **validate first, publish last**:

1. Verify the tag matches `Cargo.toml` and `tauri.conf.json`.
2. Run cross-platform validation on Linux, Windows, and macOS.
3. Build signed platform bundles.
4. Normalize bundle names and upload them as workflow artifacts.
5. Create or update the GitHub release and attach all assets plus SHA256 checksums.

## Release assets

Tagged releases upload normalized assets directly to the GitHub release page:

- `haptic-harmony-simulation-macos-x64.dmg`
- `haptic-harmony-simulation-macos-arm64.dmg`
- `haptic-harmony-simulation-windows-x64.exe`
- `haptic-harmony-simulation-windows-x64.msi`
- `haptic-harmony-simulation-linux-x64.AppImage`
- `haptic-harmony-simulation-linux-x64.deb`
- `haptic-harmony-simulation-linux-x64.rpm` when available
- CLI archives per platform/architecture
- `haptic-harmony-simulation-SHA256SUMS.txt`

## Required GitHub secrets

### macOS signing and notarization

- `APPLE_CERTIFICATE`
- `APPLE_CERTIFICATE_PASSWORD`
- `APPLE_SIGNING_IDENTITY`
- `APPLE_ID`
- `APPLE_PASSWORD`
- `APPLE_TEAM_ID`

Tagged releases **fail closed** for macOS packaging if any of these are missing.

### Windows code signing

- `WINDOWS_CERTIFICATE`
- `WINDOWS_CERTIFICATE_PASSWORD`

Tagged releases **fail closed** for Windows packaging if either secret is missing.

### Optional downstream publishing secrets

- `HOMEBREW_TAP_TOKEN`
- `CHOCOLATEY_API_KEY`
- `WINGET_TOKEN`
- `SNAPCRAFT_STORE_CREDENTIALS`

If these are absent, `.github/workflows/package-managers.yml` skips the corresponding publish job instead of failing the release pipeline.

## Local validation before tagging

Run the same supported local surface before pushing a tag:

- `just validate`
- `just verify-github-actions`
- `just show-version`

On Linux, `validate-production.sh` also runs `--all-features`. On macOS and Windows, the Linux-only BLE feature surface remains excluded locally and is validated in Linux CI.

## Tagging a release

just set-version 0.1.0
git tag v0.1.0
git push origin v0.1.0

After the tag build goes green, the GitHub release page becomes the source of truth for signed installers and packaged artifacts.