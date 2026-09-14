#!/usr/bin/env bash
set -euo pipefail

script_dir="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")" && pwd)"
repo_dir="$(cd -- "$script_dir/../.." && pwd)"
test_dir="$(mktemp -d)"
trap 'rm -rf -- "$test_dir"' EXIT
mkdir -p "$test_dir/rust" "$test_dir/typescript" "$test_dir/python"
for file in rust/Cargo.toml rust/Cargo.lock typescript/package.json typescript/package-lock.json python/pyproject.toml python/uv.lock; do
  cp "$repo_dir/$file" "$test_dir/$file"
done
cd "$test_dir"
GITHUB_OUTPUT="$test_dir/outputs" bash "$script_dir/bump-versions.sh"
python3 - "$repo_dir" <<'PYTEST'
import sys
import tomllib
from pathlib import Path

original = tomllib.loads((Path(sys.argv[1]) / "python/uv.lock").read_text())
updated = tomllib.loads(Path("python/uv.lock").read_text())
project = tomllib.loads(Path("python/pyproject.toml").read_text())["project"]
root_package = next(package for package in updated["package"] if package["name"] == project["name"])
assert root_package["version"] == project["version"]
assert root_package["source"] == {"editable": "."}
old_root = next(package for package in original["package"] if package["name"] == project["name"])
assert root_package["version"] != old_root["version"]
root_package["version"] = old_root["version"]
assert updated == original, "version bump changed dependency resolution"
PYTEST
echo 'Version bump preserves synchronized Python lock metadata'
