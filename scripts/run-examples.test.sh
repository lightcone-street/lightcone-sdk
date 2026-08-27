#!/usr/bin/env bash
set -euo pipefail

repo_dir="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
tmpdir=$(mktemp -d)
trap 'rm -rf "$tmpdir"' EXIT

mkdir "$tmpdir/bin"
touch "$tmpdir/wallet.json"

cat > "$tmpdir/bin/timeout" <<'EOF'
#!/usr/bin/env bash
set -euo pipefail

case "$*" in
    *"cargo run --example wsol_conversion"*) sdk=rs ;;
    *"tsx examples/wsol_conversion.ts"*) sdk=ts ;;
    *"python examples/wsol_conversion.py"*) sdk=py ;;
    *) exit 0 ;;
esac

[[ "${SDK_RPC_URL:-}" == "https://runner-probe.invalid" ]]
[[ -z "${SDK_API_URL+x}" ]]
[[ -z "${SDK_WS_URL+x}" ]]
[[ -z "${SDK_PROGRAM_ID+x}" ]]
touch "$RUNNER_PROBE_RESULT/$sdk"
EOF
chmod +x "$tmpdir/bin/timeout"

mkdir "$tmpdir/result"
for sdk in rs ts py; do
    CI=1 \
    LIGHTCONE_ENV=local \
    LIGHTCONE_WALLET_PATH="$tmpdir/wallet.json" \
    LIGHTCONE_WALLET_PATH_TS="$tmpdir/wallet.json" \
    LIGHTCONE_WALLET_PATH_PYTHON="$tmpdir/wallet.json" \
    SDK_RPC_URL=https://runner-probe.invalid \
    SDK_API_URL=https://api-probe.invalid \
    SDK_WS_URL=wss://ws-probe.invalid \
    SDK_PROGRAM_ID=ProgramProbe111111111111111111111111111111 \
    RUNNER_PROBE_RESULT="$tmpdir/result" \
    PATH="$tmpdir/bin:$PATH" \
        "$repo_dir/scripts/run-examples.sh" --sdk "$sdk"
done

for sdk in rs ts py; do
    if [[ ! -f "$tmpdir/result/$sdk" ]]; then
        echo "$sdk/wsol_conversion was not observed at the runner subprocess boundary" >&2
        exit 1
    fi
done
