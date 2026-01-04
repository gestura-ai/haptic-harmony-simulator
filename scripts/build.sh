#!/bin/bash
# Haptic Harmony Simulation - Cross-platform Build Script
# Builds the application for multiple platforms and architectures

set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Configuration
PROJECT_NAME="haptic-harmony-simulation"
VERSION=$(grep '^version' Cargo.toml | sed 's/version = "\(.*\)"/\1/')
BUILD_DIR="dist"
TARGETS=(
    "x86_64-unknown-linux-gnu"
    "x86_64-pc-windows-msvc"
    "x86_64-apple-darwin"
    "aarch64-apple-darwin"
)

# Functions
log_info() {
    echo -e "${BLUE}[INFO]${NC} $1"
}

log_success() {
    echo -e "${GREEN}[SUCCESS]${NC} $1"
}

log_warning() {
    echo -e "${YELLOW}[WARNING]${NC} $1"
}

log_error() {
    echo -e "${RED}[ERROR]${NC} $1"
}

check_dependencies() {
    log_info "Checking dependencies..."
    
    # Check Rust
    if ! command -v cargo &> /dev/null; then
        log_error "Rust/Cargo not found. Please install Rust."
        exit 1
    fi
    
    # Check Node.js
    if ! command -v npm &> /dev/null; then
        log_error "Node.js/NPM not found. Please install Node.js."
        exit 1
    fi
    
    log_success "Dependencies check passed"
}

install_targets() {
    log_info "Installing Rust targets..."
    for target in "${TARGETS[@]}"; do
        if rustup target list --installed | grep -q "$target"; then
            log_info "Target $target already installed"
        else
            log_info "Installing target $target"
            rustup target add "$target"
        fi
    done
}

build_frontend() {
    log_info "Building frontend..."
    cd ui
    npm ci
    npm run build
    cd ..
    log_success "Frontend build completed"
}

build_cli() {
    local target=$1
    log_info "Building CLI for $target..."
    
    cargo build --release --target "$target"
    
    # Copy binary to dist directory
    local binary_name="$PROJECT_NAME"
    if [[ "$target" == *"windows"* ]]; then
        binary_name="$PROJECT_NAME.exe"
    fi
    
    local source_path="target/$target/release/$binary_name"
    local dest_path="$BUILD_DIR/cli/$target/$binary_name"
    
    mkdir -p "$(dirname "$dest_path")"
    cp "$source_path" "$dest_path"
    
    log_success "CLI build for $target completed"
}

show_usage() {
    echo "Usage: $0 [OPTIONS]"
    echo ""
    echo "Options:"
    echo "  --cli-only      Build only CLI version"
    echo "  --target TARGET Build for specific target only"
    echo "  --clean         Clean before building"
    echo "  --help          Show this help message"
    echo ""
    echo "Targets: ${TARGETS[*]}"
}

# Main execution
main() {
    local build_cli=true
    local clean=false
    local specific_target=""
    
    # Parse arguments
    while [[ $# -gt 0 ]]; do
        case $1 in
            --cli-only)
                shift
                ;;
            --target)
                specific_target="$2"
                shift 2
                ;;
            --clean)
                clean=true
                shift
                ;;
            --help)
                show_usage
                exit 0
                ;;
            *)
                log_error "Unknown option: $1"
                show_usage
                exit 1
                ;;
        esac
    done
    
    log_info "Starting build process for $PROJECT_NAME v$VERSION"
    
    # Clean if requested
    if [ "$clean" = true ]; then
        log_info "Cleaning previous builds..."
        rm -rf "$BUILD_DIR"
        cargo clean
        log_success "Clean completed"
    fi
    
    # Check dependencies
    check_dependencies
    
    # Install targets
    if [ -z "$specific_target" ]; then
        install_targets
    else
        rustup target add "$specific_target"
    fi
    
    # Create build directory
    mkdir -p "$BUILD_DIR"
    
    # Determine targets to build
    local targets_to_build=()
    if [ -n "$specific_target" ]; then
        targets_to_build=("$specific_target")
    else
        targets_to_build=("${TARGETS[@]}")
    fi
    
    # Build for each target
    for target in "${targets_to_build[@]}"; do
        log_info "Building for target: $target"
        
        if [ "$build_cli" = true ]; then
            build_cli "$target"
        fi
    done
    
    log_success "Build process completed successfully!"
    log_info "Build artifacts available in: $BUILD_DIR"
}

# Run main function with all arguments
main "$@"
