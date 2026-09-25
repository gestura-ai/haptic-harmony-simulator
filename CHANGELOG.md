# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Added
- `rotate` gesture (device-truth kind, protocol v0.3.0): new `GestureType::Rotate { direction }` in the emulator model, projected to `SemanticGesture::Rotate` on the wire and driven through the firmware engine (`UiGesture::RotateCw/Ccw`) under `device-core`. Reachable from the CLI (`↑`/`r`, `↓`/`R`), the Tauri `trigger_gesture` command (`rotate-cw`, `rotate-ccw`) and the web UI (buttons + arrow keys). Before this, no user input could produce a rotate.
- Directional swipes from every input: the CLI `s` key now cycles Up → Right → Down → Left, `←`/`→` emit the horizontal slides that ARE the device-truth swipes, and `trigger_gesture` accepts `slide-up|down|left|right` / `swipe-left|right`. Before this, every input path hard-coded `SlideDirection::Up`.
- Legacy-parts parser accepts `swipe` and `rotate`.
- Initial project structure and documentation

### Fixed
- Build truth for outsiders: the README documents that the default `device-core` feature compiles the (private) `haptic-basic-firmware` sources and how to point `GESTURA_FIRMWARE_DIR` at a checkout or build with `--no-default-features --features cli-only`; the Dockerfile builds that way on a Rust image new enough for edition 2024 (`rust:1.75-slim` could not build this crate at all).
- README/manifest: license badge and section say GPSL-1.1 (relicensed 2026-07-10, badge still said BSD-3-Clause); clone/issue URLs and `Cargo.toml#repository` point at `gestura-ai/haptic-harmony-simulator` instead of a non-existent repository; Rust requirement is 1.85+ (`rust-version` added), not 1.70+.

## [0.1.0] - 2024-08-11

### Added
- **BLE Peripheral Simulation** - Complete Bluetooth Low Energy peripheral emulation
- **Modern Tauri GUI** - Cross-platform desktop application with web technologies
- **CLI Interface** - Command-line tools for automated testing and scripting
- **Gesture Emulation** - Comprehensive gesture recognition simulation
  - Tap, double-tap, and hold gestures
  - Slide and swipe recognition
  - Tilt and rotation detection
  - Configurable sensitivity and thresholds
- **Haptic Feedback** - Advanced haptic pattern generation and testing
  - Pattern generation and playback
  - Intensity and duration control
  - Real-time performance monitoring
  - Custom feedback loops\
- **Testing Framework** - Built-in testing tools and mock services
- **Real-time Monitoring** - Live performance metrics and system status

### Technical Features
- **Modular Rust Architecture** - Clean separation of concerns with trait-based design
- **Async/Await Support** - Full async support with Tokio runtime
- **Cross-platform Compatibility** - Support for macOS, Linux, and Windows
- **Feature-gated Compilation** - Optional GUI and CLI modes
- **Comprehensive Error Handling** - Result/Option types throughout
- **Real-time Feedback Loop** - Sub-50ms response times
- **Mock BLE Adapter** - Complete BLE GATT server simulation
- **WebSocket Client** - Real-time communication with external services

### BLE Implementation
- **Haptic Service** 
  - Gesture Events characteristic (Read/Notify)
  - Haptic Commands characteristic (Write)
  - Battery Level characteristic (Read/Notify)
  - OTA Update characteristic (Write)
- **Multi-client Connection Support**
- **Battery Level Simulation**
- **Device Information Service**
- **Advertising and Discovery**

### GUI Features
- **Modern Tauri v2.0.0-beta Integration**
- **Interactive Gesture Controls**
- **Real-time System Status Display**
- **Haptic Feedback Testing Interface**
- **Performance Monitoring Dashboard**
- **Configuration Management**
- **Cross-platform Window Management**

### CLI Features
- **Interactive Terminal Interface**
- **Keyboard Gesture Controls**
- **Real-time Feedback Display**
- **Verbose Logging Options**
- **Configuration File Support**
- **Command-line Arguments**

### Documentation
- **Comprehensive README** - Detailed setup and usage instructions
- **API Documentation** - Complete Rust API docs
- **Contributing Guidelines** - Development and contribution process
- **Architecture Documentation** - System design and component overview
- **Troubleshooting Guide** - Common issues and solutions

### Development Tools
- **Cargo Workspace** - Organized project structure
- **Test Suite** - Unit, integration, and end-to-end tests
- **Code Coverage** - Tarpaulin integration
- **Linting** - Clippy integration
- **Formatting** - Rustfmt configuration
- **CI/CD Ready** - GitHub Actions compatible

### Configuration
- **Environment Variables** - Runtime configuration
- **TOML Configuration Files** - Structured settings
- **Feature Flags** - Compile-time options
- **Logging Configuration** - Structured logging with tracing

### Performance
- **Memory Efficient** - Minimal resource usage
- **Low Latency** - Real-time gesture processing
- **Concurrent Processing** - Multi-threaded architecture
- **Resource Monitoring** - Built-in performance metrics

### Security
- **Safe Rust** - Memory safety guarantees
- **Input Validation** - Comprehensive input sanitization
- **Error Handling** - Graceful failure modes
- **Secure Defaults** - Conservative configuration

### Compatibility
- **Rust 1.70+** - Modern Rust features
- **Tauri v2.0.0-beta** - Latest Tauri framework
- **Cross-platform** - macOS, Linux, Windows support
- **Multiple Architectures** - x86_64, ARM64 support

## [0.0.1] - 2024-08-11

### Added
- Initial project setup
- Basic Rust project structure
- License and initial documentation

---

## Release Notes

### Version 0.1.0 - "Foundation Release"

This is the initial release of the Haptic Harmony Simulation toolkit. It provides a complete foundation for haptic ring simulation with both GUI and CLI interfaces.

**Key Highlights:**
- Complete BLE peripheral emulation
- Modern Tauri-based GUI application
- Comprehensive gesture simulation
- Real-time haptic feedback
- MCP protocol integration
- Cross-platform compatibility

**Getting Started:**
```bash
git clone https://github.com/haptic-harmony/haptic-harmony-simulation.git
cd haptic-harmony-simulation
cargo run --features tauri-gui -- --mode gui
```

**Breaking Changes:**
- None (initial release)

**Migration Guide:**
- None (initial release)

**Known Issues:**
- None reported

**Contributors:**
- Haptic Harmony Team

---

For more information about releases, see the [GitHub Releases](https://github.com/haptic-harmony/haptic-harmony-simulation/releases) page.
