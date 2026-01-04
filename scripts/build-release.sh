#!/bin/bash

# Haptic Harmony Simulator - Comprehensive Release Build Script
# Builds for all supported platforms and architectures

set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Configuration
VERSION=${1:-"0.1.0"}
BUILD_DIR="build"
DIST_DIR="dist"

# Function to print colored output
print_status() {
    echo -e "${BLUE}[INFO]${NC} $1"
}

print_success() {
    echo -e "${GREEN}[SUCCESS]${NC} $1"
}

print_warning() {
    echo -e "${YELLOW}[WARNING]${NC} $1"
}

print_error() {
    echo -e "${RED}[ERROR]${NC} $1"
}

# Function to check if command exists
command_exists() {
    command -v "$1" >/dev/null 2>&1
}

# Function to install Rust targets
install_rust_targets() {
    print_status "Installing Rust targets..."
    
    # macOS targets
    rustup target add x86_64-apple-darwin
    rustup target add aarch64-apple-darwin
    
    # Linux targets
    rustup target add x86_64-unknown-linux-gnu
    rustup target add aarch64-unknown-linux-gnu
    
    # Windows targets
    rustup target add x86_64-pc-windows-msvc
    rustup target add aarch64-pc-windows-msvc
    
    print_success "Rust targets installed"
}

# Function to build for a specific target
build_target() {
    local target=$1
    local features=$2
    local output_name=$3
    
    print_status "Building for target: $target"
    
    if [[ "$features" == "tauri-gui" ]]; then
        # Build frontend first for GUI builds
        print_status "Building frontend..."
        cd ui
        npm ci
        npm run build
        cd ..
    fi
    
    # Build the Rust application
    cargo build --release --target "$target" --features "$features"
    
    # Copy binary to dist directory
    local binary_name="haptic-harmony-simulation"
    if [[ "$target" == *"windows"* ]]; then
        binary_name="${binary_name}.exe"
    fi
    
    local source_path="target/$target/release/$binary_name"
    local dest_path="$DIST_DIR/$output_name"
    
    if [[ -f "$source_path" ]]; then
        cp "$source_path" "$dest_path"
        print_success "Built $output_name"
    else
        print_error "Build failed for $target"
        return 1
    fi
}

# Function to create archives
create_archives() {
    print_status "Creating distribution archives..."
    
    cd "$DIST_DIR"
    
    # Create tar.gz archives for Unix-like systems
    for file in haptic-harmony-simulation-*-{linux,macos}-*; do
        if [[ -f "$file" && "$file" != *.exe ]]; then
            tar -czf "${file}.tar.gz" "$file"
            print_success "Created ${file}.tar.gz"
        fi
    done
    
    # Create zip archives for Windows
    for file in haptic-harmony-simulation-*-windows-*.exe; do
        if [[ -f "$file" ]]; then
            zip "${file%.exe}.zip" "$file"
            print_success "Created ${file%.exe}.zip"
        fi
    done
    
    cd ..
}

# Function to generate checksums
generate_checksums() {
    print_status "Generating checksums..."
    
    cd "$DIST_DIR"
    
    # Generate SHA256 checksums
    if command_exists sha256sum; then
        sha256sum * > SHA256SUMS
    elif command_exists shasum; then
        shasum -a 256 * > SHA256SUMS
    else
        print_warning "No SHA256 utility found, skipping checksums"
        cd ..
        return
    fi
    
    print_success "Generated SHA256SUMS"
    cd ..
}

# Main build function
main() {
    print_status "Starting comprehensive build for version $VERSION"
    
    # Clean and create directories
    rm -rf "$BUILD_DIR" "$DIST_DIR"
    mkdir -p "$BUILD_DIR" "$DIST_DIR"
    
    # Generate icons first
    print_status "Generating icons..."
    if [[ -x "scripts/generate-icons.sh" ]]; then
        ./scripts/generate-icons.sh
    else
        print_warning "Icon generation script not found or not executable"
    fi
    
    # Install Rust targets
    install_rust_targets
    
    # Build matrix
    declare -A builds=(
        # CLI builds
        ["x86_64-apple-darwin"]="cli:haptic-harmony-simulation-cli-macos-x64"
        ["aarch64-apple-darwin"]="cli:haptic-harmony-simulation-cli-macos-arm64"
        ["x86_64-unknown-linux-gnu"]="cli:haptic-harmony-simulation-cli-linux-x64"
        ["aarch64-unknown-linux-gnu"]="cli:haptic-harmony-simulation-cli-linux-arm64"
        ["x86_64-pc-windows-msvc"]="cli:haptic-harmony-simulation-cli-windows-x64.exe"
        ["aarch64-pc-windows-msvc"]="cli:haptic-harmony-simulation-cli-windows-arm64.exe"
        
        # GUI builds
        ["x86_64-apple-darwin-gui"]="tauri-gui:haptic-harmony-simulation-gui-macos-x64"
        ["aarch64-apple-darwin-gui"]="tauri-gui:haptic-harmony-simulation-gui-macos-arm64"
        ["x86_64-unknown-linux-gnu-gui"]="tauri-gui:haptic-harmony-simulation-gui-linux-x64"
        ["aarch64-unknown-linux-gnu-gui"]="tauri-gui:haptic-harmony-simulation-gui-linux-arm64"
        ["x86_64-pc-windows-msvc-gui"]="tauri-gui:haptic-harmony-simulation-gui-windows-x64.exe"
        ["aarch64-pc-windows-msvc-gui"]="tauri-gui:haptic-harmony-simulation-gui-windows-arm64.exe"
    )
    
    # Build all targets
    for build_key in "${!builds[@]}"; do
        IFS=':' read -r features output_name <<< "${builds[$build_key]}"
        
        # Extract target from build_key
        target="${build_key%-gui}"
        
        # Skip builds that can't be done on current platform
        if [[ "$target" == *"windows"* ]] && [[ "$OSTYPE" != "msys" && "$OSTYPE" != "cygwin" ]]; then
            print_warning "Skipping Windows build on non-Windows platform: $target"
            continue
        fi
        
        # Build the target
        if build_target "$target" "$features" "$output_name"; then
            print_success "Successfully built $output_name"
        else
            print_error "Failed to build $output_name"
        fi
    done
    
    # Create archives and checksums
    create_archives
    generate_checksums
    
    # Display summary
    print_success "Build completed! Files in $DIST_DIR:"
    ls -la "$DIST_DIR"
    
    print_status "Build summary for version $VERSION:"
    echo "  ✓ CLI builds: Multiple architectures"
    echo "  ✓ GUI builds: Multiple architectures"
    echo "  ✓ Archives: tar.gz and zip formats"
    echo "  ✓ Checksums: SHA256SUMS file"
    echo ""
    echo "Ready for release and package manager publishing!"
}

# Check dependencies
print_status "Checking dependencies..."

if ! command_exists cargo; then
    print_error "Rust/Cargo not found. Please install Rust first."
    exit 1
fi

if ! command_exists npm; then
    print_error "Node.js/npm not found. Please install Node.js first."
    exit 1
fi

# Run main function
main "$@"
