# ![](./icons/32x32.png) Haptic Harmony Simulation

[![Rust](https://img.shields.io/badge/rust-1.70+-orange.svg)](https://www.rust-lang.org)
[![Tauri2](https://img.shields.io/badge/tauri-2.0+-purple.svg)](Tauri2)
[![License](https://img.shields.io/badge/License-BSD%203--Clause-blue.svg)](https://opensource.org/licenses/BSD-3-Clause)
[![Build Status](https://img.shields.io/badge/build-passing-brightgreen.svg)]()

> **Advanced Haptic Ring Simulation Toolkit** - A comprehensive Rust-based simulation environment for haptic feedback rings with BLE peripheral emulation.

## Overview

The Haptic Harmony Simulation is a cutting-edge toolkit designed to simulate and test haptic feedback rings in a controlled environment. Built with modern Rust architecture, it provides both CLI and GUI interfaces for comprehensive testing of gesture recognition, haptic feedback patterns, and BLE connectivity.

### Key Features

- **BLE Peripheral Simulation** - Complete Bluetooth Low Energy peripheral emulation
- **Modern Tauri GUI** - Cross-platform desktop application with web technologies
- **CLI Interface** - Command-line tools for automated testing and scripting
- **Gesture Emulation** - Comprehensive gesture recognition simulation
- **Haptic Feedback** - Advanced haptic pattern generation and testing
- **Testing Framework** - Built-in testing tools and mock services
- **Real-time Monitoring** - Live performance metrics and system status

## Quick Start

### Prerequisites

- **Rust 1.70+** - [Install Rust](https://rustup.rs/)
- **Node.js 18+** - For Tauri frontend development
- **Platform-specific dependencies**:
  - **macOS**: Xcode Command Line Tools
  - **Linux**: `libudev-dev`, `libdbus-1-dev`
  - **Windows**: Visual Studio Build Tools

### Installation

```bash
# Clone the repository
git clone https://github.com/haptic-harmony/haptic-harmony-simulation.git
cd haptic-harmony-simulation

# Build the project
cargo build --release

# Run with GUI (default)
cargo run --features tauri-gui -- --mode gui

# Run CLI mode
cargo run -- --mode cli
```

### 🖥️ GUI Mode

Launch the modern Tauri-based GUI application:

```bash
cargo run --features tauri-gui -- --mode gui
```

**Features:**
- Interactive gesture simulation controls
- Real-time BLE peripheral status
- Haptic feedback testing interface
- Performance monitoring dashboard
- Configuration management

### CLI Mode

Use the command-line interface for scripting and automation:

```bash
# Basic CLI mode
cargo run -- --mode cli

# With specific configuration
cargo run -- --mode cli --config custom-config.toml

# Enable verbose logging
RUST_LOG=debug cargo run -- --mode cli
```

## Gesture Controls

### GUI Controls
- **Interactive Buttons** - Click to simulate gestures
- **Slider Controls** - Adjust intensity and duration
- **Real-time Feedback** - Visual and haptic response

### CLI Controls
| Key | Gesture | Description |
|-----|---------|-------------|
| `Enter` | Tap | Single tap gesture |
| `Space` | Hold | Hold gesture (release to complete) |
| `d` | Double Tap | Double tap gesture |
| `s` | Slide | Slide gesture (cycles directions) |
| `t` | Tilt | Tilt gesture (cycles angles) |
| `Ctrl+C` | Exit | Gracefully shutdown simulation |

## Architecture

### Project Structure

```
haptic-harmony-simulation/
├── � src/                          # Core Rust source code
│   ├── 📄 main.rs                   # Application entry point
│   ├── 📄 lib.rs                    # Library exports
│   ├── 📄 tauri_app.rs             # Tauri GUI application
│   ├── 📁 connectivity/             # BLE and networking
│   │   ├── 📄 mod.rs               # Connection management
│   │   ├── 📄 ble_peripheral.rs    # BLE peripheral simulation
│   │   ├── 📄 ble_mock.rs          # BLE mocking utilities
│   │   └── 📄 socket_client.rs     # WebSocket client
│   ├── 📁 ui/                      # User interface modules
│   │   ├── 📄 mod.rs               # UI abstractions
│   │   ├── 📄 cli.rs               # CLI interface
│   │   ├── 📄 pad.rs               # Touch pad simulation
│   │   └── 📄 tilt.rs              # Tilt gesture simulation
│   ├── 📄 emulator.rs              # Core emulation logic
│   ├── 📄 feedback.rs              # Feedback loop management
│   └── 📄 mcp_mock.rs              # MCP protocol simulation
├── 📁 ui/                          # Tauri frontend
│   ├── � src/                     # Web UI source
│   │   ├── � index.html           # Main GUI interface
│   │   ├── 📄 styles.css           # Modern styling
│   │   └── 📄 app.js               # Interactive JavaScript
│   ├── 📁 dist/                    # Built UI assets
│   └── 📄 build.js                 # Build configuration
├── 📁 tests/                       # Test suites
├── 📁 icons/                       # Application icons
├── 📄 Cargo.toml                   # Rust dependencies
├── 📄 tauri.conf.json             # Tauri configuration
├── 📄 build.rs                     # Build script
└── 📄 README.md                    # This file
```

### Core Components

#### BLE Peripheral (`src/connectivity/ble_peripheral.rs`)
- **Complete BLE GATT server implementation**
- **Haptic service with custom characteristics**
- **Battery level simulation**
- **OTA update support**
- **Multi-client connection handling**

#### Gesture Emulation (`src/emulator.rs`)
- **Tap, double-tap, and hold gestures**
- **Slide and swipe recognition**
- **Tilt and rotation detection**
- **Configurable sensitivity and thresholds**

#### Haptic Feedback (`src/feedback.rs`)
- **Pattern generation and playback**
- **Intensity and duration control**
- **Real-time performance monitoring**
- **Custom feedback loops**

## Configuration

### Environment Variables

```bash
# Logging level
export RUST_LOG=debug

# BLE device name
export BLE_DEVICE_NAME="Custom Ring Simulator"

# GUI window settings
export TAURI_WINDOW_WIDTH=1200
export TAURI_WINDOW_HEIGHT=800
```

### Configuration Files

Create `config.toml` for custom settings:

```toml
[ble]
device_name = "Haptic Harmony Ring Simulator"
advertising_interval = 100
connection_timeout = 30

[haptic]
default_intensity = 0.7
max_duration = 5000
enabled = true

[ui]
interface_type = "gui"
enable_colors = true
window_width = 1200
window_height = 800

[logging]
level = "info"
file_output = false
```

## Testing

### Running Tests

```bash
# Run all tests
cargo test

# Run specific test suite
cargo test connectivity

# Run with verbose output
cargo test -- --nocapture

# Run integration tests
cargo test --test integration
```

### Test Coverage

```bash
# Install cargo-tarpaulin
cargo install cargo-tarpaulin

# Generate coverage report
cargo tarpaulin --out Html
```

## API Reference

### Tauri Commands

#### Gesture Simulation
```javascript
// Simulate tap gesture
await invoke('simulate_tap', { intensity: 0.8 });

// Simulate slide gesture
await invoke('simulate_slide', {
  direction: 'up',
  distance: 100
});

// Get system status
const status = await invoke('get_system_status');
```

#### Haptic Control
```javascript
// Trigger haptic feedback
await invoke('trigger_haptic', {
  pattern: 'pulse',
  intensity: 0.7,
  duration: 1000
});

// Get haptic capabilities
const capabilities = await invoke('get_haptic_capabilities');
```

### BLE Characteristics

#### Haptic Service (`12345678-1234-5678-9abc-123456789abc`)
- **Gesture Events** (`...9abe`) - Read/Notify
- **Haptic Commands** (`...9abd`) - Write
- **Battery Level** (`...9abf`) - Read/Notify
- **OTA Update** (`...9ac0`) - Write

## Development

### Building from Source

```bash
# Debug build
cargo build

# Release build
cargo build --release

# Build with specific features
cargo build --features "tauri-gui,linux-ble"

# Build for different targets
cargo build --target x86_64-apple-darwin
```

### Development Mode

```bash
# Run with hot reload (GUI)
cargo run --features tauri-gui -- --mode gui --dev

# Run with debug logging
RUST_LOG=debug cargo run -- --mode cli

# Run specific examples
cargo run --example ble_peripheral
```

### Contributing

1. **Fork the repository**
2. **Create a feature branch** (`git checkout -b feature/amazing-feature`)
3. **Commit your changes** (`git commit -m 'Add amazing feature'`)
4. **Push to the branch** (`git push origin feature/amazing-feature`)
5. **Open a Pull Request**

### Code Style

```bash
# Format code
cargo fmt

# Run clippy lints
cargo clippy

# Check for common issues
cargo audit
```

## Performance

### Benchmarks

```bash
# Run performance benchmarks
cargo bench

# Profile memory usage
cargo run --release --features profiling
```

### System Requirements

- **Memory**: 256MB RAM minimum, 512MB recommended
- **CPU**: Any modern x64 processor
- **Storage**: 100MB for installation
- **Network**: Optional for MCP features

## Troubleshooting

### Common Issues

#### BLE Connection Problems
```bash
# Check BLE adapter status
bluetoothctl show

# Reset BLE stack (Linux)
sudo systemctl restart bluetooth

# Verify permissions (Linux)
sudo usermod -a -G dialout $USER
```

#### GUI Not Starting
```bash
# Verify Tauri dependencies
cargo check --features tauri-gui

# Check system requirements
tauri info

# Run with debug output
RUST_LOG=debug cargo run --features tauri-gui
```

#### Build Failures
```bash
# Clean build cache
cargo clean

# Update dependencies
cargo update

# Check Rust version
rustc --version
```

### Debug Mode

Enable comprehensive debugging:

```bash
export RUST_LOG=haptic_harmony_simulation=debug
export RUST_BACKTRACE=1
cargo run -- --mode cli --verbose
```

## Documentation

### API Documentation

```bash
# Generate and open docs
cargo doc --open

# Generate docs with private items
cargo doc --document-private-items
```

### Examples

See the `examples/` directory for:
- **Basic BLE peripheral setup**
- **Custom gesture recognition**
- **Haptic pattern creation**
- **GUI customization**

## Community

- **GitHub Issues**: [Report bugs and request features](https://github.com/haptic-harmony/haptic-harmony-simulation/issues)
- **Discussions**: [Community discussions](https://github.com/haptic-harmony/haptic-harmony-simulation/discussions)
- **Discord**: [Join our Discord server](https://discord.gg/haptic-harmony)

## License

This project is licensed under the BSD 3-Clause License - see the [LICENSE](LICENSE) file for details.

## Acknowledgments

- **Tauri Team** - For the excellent cross-platform framework
- **btleplug Contributors** - For BLE functionality
- **Rust Community** - For the amazing ecosystem
- **Haptic Research Community** - For inspiration and guidance

---

**Made with ❤️ by the Haptic Harmony Team**
