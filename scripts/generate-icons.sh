#!/bin/bash

# Haptic Harmony Simulator - Icon Generation Script
# Generates all required Tauri icon formats from source icon.png
# 
# Requirements:
# - ImageMagick (brew install imagemagick on macOS, apt-get install imagemagick on Ubuntu)
# - icnsutils (brew install libicns on macOS, apt-get install icnsutils on Ubuntu)
# - For Windows: Install ImageMagick from https://imagemagick.org/script/download.php#windows

set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Configuration
SOURCE_ICON="icons/icon.png"
ICONS_DIR="icons"
TEMP_DIR="temp_icons"

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

# Check if source icon exists
if [ ! -f "$SOURCE_ICON" ]; then
    print_error "Source icon not found: $SOURCE_ICON"
    exit 1
fi

# Check if ImageMagick is installed
if ! command -v magick &> /dev/null && ! command -v convert &> /dev/null; then
    print_error "ImageMagick is not installed. Please install it first:"
    echo "  macOS: brew install imagemagick"
    echo "  Ubuntu: sudo apt-get install imagemagick"
    echo "  Windows: Download from https://imagemagick.org/script/download.php#windows"
    exit 1
fi

# Use the modern magick command if available, fallback to convert
if command -v magick &> /dev/null; then
    MAGICK_CMD="magick"
else
    MAGICK_CMD="convert"
fi

print_status "Starting icon generation from $SOURCE_ICON"

# Create temporary directory
mkdir -p "$TEMP_DIR"

# Generate PNG icons for various sizes
print_status "Generating PNG icons..."

# Standard sizes for Tauri
sizes=(16 32 64 128 256 512 1024)

for size in "${sizes[@]}"; do
    output_file="$ICONS_DIR/${size}x${size}.png"
    $MAGICK_CMD "$SOURCE_ICON" -resize "${size}x${size}" "$output_file"
    print_success "Generated $output_file"
done

# Generate @2x versions for high-DPI displays
print_status "Generating high-DPI (@2x) PNG icons..."
hidpi_sizes=(32 64 128 256 512)

for size in "${hidpi_sizes[@]}"; do
    double_size=$((size * 2))
    output_file="$ICONS_DIR/${size}x${size}@2x.png"
    $MAGICK_CMD "$SOURCE_ICON" -resize "${double_size}x${double_size}" "$output_file"
    print_success "Generated $output_file"
done

# Generate Windows ICO file
print_status "Generating Windows ICO file..."
$MAGICK_CMD "$SOURCE_ICON" \
    \( -clone 0 -resize 16x16 \) \
    \( -clone 0 -resize 32x32 \) \
    \( -clone 0 -resize 48x48 \) \
    \( -clone 0 -resize 64x64 \) \
    \( -clone 0 -resize 128x128 \) \
    \( -clone 0 -resize 256x256 \) \
    -delete 0 "$ICONS_DIR/icon.ico"
print_success "Generated $ICONS_DIR/icon.ico"

# Generate macOS ICNS file
print_status "Generating macOS ICNS file..."

# Create iconset directory
iconset_dir="$TEMP_DIR/icon.iconset"
mkdir -p "$iconset_dir"

# Generate all required sizes for ICNS
icns_files=(
    "icon_16x16.png:16"
    "icon_16x16@2x.png:32"
    "icon_32x32.png:32"
    "icon_32x32@2x.png:64"
    "icon_128x128.png:128"
    "icon_128x128@2x.png:256"
    "icon_256x256.png:256"
    "icon_256x256@2x.png:512"
    "icon_512x512.png:512"
    "icon_512x512@2x.png:1024"
)

for file_info in "${icns_files[@]}"; do
    filename="${file_info%:*}"
    size="${file_info#*:}"
    $MAGICK_CMD "$SOURCE_ICON" -resize "${size}x${size}" "$iconset_dir/$filename"
done

# Create ICNS file
if command -v iconutil &> /dev/null; then
    # Use macOS iconutil (preferred)
    iconutil -c icns "$iconset_dir" -o "$ICONS_DIR/icon.icns"
    print_success "Generated $ICONS_DIR/icon.icns using iconutil"
elif command -v png2icns &> /dev/null; then
    # Use libicns png2icns as fallback
    png2icns "$ICONS_DIR/icon.icns" "$iconset_dir"/*.png
    print_success "Generated $ICONS_DIR/icon.icns using png2icns"
else
    print_warning "Neither iconutil nor png2icns found. ICNS file not generated."
    print_warning "On macOS: iconutil is built-in"
    print_warning "On Linux: sudo apt-get install icnsutils"
fi

# Generate additional formats for package managers
print_status "Generating additional formats for package managers..."

# Generate SVG if source is PNG (optional, for scalable icons)
if command -v potrace &> /dev/null; then
    # Convert to bitmap first, then trace to SVG
    $MAGICK_CMD "$SOURCE_ICON" -resize 512x512 "$TEMP_DIR/temp.pbm"
    potrace "$TEMP_DIR/temp.pbm" -s -o "$ICONS_DIR/icon.svg"
    print_success "Generated $ICONS_DIR/icon.svg"
else
    print_warning "potrace not found. SVG icon not generated."
    print_warning "Install with: brew install potrace (macOS) or sudo apt-get install potrace (Linux)"
fi

# Clean up temporary directory
rm -rf "$TEMP_DIR"

print_success "Icon generation completed!"
print_status "Generated icons in $ICONS_DIR directory:"
ls -la "$ICONS_DIR"

print_status "Icon generation summary:"
echo "  ✓ PNG icons: 16x16, 32x32, 64x64, 128x128, 256x256, 512x512, 1024x1024"
echo "  ✓ High-DPI PNG icons: @2x versions for common sizes"
echo "  ✓ Windows ICO: Multi-resolution icon file"
echo "  ✓ macOS ICNS: Apple icon format (if iconutil/png2icns available)"
echo "  ✓ SVG: Scalable vector format (if potrace available)"

print_status "These icons are now ready for use with Tauri and package managers!"
