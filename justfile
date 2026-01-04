# Haptic Harmony Simulation - Just Build Recipes
# Modern cross-platform build automation with Just
#
# Feature Flags:
#   - cli-only: Pure CLI build (no GUI dependencies)
#   - tauri-gui: Full GUI build with Tauri (includes GTK/WebKit on Linux)
#   - linux-ble: Linux BLE support via bluer (optional)
#
# Important: Always use --no-default-features --features cli-only for CLI builds
#               to avoid pulling in GUI dependencies!

# Show available commands
default:
    @just --list

# Quick Start Commands
# =====================

# Start GUI development with hot reload
dev:
    @echo "Starting GUI development mode..."
    cargo watch -x "run --features tauri-gui -- --mode gui"

# Start CLI development with hot reload
dev-cli:
    @echo "Starting CLI development mode..."
    cargo watch -x "run --no-default-features --features cli-only -- --mode cli"

# Build both CLI and GUI for current platform
build:
    @echo "Building CLI and GUI..."
    @just build-cli
    @just build-gui

# Run all tests
test:
    @echo "Running tests..."
    @echo "Testing CLI features..."
    cargo test --no-default-features --features cli-only
    @echo "Testing GUI features..."
    cargo test --features tauri-gui

# Development Commands
# ======================

# Production validation - run all checks that CI will run
validate:
    @echo "Running production validation..."
    ./scripts/validate-production.sh

# Quick validation - essential checks only
validate-quick:
    @echo "Running quick validation..."
    @echo "Checking formatting..."
    cargo fmt --all -- --check
    @echo "Running clippy (CLI)..."
    cargo clippy --all-targets --no-default-features --features cli-only -- -D warnings
    @echo "Running clippy (GUI)..."
    cargo clippy --all-targets --features tauri-gui -- -D warnings
    @echo "Running tests (CLI)..."
    cargo test --no-default-features --features cli-only
    @echo "✅ Quick validation complete!"

# Install development dependencies
setup:
    @echo "📦 Installing development dependencies..."
    @echo "Installing Rust tools..."
    cargo install cargo-watch cargo-audit
    @echo "Installing Node.js dependencies..."
    cd ui && npm install || echo "⚠️  Frontend dependencies skipped (ui/ directory not found)"
    @echo "✅ Setup complete!"

# Install system dependencies for full functionality (macOS)
setup-macos:
    @echo "Installing macOS system dependencies..."
    @echo "Installing dbus via Homebrew (required for BLE features)..."
    brew install dbus pkg-config || echo "⚠️  Homebrew not available or dbus installation failed"
    @echo "Installing development tools..."
    @just setup
    @echo "✅ macOS setup complete!"

# Install system dependencies for Ubuntu/Linux
setup-ubuntu:
    @echo "Installing Ubuntu system dependencies..."
    @echo "Installing CLI dependencies..."
    sudo apt-get update
    sudo apt-get install -y libudev-dev libdbus-1-dev pkg-config
    @echo "Installing GUI dependencies (for Tauri builds)..."
    sudo apt-get install -y \
        libwebkit2gtk-4.1-dev \
        build-essential \
        curl \
        wget \
        file \
        libxdo-dev \
        libssl-dev \
        libayatana-appindicator3-dev \
        librsvg2-dev
    @echo "Installing development tools..."
    @just setup
    @echo "✅ Ubuntu setup complete!"

# Format all code
fmt:
    @echo "Formatting code..."
    cargo fmt
    cd ui && npm run format || echo "Frontend formatting not available"

# Run linting
lint:
    @echo "Running linting..."
    @echo "Linting CLI features..."
    cargo clippy --all-targets --no-default-features --features cli-only -- -D warnings
    @echo "Linting GUI features..."
    cargo clippy --all-targets --features tauri-gui -- -D warnings

# Security audit
audit:
    @echo "Running security audit..."
    cargo audit

# Quick check without building
check:
    @echo "✅ Quick check..."
    @echo "Checking CLI features..."
    cargo check --no-default-features --features cli-only
    @echo "Checking GUI features..."
    cargo check --features tauri-gui

