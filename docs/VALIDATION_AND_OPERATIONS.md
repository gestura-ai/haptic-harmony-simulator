## Validation and operations

### Supported validation matrix

- All platforms:
  - `cargo fmt --all -- --check`
  - `cargo clippy --all-targets --no-default-features --features cli-only -- -D warnings`
  - `cargo test --no-default-features --features cli-only --verbose`
  - `cargo clippy --all-targets --features tauri-gui -- -D warnings`
  - `cargo test --features tauri-gui --verbose`
  - `cargo build --release --features tauri-gui`
- Desktop native BLE surface:
  - `cargo check --features native-ble`
  - `cargo clippy --all-targets --features native-ble -- -D warnings`
  - `cargo test --features native-ble --verbose`

### Why native BLE is now cross-platform

The optional `native-ble` path now routes through a cross-platform GATT/peripheral abstraction that maps to BlueZ on Linux, CoreBluetooth on macOS, and WinRT on Windows. `linux-ble` remains as a Linux compatibility alias, but new validation and release automation should target `native-ble` across desktop platforms.

### macOS same-host validation note

On macOS, native BLE startup can succeed while same-host discovery still remains unreliable when the central and peripheral both run on the same machine. In practice:

- `haptic-harmony-simulator` may reach the native CoreBluetooth advertising state successfully
- a CoreBluetooth/btleplug central on that same Mac may still fail to surface the simulator peripheral

For authoritative macOS validation, prefer cross-device workflows:

1. host the simulator on one Mac and scan from another Mac
2. host the simulator on one Mac and scan from iPhone/iPad
3. treat bundled-app startup on the same Mac as a useful smoke check, not proof of discoverability

### Tagged release gates

Before a GitHub release is created, the tag workflow now requires:

1. Tag/version parity between `Cargo.toml`, `tauri.conf.json`, and the pushed tag.
2. Cross-platform validation on Linux, Windows, and macOS.
3. Signed Windows installers and signed/notarized macOS bundles.
4. Normalized release artifact collection and SHA256 manifest generation.
5. Successful upload of packaged assets to the GitHub release page.

### Operator trust-state controls

- `discovered`: visible but not trusted for privileged commands
- `bonded`: paired but still not trusted for privileged commands
- `enrolled`: trusted for privileged commands
- `attested`: stronger trusted identity
- `revoked`: fail closed for privileged commands until explicitly restored

### Degraded-mode gates

- `low_battery`: blocks privileged commands
- `sensor_fault`: blocks privileged commands
- `firmware_mismatch`: blocks privileged commands
- `operator_blocked`: blocks privileged commands

### Release operator checklist

1. Run `just validate` locally.
2. Run `just verify-github-actions` after workflow changes.
3. Run `just show-version` and confirm Cargo/Tauri/UI versions are in sync.
4. Confirm macOS and Windows signing secrets are configured.
5. Push a version tag matching `Cargo.toml`, `tauri.conf.json`, and `ui/package.json`.
6. Confirm the GitHub release contains installers, CLI archives, and `haptic-harmony-simulation-SHA256SUMS.txt`.