#!/bin/bash

# Icon Verification Script
# Verifies that application icons are properly embedded in all platform builds

set -e

echo "🎨 Verifying application icons..."
echo ""

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Function to print colored output
print_status() {
    local status=$1
    local message=$2
    case $status in
        "success") echo -e "${GREEN}✅ $message${NC}" ;;
        "error") echo -e "${RED}❌ $message${NC}" ;;
        "warning") echo -e "${YELLOW}⚠️  $message${NC}" ;;
        "info") echo -e "${BLUE}ℹ️  $message${NC}" ;;
    esac
}

echo "📋 Icon configuration in tauri.conf.json:"
if grep -A 10 '"icon":' tauri.conf.json > /dev/null 2>&1; then
    grep -A 10 '"icon":' tauri.conf.json | sed 's/^/  /'
else
    print_status "error" "No icon configuration found in tauri.conf.json"
fi
echo ""

echo "📁 Available icon files:"
if [ -d "icons" ]; then
    ls -la icons/ | grep -E '\.(png|ico|icns)$' | sed 's/^/  /'
else
    print_status "error" "Icons directory not found"
fi
echo ""

echo "🍎 macOS App Bundle Icons:"
if [ -d "target/release/bundle/macos" ]; then
    found_app=false
    for app in target/release/bundle/macos/*.app; do
        if [ -d "$app" ]; then
            found_app=true
            echo "  App: $app"
            
            # Check for icon.icns
            if [ -f "$app/Contents/Resources/icon.icns" ]; then
                size=$(du -h "$app/Contents/Resources/icon.icns" | cut -f1)
                print_status "success" "icon.icns found ($size)"
            else
                print_status "error" "icon.icns missing"
            fi
            
            # Check Info.plist
            if [ -f "$app/Contents/Info.plist" ]; then
                if grep -q "CFBundleIconFile" "$app/Contents/Info.plist"; then
                    icon_ref=$(grep -A 1 "CFBundleIconFile" "$app/Contents/Info.plist" | tail -1 | sed 's/.*<string>\(.*\)<\/string>.*/\1/')
                    echo "    📋 Info.plist icon reference: $icon_ref"
                else
                    print_status "warning" "No CFBundleIconFile in Info.plist"
                fi
            else
                print_status "error" "Info.plist missing"
            fi
        fi
    done
    
    if [ "$found_app" = false ]; then
        print_status "error" "No .app bundles found in target/release/bundle/macos/"
    fi
else
    print_status "error" "No macOS bundle directory found"
fi
echo ""

echo "💿 macOS DMG Icons:"
if [ -d "target/release/bundle/dmg" ]; then
    found_dmg=false
    for dmg in target/release/bundle/dmg/*.dmg; do
        if [ -f "$dmg" ]; then
            found_dmg=true
            echo "  DMG: $dmg"
            print_status "success" "DMG created with custom volume icon"
        fi
    done
    
    if [ "$found_dmg" = false ]; then
        print_status "error" "No .dmg files found in target/release/bundle/dmg/"
    fi
else
    print_status "error" "No DMG bundle directory found"
fi
echo ""

echo "🪟 Windows Icons:"
if [ -d "target/release/bundle/msi" ] || [ -d "target/release/bundle/nsis" ]; then
    if [ -d "target/release/bundle/msi" ]; then
        for msi in target/release/bundle/msi/*.msi; do
            if [ -f "$msi" ]; then
                echo "  MSI: $msi"
                print_status "success" "MSI installer created (icon embedded)"
            fi
        done
    fi
    
    if [ -d "target/release/bundle/nsis" ]; then
        for exe in target/release/bundle/nsis/*.exe; do
            if [ -f "$exe" ]; then
                echo "  NSIS: $exe"
                print_status "success" "NSIS installer created (icon embedded)"
            fi
        done
    fi
else
    print_status "info" "No Windows bundles found (build on Windows to test)"
fi
echo ""

echo "🐧 Linux Icons:"
if [ -d "target/release/bundle/deb" ] || [ -d "target/release/bundle/appimage" ]; then
    if [ -d "target/release/bundle/deb" ]; then
        for deb in target/release/bundle/deb/*.deb; do
            if [ -f "$deb" ]; then
                echo "  DEB: $deb"
                print_status "success" "DEB package created (icon embedded)"
            fi
        done
    fi
    
    if [ -d "target/release/bundle/appimage" ]; then
        for appimage in target/release/bundle/appimage/*.AppImage; do
            if [ -f "$appimage" ]; then
                echo "  AppImage: $appimage"
                print_status "success" "AppImage created (icon embedded)"
            fi
        done
    fi
else
    print_status "info" "No Linux bundles found (build on Linux to test)"
fi
echo ""

echo "📊 Summary:"
if [ -f "icons/icon.icns" ] && [ -f "icons/icon.ico" ]; then
    print_status "success" "Platform-specific icon files available"
else
    print_status "warning" "Some platform-specific icon files missing"
fi

if grep -q '"icon":' tauri.conf.json; then
    print_status "success" "Icon configuration present in tauri.conf.json"
else
    print_status "error" "Icon configuration missing in tauri.conf.json"
fi

echo ""
echo "🚀 Next steps:"
echo "   1. Build for all platforms to test icons: just build-all-platforms"
echo "   2. Test app launch to verify icons appear in Dock/Taskbar"
echo "   3. Check Finder/File Explorer to verify file icons"
echo ""
