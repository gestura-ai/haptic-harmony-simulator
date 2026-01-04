#!/bin/bash
# Production validation script - ensures code is ready for CI/CD
# This script runs all the same checks that GitHub Actions will run

set -e  # Exit on any error

echo "🔍 PRODUCTION VALIDATION PIPELINE"
echo "=================================="

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Function to print status
print_status() {
    echo -e "${BLUE}[$(date +'%H:%M:%S')]${NC} $1"
}

print_success() {
    echo -e "${GREEN}✅ $1${NC}"
}

print_error() {
    echo -e "${RED}❌ $1${NC}"
}

print_warning() {
    echo -e "${YELLOW}⚠️  $1${NC}"
}

# Check if we're in the right directory
if [ ! -f "Cargo.toml" ]; then
    print_error "Must be run from project root (where Cargo.toml is located)"
    exit 1
fi

# Step 1: Code Formatting Check
print_status "Step 1/8: Checking code formatting..."
if cargo fmt --all -- --check; then
    print_success "Code formatting is correct"
else
    print_error "Code formatting failed. Run 'cargo fmt --all' to fix."
    exit 1
fi

# Step 2: Clippy Lints (CLI)
print_status "Step 2/8: Running clippy lints (CLI features)..."
if cargo clippy --all-targets --features cli-only -- -D warnings; then
    print_success "Clippy lints passed (CLI)"
else
    print_error "Clippy lints failed (CLI features)"
    exit 1
fi

# Step 3: Clippy Lints (GUI) - only if frontend is built
print_status "Step 3/8: Checking if frontend is built..."
if [ -d "ui/dist" ]; then
    print_status "Frontend found, running clippy lints (GUI features)..."
    if cargo clippy --all-targets --features tauri-gui -- -D warnings; then
        print_success "Clippy lints passed (GUI)"
    else
        print_error "Clippy lints failed (GUI features)"
        exit 1
    fi
else
    print_warning "Frontend not built, skipping GUI clippy checks"
    print_warning "Run 'cd ui && npm run build' to enable GUI validation"
fi

# Step 4: Unit Tests (CLI)
print_status "Step 4/8: Running unit tests (CLI features)..."
if cargo test --features cli-only; then
    print_success "Unit tests passed (CLI)"
else
    print_error "Unit tests failed (CLI features)"
    exit 1
fi

# Step 5: Unit Tests (GUI) - only if frontend is built
if [ -d "ui/dist" ]; then
    print_status "Step 5/8: Running unit tests (GUI features)..."
    if cargo test --features tauri-gui; then
        print_success "Unit tests passed (GUI)"
    else
        print_error "Unit tests failed (GUI features)"
        exit 1
    fi
else
    print_status "Step 5/8: Skipping GUI tests (frontend not built)"
fi

# Step 6: Build Check (CLI)
print_status "Step 6/8: Building CLI version..."
if cargo build --release --features cli-only; then
    print_success "CLI build successful"
else
    print_error "CLI build failed"
    exit 1
fi

# Step 7: Build Check (GUI) - only if frontend is built
if [ -d "ui/dist" ]; then
    print_status "Step 7/8: Building GUI version..."
    if cargo build --release --features tauri-gui; then
        print_success "GUI build successful"
    else
        print_error "GUI build failed"
        exit 1
    fi
else
    print_status "Step 7/8: Skipping GUI build (frontend not built)"
fi

# Step 8: Documentation Check
print_status "Step 8/8: Checking documentation..."
if cargo doc --no-deps --features cli-only; then
    print_success "Documentation generation successful"
else
    print_error "Documentation generation failed"
    exit 1
fi

# Final Summary
echo ""
echo "🎉 PRODUCTION VALIDATION COMPLETE"
echo "================================="
print_success "All checks passed! Code is ready for production."
echo ""
echo "Summary:"
echo "  ✅ Code formatting"
echo "  ✅ Clippy lints (CLI)"
if [ -d "ui/dist" ]; then
    echo "  ✅ Clippy lints (GUI)"
    echo "  ✅ Unit tests (GUI)"
    echo "  ✅ Build check (GUI)"
else
    echo "  ⚠️  GUI checks skipped (run 'cd ui && npm run build' first)"
fi
echo "  ✅ Unit tests (CLI)"
echo "  ✅ Build check (CLI)"
echo "  ✅ Documentation"
echo ""
echo "🚀 Ready to commit and push!"
