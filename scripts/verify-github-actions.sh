#!/bin/bash

set -euo pipefail

RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m'

ok() { echo -e "${GREEN}✅ $1${NC}"; }
warn() { echo -e "${YELLOW}⚠️  $1${NC}"; }
fail() { echo -e "${RED}❌ $1${NC}"; exit 1; }

echo "🔍 Verifying GitHub Actions configuration"
echo "========================================="

[[ -d .github/workflows ]] || fail ".github/workflows directory not found"

for workflow in ci.yml release.yml package-managers.yml; do
  [[ -f ".github/workflows/${workflow}" ]] || fail "Missing workflow: ${workflow}"
  ok "Found workflow: ${workflow}"
done

grep -q "tags:" .github/workflows/release.yml || fail "release.yml is not tag-driven"
grep -q "workflow_dispatch:" .github/workflows/release.yml || fail "release.yml is missing workflow_dispatch"
grep -q "Publish GitHub release" .github/workflows/release.yml || fail "release.yml is missing final release publication"
grep -q "actions/upload-artifact@v4" .github/workflows/release.yml || fail "release.yml is missing artifact upload"
grep -q "actions/download-artifact@v4" .github/workflows/release.yml || fail "release.yml is missing artifact download"
grep -q "APPLE_CERTIFICATE" .github/workflows/release.yml || fail "release.yml is missing macOS signing secrets"
grep -q "WINDOWS_CERTIFICATE" .github/workflows/release.yml || fail "release.yml is missing Windows signing secrets"
grep -q "codesign --verify" .github/workflows/release.yml || fail "release.yml is missing macOS signature verification"
grep -q "Get-AuthenticodeSignature" .github/workflows/release.yml || fail "release.yml is missing Windows signature verification"
ok "release.yml includes tag gating, packaging, signing, and release publication"

grep -q "types:" .github/workflows/package-managers.yml || fail "package-managers.yml is missing release trigger configuration"
grep -q "published" .github/workflows/package-managers.yml || fail "package-managers.yml is not wired to published releases"
ok "package-managers.yml is downstream of release publication"

[[ -f scripts/collect_release_assets.py ]] || fail "scripts/collect_release_assets.py is missing"
[[ -f scripts/verify-icons.sh ]] || warn "scripts/verify-icons.sh is missing"
ok "Release helper scripts are present"

python3 - <<'PY'
import pathlib
import yaml

for path in [
    pathlib.Path('.github/workflows/ci.yml'),
    pathlib.Path('.github/workflows/release.yml'),
    pathlib.Path('.github/workflows/package-managers.yml'),
]:
    yaml.safe_load(path.read_text())
PY
ok "Workflow YAML syntax is valid"

echo
ok "GitHub Actions configuration looks structurally sound"