#!/bin/bash

# Apple Developer Certificate Verification Script
# This script helps verify that your Apple Developer certificate is properly configured

set -e

echo "Apple Developer Certificate Verification"
echo "==========================================="
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

# Check if running on macOS
if [[ "$OSTYPE" != "darwin"* ]]; then
    print_status "error" "This script must be run on macOS"
    exit 1
fi

print_status "info" "Checking Apple Developer certificate setup..."
echo ""

# Step 1: Check for signing identities
echo "1. Checking for code signing identities..."
if security find-identity -v -p codesigning | grep -q "Developer ID Application"; then
    print_status "success" "Developer ID Application certificate found"
    echo "   Available identities:"
    security find-identity -v -p codesigning | grep "Developer ID Application" | sed 's/^/   /'
else
    print_status "error" "No Developer ID Application certificate found"
    echo "   Please install your Apple Developer certificate in Keychain Access"
    exit 1
fi
echo ""

# Step 2: Check environment variables
echo "2. Checking environment variables..."
check_env_var() {
    local var_name=$1
    local show_value=$2
    if [[ -n "${!var_name}" ]]; then
        if [[ "$show_value" == "true" ]]; then
            print_status "success" "$var_name is set: ${!var_name}"
        else
            print_status "success" "$var_name is set (hidden for security)"
        fi
        return 0
    else
        print_status "error" "$var_name is not set"
        return 1
    fi
}

all_vars_set=true

check_env_var "APPLE_CERTIFICATE" "false" || all_vars_set=false
check_env_var "APPLE_CERTIFICATE_PASSWORD" "false" || all_vars_set=false
check_env_var "APPLE_SIGNING_IDENTITY" "true" || all_vars_set=false
check_env_var "APPLE_ID" "true" || all_vars_set=false
check_env_var "APPLE_PASSWORD" "false" || all_vars_set=false
check_env_var "APPLE_TEAM_ID" "true" || all_vars_set=false

echo ""

if [[ "$all_vars_set" == "false" ]]; then
    print_status "warning" "Some environment variables are missing"
    echo "   For GitHub Actions, these should be configured as repository secrets"
    echo "   For local testing, you can set them in your shell environment"
    echo ""
fi

# Step 3: Test certificate if environment variables are set
if [[ "$all_vars_set" == "true" ]]; then
    echo "3. Testing certificate import..."
    
    # Create temporary certificate file
    temp_cert=$(mktemp /tmp/test-cert.XXXXXX.p12)
    
    # Decode base64 certificate
    if echo "$APPLE_CERTIFICATE" | base64 -d > "$temp_cert" 2>/dev/null; then
        print_status "success" "Certificate decoded successfully"
        
        # Test certificate import
        if security import "$temp_cert" -k ~/Library/Keychains/login.keychain -P "$APPLE_CERTIFICATE_PASSWORD" -T /usr/bin/codesign 2>/dev/null; then
            print_status "success" "Certificate import test passed"
        else
            print_status "warning" "Certificate import test completed (may already be imported)"
        fi
        
        # Clean up
        rm -f "$temp_cert"
    else
        print_status "error" "Failed to decode certificate - check APPLE_CERTIFICATE format"
    fi
    echo ""
fi

# Step 4: Verify signing identity matches
echo "4. Verifying signing identity..."
if [[ -n "$APPLE_SIGNING_IDENTITY" ]]; then
    if security find-identity -v -p codesigning | grep -q "$APPLE_SIGNING_IDENTITY"; then
        print_status "success" "Signing identity matches available certificate"
    else
        print_status "error" "Signing identity '$APPLE_SIGNING_IDENTITY' not found in keychain"
        echo "   Available identities:"
        security find-identity -v -p codesigning | grep "Developer ID Application" | sed 's/^/   /'
    fi
else
    print_status "warning" "APPLE_SIGNING_IDENTITY not set - skipping verification"
fi
echo ""

# Step 5: Check Apple ID and Team ID format
echo "5. Validating Apple ID and Team ID format..."
if [[ -n "$APPLE_ID" ]]; then
    if [[ "$APPLE_ID" =~ ^[a-zA-Z0-9._%+-]+@[a-zA-Z0-9.-]+\.[a-zA-Z]{2,}$ ]]; then
        print_status "success" "Apple ID format is valid"
    else
        print_status "error" "Apple ID format appears invalid"
    fi
fi

if [[ -n "$APPLE_TEAM_ID" ]]; then
    if [[ "$APPLE_TEAM_ID" =~ ^[A-Z0-9]{10}$ ]]; then
        print_status "success" "Team ID format is valid"
    else
        print_status "error" "Team ID format appears invalid (should be 10 alphanumeric characters)"
    fi
fi
echo ""

# Step 6: Check for existing app bundles
echo "6. Checking for existing app bundles..."
if find target -name "*.app" -type d 2>/dev/null | head -1 | grep -q ".app"; then
    print_status "success" "Found existing .app bundles:"
    find target -name "*.app" -type d 2>/dev/null | sed 's/^/   /'
    echo ""
    echo "   Testing signature on existing bundle:"
    app_bundle=$(find target -name "*.app" -type d 2>/dev/null | head -1)
    if [[ -n "$app_bundle" ]]; then
        if codesign --verify --verbose=2 "$app_bundle" 2>/dev/null; then
            print_status "success" "App bundle is properly signed"
        else
            print_status "warning" "App bundle is not signed or signature is invalid"
        fi
    fi
else
    print_status "info" "No existing .app bundles found"
    echo "   Build the GUI to create app bundles:"
    echo "   cargo build --release --features tauri-gui"
    echo "   or: just build-macos-gui"
fi
echo ""

# Step 7: GitHub Secrets configuration guide
echo "7. GitHub Secrets Configuration:"
echo "   To configure these values in GitHub Actions:"
echo "   1. Go to your repository → Settings → Secrets and variables → Actions"
echo "   2. Add the following repository secrets:"
echo ""
echo "   APPLE_CERTIFICATE:"
echo "   - Export your certificate as .p12 from Keychain Access"
echo "   - Convert to base64: base64 -i certificate.p12 | pbcopy"
echo "   - Paste the base64 string as the secret value"
echo ""
echo "   APPLE_CERTIFICATE_PASSWORD:"
echo "   - The password you set when exporting the .p12 certificate"
echo ""
echo "   APPLE_SIGNING_IDENTITY:"
echo "   - Copy the exact identity string from step 1 above"
echo "   - Format: 'Developer ID Application: Your Name (TEAMID)'"
echo ""
echo "   APPLE_ID:"
echo "   - Your Apple ID email address"
echo ""
echo "   APPLE_PASSWORD:"
echo "   - App-specific password from appleid.apple.com"
echo "   - NOT your regular Apple ID password"
echo ""
echo "   APPLE_TEAM_ID:"
echo "   - Your 10-character team ID from Apple Developer Portal"
echo ""

# Final summary
echo "Summary:"
if security find-identity -v -p codesigning | grep -q "Developer ID Application"; then
    print_status "success" "Apple Developer certificate is installed"
else
    print_status "error" "Apple Developer certificate is missing"
fi

if [[ "$all_vars_set" == "true" ]]; then
    print_status "success" "All environment variables are configured"
    print_status "info" "Ready for GitHub Actions code signing!"
else
    print_status "warning" "Some environment variables need configuration"
    print_status "info" "Configure GitHub Secrets for automated signing"
fi

echo ""
echo "Next steps:"
echo "   1. Configure missing GitHub Secrets (if any)"
echo "   2. Test with: just test-apple-cert"
echo "   3. Create a release to test the full signing pipeline"
echo ""