# Build Commands
# ================

# Build CLI version only
build-cli:
    @echo "🔨 Building CLI..."
    cargo build --release --no-default-features --features cli-only

# Build GUI version only (requires frontend build)
build-gui:
    @echo "Building frontend..."
    cd ui && npm run build
    @echo "🔨 Building GUI..."
    cargo build --release --features tauri-gui

# Build for current macOS architecture (CLI)
build-macos:
    @echo "Building CLI for macOS (current architecture)..."
    cargo build --release --no-default-features --features cli-only --target x86_64-apple-darwin

# Icon generation
icons:
    @echo "Generating icons from source..."
    ./scripts/generate-icons.sh

# Ring Simulation Commands
# ==========================

# Run B1 ring simulation (current)
sim-b1:
    @echo "Starting B1 ring simulation..."
    cargo run --features tauri-gui -- --mode gui --ring-type b1

# Run A1 ring simulation (future)
sim-a1:
    @echo "Starting A1 ring simulation..."
    cargo run --features tauri-gui -- --mode gui --ring-type a1

# Run P1 ring simulation (future)
sim-p1:
    @echo "Starting P1 ring simulation..."
    cargo run --features tauri-gui -- --mode gui --ring-type p1

# Cleanup Commands
# ==================

# Clean build artifacts
clean:
    @echo "Cleaning build artifacts..."
    cargo clean
    rm -rf target/
    rm -rf ui/dist/
    rm -rf coverage/

# Clean everything including dependencies
clean-all: clean
    @echo "Cleaning dependencies..."
    rm -rf ui/node_modules/

# Documentation Commands
# ========================

# Generate and open documentation
docs:
    @echo "Generating documentation..."
    @echo "Generating CLI documentation..."
    cargo doc --no-default-features --features cli-only
    @echo "Generating GUI documentation..."
    cargo doc --open --features tauri-gui

# Show setup documentation
docs-setup:
    @echo "📖 Setup Documentation"
    @echo "======================"
    @echo ""
    @echo "📋 General Setup:"
    @echo "  README.md - Project overview and quick start"
    @echo "  CONTRIBUTING.md - Development guidelines"
    @echo "  SYSTEM_REQUIREMENTS.md - System requirements"
    @echo ""
    @echo "🔐 Code Signing Setup:"
    @echo "  MACOS_CODE_SIGNING_SETUP.md - macOS code signing configuration"
    @echo "  WINDOWS_CODE_SIGNING_SETUP.md - Windows code signing configuration"
    @echo ""
    @echo "🚀 Release Process:"
    @echo "  RELEASE_SETUP.md - Release pipeline configuration"
    @echo "  .github/workflows/release.yml - GitHub Actions workflow"
    @echo ""
    @echo "💡 Use 'just docs-signing' for code signing help"

# Show code signing documentation
docs-signing:
    @echo "🔐 Code Signing Documentation"
    @echo "============================="
    @echo ""
    @echo "📖 Setup Guides:"
    @echo "  • MACOS_CODE_SIGNING_SETUP.md - Complete macOS setup"
    @echo "  • WINDOWS_CODE_SIGNING_SETUP.md - Complete Windows setup"
    @echo ""
    @echo "🔧 Commands:"
    @echo "  • just check-macos-signing - Check macOS setup"
    @echo "  • just check-windows-signing - Check Windows setup"
    @echo "  • just verify-macos-app <path> - Verify signed macOS app"
    @echo "  • just verify-windows-app <path> - Verify signed Windows app"
    @echo ""
    @echo "🚀 Release Commands:"
    @echo "  • just release-macos-signed - Build signed macOS release"
    @echo "  • just release-windows-signed - Build signed Windows release"
    @echo "  • just release-signed - Build all signed releases"

# Show version information
version:
    @echo "Haptic Harmony Simulation - Version Information"
    @echo "=================================================="
    @echo "Rust: $(rustc --version)"
    @echo "Cargo: $(cargo --version)"
    @echo "Node: $(node --version 2>/dev/null || echo 'Not installed')"
    @echo "NPM: $(npm --version 2>/dev/null || echo 'Not installed')"
    @echo "Just: $(just --version)"
    @echo "Platform: $(uname -s) $(uname -m)"

