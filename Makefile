# Haptic Harmony Simulation - Build Automation
# Cross-platform build system for development and production

.PHONY: help install build build-gui build-cli test clean dev dev-gui dev-cli format lint audit coverage release docker \
        check-signing verify-macos verify-windows release-signed setup-signing

# Default target
help: ## Show this help message
	@echo "Haptic Harmony Simulation - Build Commands"
	@echo "=========================================="
	@echo ""
	@echo "🔧 Development Commands:"
	@awk 'BEGIN {FS = ":.*?## "} /^(build|dev|test|format|lint|audit|coverage):.*?## / {printf "\033[36m%-20s\033[0m %s\n", $$1, $$2}' $(MAKEFILE_LIST)
	@echo ""
	@echo "🚀 Release Commands:"
	@awk 'BEGIN {FS = ":.*?## "} /^(release|build-all|build-linux|build-windows|build-macos):.*?## / {printf "\033[36m%-20s\033[0m %s\n", $$1, $$2}' $(MAKEFILE_LIST)
	@echo ""
	@echo "🔐 Code Signing Commands:"
	@awk 'BEGIN {FS = ":.*?## "} /^(check-signing|verify|setup-signing|release-signed):.*?## / {printf "\033[36m%-20s\033[0m %s\n", $$1, $$2}' $(MAKEFILE_LIST)
	@echo ""
	@echo "🛠️  Other Commands:"
	@awk 'BEGIN {FS = ":.*?## "} /^(install|clean|docs|setup|sim|check|size|version):.*?## / {printf "\033[36m%-20s\033[0m %s\n", $$1, $$2}' $(MAKEFILE_LIST)
	@echo ""
	@echo "📖 For detailed code signing setup, see:"
	@echo "   • MACOS_CODE_SIGNING_SETUP.md"
	@echo "   • WINDOWS_CODE_SIGNING_SETUP.md"

# Installation and setup
install: ## Install all dependencies and tools
	@echo "Installing Rust dependencies..."
	cargo install cargo-watch cargo-tarpaulin cargo-audit
	@echo "Installing Node.js dependencies..."
	cd ui && npm install
	@echo "Installing system dependencies..."
	@if [ "$$(uname)" = "Linux" ]; then \
		echo "Please install: libwebkit2gtk-4.1-dev libxdo-dev libayatana-appindicator3-dev librsvg2-dev libudev-dev libdbus-1-dev pkg-config build-essential"; \
	elif [ "$$(uname)" = "Darwin" ]; then \
		echo "macOS dependencies should be available by default"; \
	fi

# Build targets
build: build-cli build-gui ## Build both CLI and GUI versions

build-cli: ## Build CLI version only
	@echo "Building CLI version..."
	cargo build --release --no-default-features --features cli-only

build-gui: ## Build GUI version only
	@echo "Building frontend..."
	cd ui && npm run build
	@echo "Building GUI version..."
	cargo build --release --features tauri-gui

build-debug: ## Build debug versions
	cargo build --no-default-features --features cli-only
	cargo build --features tauri-gui

# Development targets
dev: dev-cli ## Start development mode (CLI)

dev-cli: ## Start CLI development with hot reload
	@echo "Starting CLI development mode..."
	cargo watch -x "run --no-default-features --features cli-only -- --mode cli"

dev-gui: ## Start GUI development with hot reload
	@echo "Starting GUI development mode..."
	cargo watch -x "run --features tauri-gui -- --mode gui"

dev-frontend: ## Start frontend development server
	cd ui && npm run dev

# Testing
test: ## Run all tests
	@echo "Running tests..."
	cargo test --no-default-features --features cli-only
	cargo test --features tauri-gui

test-watch: ## Run tests with hot reload
	cargo watch -x test

test-integration: ## Run integration tests only
	cargo test --test integration_tests

# Code quality
format: ## Format code
	@echo "Formatting Rust code..."
	cargo fmt
	@echo "Formatting frontend code..."
	cd ui && npm run format

lint: ## Run linting
	@echo "Running Rust linting..."
	cargo clippy --all-targets --all-features -- -D warnings
	@echo "Running frontend linting..."
	cd ui && npm run lint

