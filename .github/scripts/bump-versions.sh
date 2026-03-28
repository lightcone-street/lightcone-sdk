#!/usr/bin/env bash
set -euo pipefail

# Increment the patch component of a semver string: "0.4.3" -> "0.4.4"
bump_patch() {
  local version="$1"
  local major minor patch
  IFS='.' read -r major minor patch <<< "$version"
  echo "${major}.${minor}.$((patch + 1))"
}

# ── Rust (Cargo.toml) ──
RUST_FILE="rust/Cargo.toml"
RUST_OLD=$(grep '^version = ' "$RUST_FILE" | head -1 | sed 's/version = "\(.*\)"/\1/')
RUST_NEW=$(bump_patch "$RUST_OLD")
sed -i "0,/^version = \"${RUST_OLD}\"/s//version = \"${RUST_NEW}\"/" "$RUST_FILE"
echo "Rust: $RUST_OLD -> $RUST_NEW"
(cd rust && cargo generate-lockfile)

# ── TypeScript (package.json) ──
TS_FILE="typescript/package.json"
TS_OLD=$(node -p "require('./$TS_FILE').version")
TS_NEW=$(bump_patch "$TS_OLD")
node -e "
  const fs = require('fs');
  const pkg = JSON.parse(fs.readFileSync('$TS_FILE', 'utf8'));
  pkg.version = '$TS_NEW';
  fs.writeFileSync('$TS_FILE', JSON.stringify(pkg, null, 2) + '\n');
"
echo "TypeScript: $TS_OLD -> $TS_NEW"
(cd typescript && npm install --package-lock-only)

# ── Python (pyproject.toml) ──
PY_FILE="python/pyproject.toml"
PY_OLD=$(grep '^version = ' "$PY_FILE" | head -1 | sed 's/version = "\(.*\)"/\1/')
PY_NEW=$(bump_patch "$PY_OLD")
sed -i "s/^version = \"${PY_OLD}\"/version = \"${PY_NEW}\"/" "$PY_FILE"
echo "Python: $PY_OLD -> $PY_NEW"

# ── Set outputs for GitHub Actions ──
echo "rust_version=$RUST_NEW" >> "$GITHUB_OUTPUT"
echo "ts_version=$TS_NEW" >> "$GITHUB_OUTPUT"
echo "python_version=$PY_NEW" >> "$GITHUB_OUTPUT"