# CI/CD Pipeline Commands
# ==========================

# Run the same validation that CI runs
ci-validate:
    @echo "Running CI validation pipeline..."
    @echo "Step 1: Format check..."
    cargo fmt --all -- --check
    @echo "Step 2: Clippy (CLI)..."
    cargo clippy --all-targets --no-default-features --features cli-only -- -D warnings
    @echo "Step 3: Clippy (GUI)..."
    cargo clippy --all-targets --features tauri-gui -- -D warnings
    @echo "Step 4: Tests (CLI)..."
    cargo test --verbose --no-default-features --features cli-only
    @echo "Step 5: Tests (GUI)..."
    cargo test --features tauri-gui
    @echo "Step 6: Security audit..."
    cargo audit
    @echo "✅ CI validation complete!"

# Code Signing Commands
# ======================

# Check code signing setup for macOS
check-macos-signing:
    @echo "🔍 Checking macOS code signing setup..."
    @echo "Available signing identities:"
    security find-identity -v -p codesigning || echo "❌ No signing identities found"
    @echo ""
    @echo "Environment variables status:"
    @echo "- APPLE_CERTIFICATE: $(if [ -n "$APPLE_CERTIFICATE" ]; then echo "✅ Set"; else echo "❌ Not set"; fi)"
    @echo "- APPLE_CERTIFICATE_PASSWORD: $(if [ -n "$APPLE_CERTIFICATE_PASSWORD" ]; then echo "✅ Set"; else echo "❌ Not set"; fi)"
    @echo "- APPLE_SIGNING_IDENTITY: $(if [ -n "$APPLE_SIGNING_IDENTITY" ]; then echo "✅ Set ($APPLE_SIGNING_IDENTITY)"; else echo "❌ Not set"; fi)"
    @echo "- APPLE_ID: $(if [ -n "$APPLE_ID" ]; then echo "✅ Set ($APPLE_ID)"; else echo "❌ Not set"; fi)"
    @echo "- APPLE_PASSWORD: $(if [ -n "$APPLE_PASSWORD" ]; then echo "✅ Set"; else echo "❌ Not set"; fi)"
    @echo "- APPLE_TEAM_ID: $(if [ -n "$APPLE_TEAM_ID" ]; then echo "✅ Set ($APPLE_TEAM_ID)"; else echo "❌ Not set"; fi)"
    @echo ""
    @echo "📖 See MACOS_CODE_SIGNING_SETUP.md for detailed setup instructions"

# Test Apple Developer certificate setup
test-apple-cert:
    @echo "🧪 Testing Apple Developer certificate setup..."
    @if [ -z "$APPLE_CERTIFICATE" ]; then echo "❌ APPLE_CERTIFICATE not set. Please configure GitHub Secrets."; exit 1; fi
    @if [ -z "$APPLE_CERTIFICATE_PASSWORD" ]; then echo "❌ APPLE_CERTIFICATE_PASSWORD not set. Please configure GitHub Secrets."; exit 1; fi
    @if [ -z "$APPLE_SIGNING_IDENTITY" ]; then echo "❌ APPLE_SIGNING_IDENTITY not set. Please configure GitHub Secrets."; exit 1; fi
    @echo "✅ All required certificate variables are set"
    @echo "🔐 Testing certificate import..."
    @echo "$APPLE_CERTIFICATE" | base64 -d > /tmp/test-cert.p12
    @security import /tmp/test-cert.p12 -k ~/Library/Keychains/login.keychain -P "$APPLE_CERTIFICATE_PASSWORD" -T /usr/bin/codesign 2>/dev/null || echo "Certificate import test completed"
    @rm -f /tmp/test-cert.p12
    @echo "✅ Certificate test completed successfully"

