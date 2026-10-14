#!/usr/bin/env bash
set -euo pipefail

# Promote an existing prerelease, or increment a stable version's patch.
if [[ $# != 1 || ! "$1" =~ ^(0|[1-9][0-9]*)\.(0|[1-9][0-9]*)\.(0|[1-9][0-9]*)(-[0-9A-Za-z]+([.-][0-9A-Za-z]+)*)?(\+[0-9A-Za-z-]+(\.[0-9A-Za-z-]+)*)?$ ]]; then
  echo "Expected a stable or prerelease semantic version" >&2
  exit 1
fi

major="${BASH_REMATCH[1]}"
minor="${BASH_REMATCH[2]}"
patch="${BASH_REMATCH[3]}"
prerelease="${BASH_REMATCH[4]}"
if [[ -z "$prerelease" ]]; then
  patch="$((patch + 1))"
fi
printf '%s.%s.%s\n' "$major" "$minor" "$patch"