audit: ## Run security audit
	@echo "Running security audit..."
	cargo audit

coverage: ## Generate code coverage report
	@echo "Generating coverage report..."
	cargo tarpaulin --out html --output-dir coverage

# Cross-platform builds
build-all: ## Build for all supported platforms
	@echo "Building for all platforms..."
	$(MAKE) build-linux
	$(MAKE) build-windows
	$(MAKE) build-macos

build-linux: ## Build for Linux x64 (CLI and GUI)
	@echo "Building CLI for Linux x64..."
	cargo build --release --target x86_64-unknown-linux-gnu --no-default-features --features cli-only
	@echo "Building GUI for Linux x64..."
	cd ui && npm run build
	cargo build --release --target x86_64-unknown-linux-gnu --features tauri-gui

build-linux-arm: ## Build for Linux ARM64 (CLI and GUI)
	@echo "Building CLI for Linux ARM64..."
	cargo build --release --target aarch64-unknown-linux-gnu --no-default-features --features cli-only
	@echo "Building GUI for Linux ARM64..."
	cd ui && npm run build
	cargo build --release --target aarch64-unknown-linux-gnu --features tauri-gui

build-windows: ## Build for Windows x64 (CLI and GUI)
	@echo "Building CLI for Windows x64..."
	cargo build --release --target x86_64-pc-windows-msvc --no-default-features --features cli-only
	@echo "Building GUI for Windows x64..."
	cd ui && npm run build
	cargo build --release --target x86_64-pc-windows-msvc --features tauri-gui

build-windows-arm: ## Build for Windows ARM64 (CLI and GUI)
	@echo "Building CLI for Windows ARM64..."
	cargo build --release --target aarch64-pc-windows-msvc --no-default-features --features cli-only
	@echo "Building GUI for Windows ARM64..."
	cd ui && npm run build
	cargo build --release --target aarch64-pc-windows-msvc --features tauri-gui

build-macos: ## Build for macOS (Intel and Apple Silicon)
	@echo "Building CLI for macOS x64..."
	cargo build --release --target x86_64-apple-darwin --no-default-features --features cli-only
	@echo "Building GUI for macOS x64..."
	cd ui && npm run build
	cargo build --release --target x86_64-apple-darwin --features tauri-gui
	@echo "Building CLI for macOS ARM64..."
	cargo build --release --target aarch64-apple-darwin --no-default-features --features cli-only
	@echo "Building GUI for macOS ARM64..."
	cargo build --release --target aarch64-apple-darwin --features tauri-gui

# Tauri specific builds
tauri-dev: ## Start Tauri development mode
	cd ui && npm run tauri dev

tauri-build: ## Build Tauri application
	cd ui && npm run tauri build

tauri-bundle: ## Create platform-specific bundles
	cd ui && npm run tauri build -- --bundles all

# Release preparation
release-prep: ## Prepare for release (format, lint, test)
	$(MAKE) format
	$(MAKE) lint
	$(MAKE) test
	$(MAKE) audit

release-build: ## Build release versions
	$(MAKE) build-all
	$(MAKE) tauri-bundle

# Code signing targets
check-signing: ## Check code signing setup
	@echo "Checking code signing setup..."
	@echo "macOS signing identities:"
	@security find-identity -v -p codesigning 2>/dev/null || echo "No macOS signing identities found"
	@echo ""
	@echo "Required GitHub Secrets for CI/CD:"
	@echo "macOS: APPLE_CERTIFICATE, APPLE_CERTIFICATE_PASSWORD, APPLE_SIGNING_IDENTITY, APPLE_ID, APPLE_PASSWORD, APPLE_TEAM_ID"
	@echo "Windows: WINDOWS_CERTIFICATE, WINDOWS_CERTIFICATE_PASSWORD"
	@echo ""
	@echo "See MACOS_CODE_SIGNING_SETUP.md and WINDOWS_CODE_SIGNING_SETUP.md for details"

verify-macos: ## Verify macOS application signature (usage: make verify-macos APP=/path/to/app.app)
	@echo "Verifying macOS application signature..."
	@if [ -z "$(APP)" ]; then echo "Usage: make verify-macos APP=/path/to/app.app"; exit 1; fi
	codesign --verify --verbose=2 "$(APP)"
	codesign -dv --verbose=4 "$(APP)"
	spctl --assess --type open --context context:primary-signature --verbose=2 "$(APP)"