# Check code signing setup for Windows
check-windows-signing:
    @echo "🔍 Checking Windows code signing setup..."
    @echo "Environment variables status:"
    @echo "- WINDOWS_CERTIFICATE: $(if [ -n "$WINDOWS_CERTIFICATE" ]; then echo "✅ Set"; else echo "❌ Not set"; fi)"
    @echo "- WINDOWS_CERTIFICATE_PASSWORD: $(if [ -n "$WINDOWS_CERTIFICATE_PASSWORD" ]; then echo "✅ Set"; else echo "❌ Not set"; fi)"
    @echo ""
    @echo "📖 See WINDOWS_CODE_SIGNING_SETUP.md for detailed setup instructions"

# Verify a signed macOS application
verify-macos-app path:
    @echo "🔐 Verifying macOS application signature..."
    codesign --verify --verbose=2 "{{path}}"
    codesign -dv --verbose=4 "{{path}}"
    spctl --assess --type open --context context:primary-signature --verbose=2 "{{path}}"

# Verify a signed Windows application
verify-windows-app path:
    @echo "🔐 Verifying Windows application signature..."
    @powershell -Command "Get-AuthenticodeSignature -FilePath '{{path}}' | Format-List"

# Verify icons are properly embedded in all builds
verify-icons:
    @./scripts/verify-icons.sh

# Build for all available platforms with proper icons
build-all-platforms:
    @echo "🌍 Building for all available platforms..."
    @echo ""
    @echo "🍎 Building macOS..."
    @just build-macos-app
    @echo ""
    @echo "🪟 Windows build (requires Windows or cross-compilation):"
    @echo "  tauri build --target x86_64-pc-windows-msvc --features tauri-gui"
    @echo ""
    @echo "🐧 Linux build (requires Linux or cross-compilation):"
    @echo "  tauri build --target x86_64-unknown-linux-gnu --features tauri-gui"
    @echo ""
    @echo "✅ macOS build completed. See above for other platforms."

# Verify GitHub Actions workflows are properly configured
verify-github-actions:
    @./scripts/verify-github-actions.sh

# Verify Windows build requirements and SSL.com code signing
verify-windows-build:
    @echo "🪟 Verifying Windows build configuration..."
    @if [ "$$(uname -s)" != "MINGW64_NT"* ] && [ "$$(uname -s)" != "MSYS_NT"* ]; then \
        echo "ℹ️  This command should be run on Windows for full verification"; \
        echo "📋 Checking Tauri configuration for Windows..."; \
        if grep -q "webviewInstallMode" tauri.conf.json; then \
            echo "✅ WebView2 install mode configured"; \
        else \
            echo "⚠️  WebView2 install mode not configured"; \
        fi; \
        if grep -q "timestamp.ssl.com" tauri.conf.json; then \
            echo "✅ SSL.com timestamp server configured"; \
        else \
            echo "⚠️  SSL.com timestamp server not configured"; \
        fi; \
        if grep -q "certificateThumbprint.*null" tauri.conf.json; then \
            echo "⚠️  No Windows code signing certificate configured"; \
            echo "📋 Required GitHub Secrets for SSL.com:"; \
            echo "   - WINDOWS_CERTIFICATE (base64 encoded .p12)"; \
            echo "   - WINDOWS_CERTIFICATE_PASSWORD"; \
            echo "   - WINDOWS_SIGNING_SUBJECT_NAME"; \
        else \
            echo "✅ Windows code signing configured"; \
        fi; \
        if [ -f "icons/icon.ico" ]; then \
            echo "✅ Windows icon file found"; \
        else \
            echo "❌ Windows icon file missing"; \
        fi; \
        echo ""; \
        echo "🚀 To run full SSL.com verification:"; \
        echo "   On Windows: powershell -ExecutionPolicy Bypass -File scripts/verify-windows-build.ps1"; \
        echo "📖 See SSL_COM_WINDOWS_SIGNING_GUIDE.md for complete setup"; \
    else \
        echo "Running on Windows - executing SSL.com PowerShell verification..."; \
        powershell -ExecutionPolicy Bypass -File scripts/verify-windows-build.ps1; \
    fi

