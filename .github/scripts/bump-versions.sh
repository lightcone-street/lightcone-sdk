#!/usr/bin/env bash
set -euo pipefail

# Existing prereleases promote their core version; stable versions bump patch.
script_dir="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")" && pwd)"
next_release_version() {
  bash "$script_dir/next-release-version.sh" "$1"
}

# ── Rust (Cargo.toml) ──
RUST_FILE="rust/Cargo.toml"
RUST_OLD=$(grep '^version = ' "$RUST_FILE" | head -1 | sed 's/version = "\(.*\)"/\1/')
RUST_NEW=$(next_release_version "$RUST_OLD")
sed -i "0,/^version = \"${RUST_OLD}\"/s//version = \"${RUST_NEW}\"/" "$RUST_FILE"
echo "Rust: $RUST_OLD -> $RUST_NEW"
sed -i "/^name = \"lightcone\"/{n;s/version = \"${RUST_OLD}\"/version = \"${RUST_NEW}\"/;}" rust/Cargo.lock

# ── TypeScript (package.json) ──
TS_FILE="typescript/package.json"
TS_OLD=$(node -p "require('./$TS_FILE').version")
TS_NEW=$(next_release_version "$TS_OLD")
node -e "
  const fs = require('fs');
  const pkg = JSON.parse(fs.readFileSync('$TS_FILE', 'utf8'));
  pkg.version = '$TS_NEW';
  fs.writeFileSync('$TS_FILE', JSON.stringify(pkg, null, 2) + '\n');
"
echo "TypeScript: $TS_OLD -> $TS_NEW"
sed -i "s/\"version\": \"${TS_OLD}\"/\"version\": \"${TS_NEW}\"/g" typescript/package-lock.json

# ── Python (pyproject.toml) ──
PY_FILE="python/pyproject.toml"
PY_OLD=$(grep '^version = ' "$PY_FILE" | head -1 | sed 's/version = "\(.*\)"/\1/')
PY_NEW=$(next_release_version "$PY_OLD")
sed -i "s/^version = \"${PY_OLD}\"/version = \"${PY_NEW}\"/" "$PY_FILE"
echo "Python: $PY_OLD -> $PY_NEW"
# Only the editable project's version changes; dependency pins remain intact.
sed -i "/^name = \"lightcone-sdk\"$/{n;s/^version = \"${PY_OLD}\"$/version = \"${PY_NEW}\"/;}" python/uv.lock

# ── Set outputs for GitHub Actions ──
echo "rust_version=$RUST_NEW" >> "$GITHUB_OUTPUT"
echo "ts_version=$TS_NEW" >> "$GITHUB_OUTPUT"
echo "python_version=$PY_NEW" >> "$GITHUB_OUTPUT"