verify-windows: ## Verify Windows application signature (usage: make verify-windows APP=/path/to/app.exe)
	@echo "Verifying Windows application signature..."
	@if [ -z "$(APP)" ]; then echo "Usage: make verify-windows APP=/path/to/app.exe"; exit 1; fi
	@powershell -Command "Get-AuthenticodeSignature -FilePath '$(APP)' | Format-List"

release-signed: ## Build signed releases for all platforms (requires certificates)
	@echo "Building signed releases for all platforms..."
	@echo "⚠️  This requires code signing certificates to be configured"
	$(MAKE) check-signing
	$(MAKE) build-all
	$(MAKE) tauri-bundle
	@echo "✅ Signed releases built!"

setup-signing: ## Show code signing setup instructions
	@echo "Code Signing Setup Instructions"
	@echo "==============================="
	@echo ""
	@echo "📖 Documentation:"
	@echo "  • MACOS_CODE_SIGNING_SETUP.md - Complete macOS setup guide"
	@echo "  • WINDOWS_CODE_SIGNING_SETUP.md - Complete Windows setup guide"
	@echo ""
	@echo "🔧 Commands:"
	@echo "  • make check-signing - Check current signing setup"
	@echo "  • make verify-macos APP=/path/to/app.app - Verify macOS signature"
	@echo "  • make verify-windows APP=/path/to/app.exe - Verify Windows signature"
	@echo "  • make release-signed - Build signed releases"

# Docker targets
docker-build: ## Build Docker image
	docker build -t haptic-harmony-simulation .

docker-run: ## Run in Docker container
	docker run -it --rm haptic-harmony-simulation

# Cleanup
clean: ## Clean build artifacts
	@echo "Cleaning build artifacts..."
	cargo clean
	cd ui && npm run clean
	rm -rf coverage/
	rm -rf target/
	rm -rf ui/dist/

clean-deps: ## Clean dependencies (requires reinstall)
	$(MAKE) clean
	rm -rf ~/.cargo/registry
	rm -rf ui/node_modules/

# Documentation
docs: ## Generate documentation
	@echo "Generating Rust documentation..."
	cargo doc --open --all-features
	@echo "Generating frontend documentation..."
	cd ui && npm run docs

docs-serve: ## Serve documentation locally
	cargo doc --no-deps --open

# Environment setup
setup-dev: ## Setup development environment
	@echo "Setting up development environment..."
	$(MAKE) install
	$(MAKE) build-debug
	@echo "Development environment ready!"

setup-ci: ## Setup CI environment
	rustup component add rustfmt clippy
	cargo install cargo-audit cargo-tarpaulin

# Ring simulation targets
sim-b1: ## Run B1 ring simulation
	cargo run --features tauri-gui -- --mode gui --ring-type b1

sim-a1: ## Run A1 ring simulation (future)
	cargo run --features tauri-gui -- --mode gui --ring-type a1

sim-p1: ## Run P1 ring simulation (future)
	cargo run --features tauri-gui -- --mode gui --ring-type p1

# Utility targets
check: ## Quick check without building
	cargo check
	cargo check --features tauri-gui

size: ## Show binary sizes
	@echo "Binary sizes:"
	@ls -lh target/release/haptic-harmony-simulation 2>/dev/null || echo "CLI binary not found"
	@find target/release -name "*.app" -o -name "*.exe" -o -name "*.dmg" -o -name "*.deb" | xargs ls -lh 2>/dev/null || echo "GUI bundles not found"

version: ## Show version information
	@echo "Haptic Harmony Simulation Build Information"
	@echo "==========================================="
	@echo "Rust version: $$(rustc --version)"
	@echo "Cargo version: $$(cargo --version)"
	@echo "Node version: $$(node --version 2>/dev/null || echo 'Not installed')"
	@echo "NPM version: $$(npm --version 2>/dev/null || echo 'Not installed')"
	@echo "Platform: $$(uname -s) $$(uname -m)"