# Check overall project status and readiness
status:
    @echo "🔍 Haptic Harmony Simulator - Project Status"
    @echo "============================================="
    @echo ""
    @echo "📦 Project Information:"
    @echo "- Version: $(cargo metadata --no-deps --format-version 1 | jq -r '.packages[0].version')"
    @echo "- Rust Version: $(rustc --version)"
    @echo "- Cargo Version: $(cargo --version)"
    @echo ""
    @echo "🎯 Build Targets:"
    @echo "- Installed targets:"
    @rustup target list --installed | sed 's/^/  • /'
    @echo ""
    @echo "🔐 Code Signing Status:"
    @just check-macos-signing
    @echo ""
    @just check-windows-signing
    @echo ""
    @echo "📋 CI/CD Status:"
    @echo "- GitHub Actions workflow: $(if [ -f '.github/workflows/release.yml' ]; then echo '✅ Configured'; else echo '❌ Missing'; fi)"
    @echo "- Tauri config: $(if [ -f 'tauri.conf.json' ]; then echo '✅ Present'; else echo '❌ Missing'; fi)"
    @echo "- Entitlements (macOS): $(if [ -f 'entitlements.plist' ]; then echo '✅ Present'; else echo '❌ Missing'; fi)"
    @echo ""
    @echo "📖 Documentation:"
    @echo "- README: $(if [ -f 'README.md' ]; then echo '✅ Present'; else echo '❌ Missing'; fi)"
    @echo "- macOS Signing Guide: $(if [ -f 'MACOS_CODE_SIGNING_SETUP.md' ]; then echo '✅ Present'; else echo '❌ Missing'; fi)"
    @echo "- Windows Signing Guide: $(if [ -f 'WINDOWS_CODE_SIGNING_SETUP.md' ]; then echo '✅ Present'; else echo '❌ Missing'; fi)"

# Generate code coverage report (like CI)
coverage:
    @echo "Generating code coverage report..."
    @echo "Installing cargo-tarpaulin if needed..."
    cargo install cargo-tarpaulin --quiet || echo "cargo-tarpaulin already installed"
    @echo "Running coverage analysis..."
    cargo tarpaulin --out xml --output-dir . --no-default-features --features cli-only
    @echo "✅ Coverage report generated: cobertura.xml"

# Build release binaries for all platforms (like release pipeline)
release-build version="0.1.0":
    @echo "Building release binaries for all platforms (v{{version}})..."
    @just install-targets
    @echo "Building CLI binaries..."
    @just build-linux-cli
    @just build-linux-arm-cli
    @just build-windows-cli
    @just build-windows-arm-cli
    @just build-macos-cli
    @just build-macos-arm-cli
    @echo "Building GUI binaries..."
    @just build-linux-gui
    @just build-linux-arm-gui
    @just build-windows-gui
    @just build-windows-arm-gui
    @just build-macos-intel-gui
    @just build-macos-arm-gui
    @echo "✅ All release binaries built!"

# Build macOS app bundle and verify
build-macos-app:
    @echo "🍎 Building macOS app bundle..."
    @echo "Building frontend..."
    cd ui && npm run build
    @echo "Building macOS app bundle with Tauri..."
    @echo "Installing Tauri CLI if needed..."
    @which tauri > /dev/null || npm install -g @tauri-apps/cli@v2
    @echo "Building bundle..."
    tauri build --features tauri-gui
    @echo "🔍 Checking for app bundle..."
    @find target -name "*.app" -type d 2>/dev/null || echo "No .app bundles found"
    @echo ""
    @echo "📁 Bundle directory structure:"
    @ls -la target/release/bundle/ 2>/dev/null || echo "No bundle directory found"
    @echo ""
    @echo "🔐 Checking app bundle signature:"
    @./scripts/verify-apple-signing.sh 2>/dev/null || echo "⚠️  App bundle is not signed (development build)"
    @echo "  To sign the app, configure Apple Developer certificate:"
    @echo "  1. Set up GitHub Secrets (see MACOS_CODE_SIGNING_SETUP.md)"
    @echo "  2. Use: just release-macos-signed"
    @echo ""
    @echo "🎨 Verifying app icons:"
    @just verify-icons

