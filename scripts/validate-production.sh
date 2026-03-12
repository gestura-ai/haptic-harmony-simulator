#!/bin/bash

# Production validation script - mirrors the supported local release gate.

set -euo pipefail

echo "🔍 PRODUCTION VALIDATION PIPELINE"
echo "=================================="

RED='\033[0;31m'
GREEN='\033[0;32m'
BLUE='\033[0;34m'
NC='\033[0m'

step() { echo -e "${BLUE}[$(date +'%H:%M:%S')]${NC} $1"; }
ok() { echo -e "${GREEN}✅ $1${NC}"; }
fail() { echo -e "${RED}❌ $1${NC}"; exit 1; }

[[ -f Cargo.toml ]] || fail "Run this script from the repository root."

step "Step 1/8: Install frontend dependencies"
(cd ui && npm ci)
ok "Frontend dependencies installed"

step "Step 2/8: Build frontend"
(cd ui && npm run build)
ok "Frontend build completed"

step "Step 3/8: Check formatting"
cargo fmt --all -- --check
ok "Formatting is correct"

step "Step 4/8: Run clippy (CLI)"
cargo clippy --all-targets --no-default-features --features cli-only -- -D warnings
ok "CLI clippy passed"

step "Step 5/8: Run tests (CLI)"
cargo test --no-default-features --features cli-only --verbose
ok "CLI tests passed"

step "Step 6/8: Run clippy + tests (GUI)"
cargo clippy --all-targets --features tauri-gui -- -D warnings
cargo test --features tauri-gui --verbose
ok "GUI clippy and tests passed"

step "Step 7/8: Build GUI release binary"
cargo build --release --features tauri-gui
ok "GUI release build passed"

step "Step 8/8: Run platform-specific final checks"
if [[ "$(uname -s)" == "Linux" ]]; then
  cargo test --all-features --verbose
  cargo clippy --all-targets --all-features -- -D warnings
  ok "Linux all-features validation passed"
else
  echo "ℹ️  Skipping --all-features checks on $(uname -s); linux-ble remains Linux-only."
fi

echo
echo "🎉 PRODUCTION VALIDATION COMPLETE"
echo "================================="
ok "Supported local validation passed."