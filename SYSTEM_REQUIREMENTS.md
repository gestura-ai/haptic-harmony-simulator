# System Requirements

This document outlines the system requirements and dependencies for the Haptic Harmony Simulation project.

## Core Requirements

### Rust Toolchain
- **Rust**: 1.70+ (latest stable recommended)
- **Cargo**: Included with Rust installation
- **Target**: Native platform (x86_64 or aarch64)

### Build System
- **Just**: Command runner for build automation
  - Install: `cargo install just` or `brew install just`
  - Version: 1.0+ recommended

## Development Dependencies

### Essential Tools
These tools are required for the primary development workflow:

- **cargo-watch**: File watching for automatic rebuilds
  - Install: `cargo install cargo-watch`
  - Used by: `just dev`, `just dev-cli`
  
- **cargo-audit**: Security vulnerability scanning
  - Install: `cargo install cargo-audit`
  - Used by: `just audit`, `just release-prep`

### Optional Tools
These tools enhance the development experience but are not required:

- **Node.js**: 18+ (for frontend development)
  - Install: [nodejs.org](https://nodejs.org/) or `brew install node`
  - Used by: Frontend build commands (when `ui/` directory exists)
  
- **NPM**: Included with Node.js
  - Used by: Frontend package management

## Platform-Specific Requirements

### macOS
- **Xcode Command Line Tools**: Required for native compilation
  - Install: `xcode-select --install`
- **Homebrew**: Recommended for package management
  - Install: [brew.sh](https://brew.sh/)
- **dbus and pkg-config**: Required for full BLE functionality
  - Install: `brew install dbus pkg-config`
  - Alternative: Use `just setup-macos` for automated installation

### Linux
- **Build essentials**: GCC, make, etc.
  - Ubuntu/Debian: `sudo apt install build-essential`
  - RHEL/CentOS: `sudo yum groupinstall "Development Tools"`

### Windows
- **Visual Studio Build Tools**: Required for MSVC toolchain
  - Install: Visual Studio Installer → Build Tools for Visual Studio
- **Git for Windows**: Recommended for shell compatibility

## Feature-Specific Dependencies

### BLE Functionality
The project includes Bluetooth Low Energy simulation features that may require additional system libraries:

- **Linux**: `libdbus-1-dev`
  - Ubuntu/Debian: `sudo apt install libdbus-1-dev`
  - RHEL/CentOS: `sudo yum install dbus-devel`

- **macOS**: No additional dependencies (uses system frameworks)

- **Windows**: No additional dependencies (uses Windows APIs)

**Note**: BLE features are mocked for simulation purposes and don't require actual Bluetooth hardware.

### GUI Features (Tauri)
When the GUI features are enabled, additional system dependencies may be required:

- **Linux**: WebKit and GTK development libraries
  - Ubuntu/Debian: `sudo apt install libwebkit2gtk-4.0-dev libgtk-3-dev libayatana-appindicator3-dev librsvg2-dev`

- **macOS**: No additional dependencies (uses system WebView)

- **Windows**: No additional dependencies (uses system WebView2)

## Quick Setup

### Automated Setup
Run the setup command to install development dependencies:
```bash
just setup
```

### Manual Setup
1. Install Rust: [rustup.rs](https://rustup.rs/)
2. Install Just: `cargo install just`
3. Install development tools: `cargo install cargo-watch cargo-audit`
4. (Optional) Install Node.js for frontend development

### Verification
Check your system setup:
```bash
just check-system
```

Test critical workflows:
```bash
just test-workflows
```

## Troubleshooting

### Common Issues

#### "cargo-watch not found"
- **Solution**: Run `just setup` or `cargo install cargo-watch`

#### "dbus library not found" (Linux/macOS)
- **Linux Solution**: Install dbus development libraries
  - **Ubuntu/Debian**: `sudo apt install libdbus-1-dev pkg-config`
  - **RHEL/CentOS**: `sudo yum install dbus-devel pkgconf-pkg-config`
- **macOS Solution**: Install dbus via Homebrew
  - **Manual**: `brew install dbus pkg-config`
  - **Automated**: `just setup-macos`
- **Note**: This affects `just test`, `just lint`, and `just check` commands
- **Workaround**: Use `just build-cli` and `just dev` which work without dbus

#### "Node.js not found"
- **Solution**: Install Node.js or skip frontend commands
- **Note**: Frontend commands gracefully handle missing Node.js

#### Build failures on Windows
- **Solution**: Ensure Visual Studio Build Tools are installed
- **Alternative**: Use WSL2 with Linux setup

### Performance Notes

#### Compilation Times
- **First build**: 5-10 minutes (downloads and compiles dependencies)
- **Incremental builds**: 10-30 seconds
- **Watch mode**: Near-instant for small changes

#### Resource Usage
- **RAM**: 2GB+ recommended for compilation
- **Disk**: 1GB+ for target directory and dependencies
- **CPU**: Multi-core recommended for parallel compilation

## Supported Platforms

### Tier 1 (Fully Supported)
- **macOS**: x86_64, aarch64 (Apple Silicon)
- **Linux**: x86_64 (Ubuntu 20.04+, similar distributions)
- **Windows**: x86_64 (Windows 10+)

### Tier 2 (Best Effort)
- **Linux**: aarch64 (ARM64)
- **Other Unix-like systems**: May work with manual dependency installation

## Version Compatibility

### Minimum Versions
- **Rust**: 1.70.0
- **Just**: 1.0.0
- **Node.js**: 18.0.0 (if using frontend features)

### Tested Versions
- **Rust**: 1.75.0
- **Just**: 1.16.0
- **Node.js**: 20.10.0
- **cargo-watch**: 8.5.3
- **cargo-audit**: 0.21.2

## Security Considerations

### Development Dependencies
- All Rust dependencies are audited with `cargo audit`
- Node.js dependencies (when present) should be regularly updated
- Use `just audit` to check for known vulnerabilities

### Network Access
- The simulation may create local network connections for testing
- No external network access is required for core functionality
- BLE simulation is entirely local (no actual Bluetooth required)