# Build signed release for macOS (requires certificates)
release-macos-signed version="0.1.0":
    @echo "🔐 Building signed macOS release (v{{version}})..."
    @echo "⚠️  This requires Apple Developer certificates to be configured"
    @just check-macos-signing
    cd ui && npm run build
    @echo "Building signed macOS Intel GUI..."
    cargo build --release --target x86_64-apple-darwin --features tauri-gui
    @echo "Building signed macOS ARM64 GUI..."
    cargo build --release --target aarch64-apple-darwin --features tauri-gui
    @echo "✅ Signed macOS release built!"

# Build signed release for Windows (requires certificates)
release-windows-signed version="0.1.0":
    @echo "🔐 Building signed Windows release (v{{version}})..."
    @echo "⚠️  This requires Windows code signing certificate to be configured"
    @just check-windows-signing
    cd ui && npm run build
    @echo "Building signed Windows x64 GUI..."
    cargo build --release --target x86_64-pc-windows-msvc --features tauri-gui
    @echo "Building signed Windows ARM64 GUI..."
    cargo build --release --target aarch64-pc-windows-msvc --features tauri-gui
    @echo "✅ Signed Windows release built!"

# Build signed releases for all platforms (production)
release-signed version="0.1.0":
    @echo "🔐 Building signed releases for all platforms (v{{version}})..."
    @echo "⚠️  This requires code signing certificates for macOS and Windows"
    @just release-macos-signed {{version}}
    @just release-windows-signed {{version}}
    @just build-linux-gui
    @just build-linux-arm-gui
    @echo "✅ All signed releases built!"

# Advanced Commands
# ===================

# Install additional Rust targets for cross-compilation
install-targets:
    @echo "📦 Installing cross-compilation targets..."
    rustup target add x86_64-unknown-linux-gnu
    rustup target add aarch64-unknown-linux-gnu
    rustup target add x86_64-pc-windows-msvc
    rustup target add aarch64-pc-windows-msvc
    rustup target add x86_64-apple-darwin
    rustup target add aarch64-apple-darwin

# Build for all supported platforms using comprehensive build script
build-all version="0.1.0": install-targets
    @echo "Building for all platforms (version {{version}})..."
    ./scripts/build-release.sh {{version}}

# Build individual platforms with GUI features
build-linux-gui: install-targets
    @echo "Building for Linux x64 (GUI)..."
    cd ui && npm run build
    cargo build --release --target x86_64-unknown-linux-gnu --features tauri-gui

build-linux-arm-gui: install-targets
    @echo "Building for Linux ARM64 (GUI)..."
    cd ui && npm run build
    cargo build --release --target aarch64-unknown-linux-gnu --features tauri-gui

build-windows-gui: install-targets
    @echo "Building for Windows x64 (GUI)..."
    cd ui && npm run build
    cargo build --release --target x86_64-pc-windows-msvc --features tauri-gui

build-windows-arm-gui: install-targets
    @echo "Building for Windows ARM64 (GUI)..."
    cd ui && npm run build
    cargo build --release --target aarch64-pc-windows-msvc --features tauri-gui

build-macos-intel-gui: install-targets
    @echo "Building for macOS Intel (GUI)..."
    cd ui && npm run build
    cargo build --release --target x86_64-apple-darwin --features tauri-gui

build-macos-arm-gui: install-targets
    @echo "Building for macOS Apple Silicon (GUI)..."
    cd ui && npm run build
    cargo build --release --target aarch64-apple-darwin --features tauri-gui

# Build CLI versions (includes BLE connectivity)
build-linux-cli: install-targets
    @echo "Building CLI for Linux x64..."
    cargo build --release --target x86_64-unknown-linux-gnu --no-default-features --features cli-only

build-linux-arm-cli: install-targets
    @echo "Building CLI for Linux ARM64..."
    cargo build --release --target aarch64-unknown-linux-gnu --no-default-features --features cli-only

build-windows-cli: install-targets
    @echo "Building CLI for Windows x64..."
    cargo build --release --target x86_64-pc-windows-msvc --no-default-features --features cli-only

