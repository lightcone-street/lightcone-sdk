#!/usr/bin/env bash
set -euo pipefail

script_dir="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")" && pwd)"
next_version() { bash "$script_dir/next-release-version.sh" "$1"; }

test "$(next_version 0.8.2)" = 0.8.3
test "$(next_version 0.10.0-rc.1)" = 0.10.0
test "$(next_version 1.0.0-alpha.2)" = 1.0.0
test "$(next_version 2.9.99)" = 2.9.100
for invalid in '' 0.8 0.08.2 0.8.x '0.8.2;exit' 0.8.2-; do
  if next_version "$invalid" >/dev/null 2>&1; then
    echo "Unexpectedly accepted invalid version: $invalid" >&2
    exit 1
  fi
done
echo 'Release version checks passed'