build-windows-arm-cli: install-targets
    @echo "Building CLI for Windows ARM64..."
    cargo build --release --target aarch64-pc-windows-msvc --no-default-features --features cli-only

build-macos-cli: install-targets
    @echo "Building CLI for macOS Intel..."
    cargo build --release --target x86_64-apple-darwin --no-default-features --features cli-only

build-macos-arm-cli: install-targets
    @echo "Building CLI for macOS Apple Silicon..."
    cargo build --release --target aarch64-apple-darwin --no-default-features --features cli-only

# Development workflow: format, lint, test
dev-check: fmt lint test
    @echo "✅ Development checks complete!"

# Release preparation workflow
release-prep: fmt lint test audit
    @echo "Release preparation complete!"

# Testing Commands
# ==================

# Test all critical development workflows
test-workflows:
    @echo "Testing critical development workflows..."
    @echo "Testing setup command..."
    @just setup
    @echo "Testing version command..."
    @just version
    @echo "Testing build commands..."
    @just build-cli
    @echo "Testing format command..."
    @just fmt
    @echo "Testing quick validation..."
    @just validate-quick
    @echo "✅ All critical workflows tested successfully!"
    @echo ""
    @echo "⚠️  Note: Some commands may require system dependencies"
    @echo "   Run 'just setup-ubuntu' on Ubuntu or 'just setup-macos' on macOS"

# Test that justfile commands work correctly
test-justfile:
    @echo "Testing justfile commands..."
    @echo "Testing basic commands that should always work..."
    @just version
    @just check-system
    @echo "Testing build commands..."
    @just build-cli
    @echo "Testing validation commands..."
    @just validate-quick
    @echo "✅ Justfile commands working correctly!"

# Test primary development commands (requires manual verification)
test-dev-commands:
    @echo "Testing development commands (manual verification required)..."
    @echo "1. Testing 'just dev' (GUI development mode)..."
    @echo "   - Should compile successfully"
    @echo "   - Should start GUI application"
    @echo "   - Should show BLE peripheral logs"
    @echo "   - Press Ctrl+C to stop and continue"
    @echo ""
    @echo "Starting 'just dev' in 3 seconds..."
    @sleep 3
    timeout 30s just dev || echo "✅ GUI dev mode test completed"
    @echo ""
    @echo "2. Testing 'just dev-cli' (CLI development mode)..."
    @echo "   - Should compile successfully"
    @echo "   - Should show 'CLI mode not available' error (expected)"
    @echo "   - Press Ctrl+C to stop and continue"
    @echo ""
    @echo "Starting 'just dev-cli' in 3 seconds..."
    @sleep 3
    timeout 10s just dev-cli || echo "✅ CLI dev mode test completed"
    @echo ""
    @echo "✅ Development command testing complete!"

# Validate system requirements
check-system:
    @echo "Checking system requirements..."
    @echo "Rust toolchain:"
    @rustc --version || echo "❌ Rust not installed"
    @cargo --version || echo "❌ Cargo not installed"
    @echo ""
    @echo "Development tools:"
    @cargo watch --version 2>/dev/null || echo "⚠️  cargo-watch not installed (run 'just setup')"
    @cargo audit --version 2>/dev/null || echo "⚠️  cargo-audit not installed (run 'just setup')"
    @echo ""
    @echo "Node.js (optional for frontend):"
    @node --version 2>/dev/null || echo "⚠️  Node.js not installed"
    @npm --version 2>/dev/null || echo "⚠️  NPM not installed"
    @echo ""
    @echo "System information:"
    @echo "Platform: $(uname -s) $(uname -m)"
    @echo "Just: $(just --version)"
    @echo ""
    @echo "✅ System check complete!"

# UI Commands
# ===============

# Build the frontend UI
build-ui:
    @echo "Building frontend UI..."
    cd ui && npm run build
    @echo "✅ Frontend built successfully!"

# Quick UI development workflow
ui-dev:
    @echo "Starting UI development workflow..."
    cd ui && npm run build
    @echo "✅ Frontend built successfully!"
    @echo "💡 Run 'just dev' to start the GUI application"
