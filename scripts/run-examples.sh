#!/usr/bin/env bash
set -uo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
TIMEOUT=120

if [ -z "${CI:-}" ] && [ -t 1 ]; then
    RED='\033[0;31m'
    GREEN='\033[0;32m'
    YELLOW='\033[0;33m'
    BLUE='\033[0;34m'
    BOLD='\033[1m'
    RESET='\033[0m'
else
    RED='' GREEN='' YELLOW='' BLUE='' BOLD='' RESET=''
fi

usage() {
    echo "Usage: $0 [--sdk rs|ts|py] [--help]"
    echo ""
    echo "Run SDK examples for Rust, TypeScript, and/or Python."
    echo ""
    echo "Options:"
    echo "  --sdk rs|ts|py   Run examples for a single SDK only"
    echo "  --help, -h       Show this help message"
    echo ""
    echo "Environment variables:"
    echo "  LIGHTCONE_ENV              Required. Target environment (local, staging, prod)"
    echo "  LIGHTCONE_WALLET_PATH      Wallet keypair path for Rust examples"
    echo "  LIGHTCONE_WALLET_PATH_TS   Wallet keypair path for TypeScript examples"
    echo "  LIGHTCONE_WALLET_PATH_PYTHON  Wallet keypair path for Python examples"
    echo "  SDK_RPC_URL                Solana RPC URL override (recommended: private RPC to avoid 429s)"
    echo "  SKIP_EXAMPLES              Set to 1 to bypass the pre-commit hook prompt"
    echo ""
    echo "Examples:"
    echo "  LIGHTCONE_ENV=local $0                  # Run all SDKs in parallel"
    echo "  LIGHTCONE_ENV=local $0 --sdk rs         # Run Rust examples only"
    echo "  LIGHTCONE_ENV=staging $0 --sdk ts       # Run TypeScript against staging"
}

sdk_filter=""

while [[ $# -gt 0 ]]; do
    case "$1" in
        --sdk)
            sdk_filter="$2"
            shift 2
            ;;
        --help|-h)
            usage
            exit 0
            ;;
        *)
            echo -e "${RED}Unknown argument: $1${RESET}" >&2
            usage >&2
            exit 1
            ;;
    esac
done

if [ -z "${LIGHTCONE_ENV:-}" ]; then
    echo -e "${RED}Error: LIGHTCONE_ENV is required (local, staging, prod)${RESET}" >&2
    exit 1
fi

if [ -n "$sdk_filter" ]; then
    case "$sdk_filter" in
        rs|ts|py) ;;
        *)
            echo -e "${RED}Error: Unknown SDK '$sdk_filter'. Options: rs, ts, py${RESET}" >&2
            exit 1
            ;;
    esac
    sdks=("$sdk_filter")
else
    sdks=("rs" "ts" "py")
fi

wallet_var_for_sdk() {
    case "$1" in
        rs) echo "LIGHTCONE_WALLET_PATH" ;;
        ts) echo "LIGHTCONE_WALLET_PATH_TS" ;;
        py) echo "LIGHTCONE_WALLET_PATH_PYTHON" ;;
    esac
}

for sdk in "${sdks[@]}"; do
    wallet_var=$(wallet_var_for_sdk "$sdk")
    if [ -z "${!wallet_var:-}" ]; then
        echo -e "${RED}Error: $wallet_var is required${RESET}" >&2
        echo "Each SDK needs its own wallet to avoid race conditions when running in parallel." >&2
        echo "Set: LIGHTCONE_WALLET_PATH, LIGHTCONE_WALLET_PATH_TS, LIGHTCONE_WALLET_PATH_PYTHON" >&2
        exit 1
    fi
    wallet_path="${!wallet_var}"
    if [ ! -f "$wallet_path" ]; then
        echo -e "${RED}Error: Wallet file not found: $wallet_path ($wallet_var)${RESET}" >&2
        exit 1
    fi
done

should_skip() {
    local name="$1"
    case "$name" in
        # The peer transfer remains manual. Conversion runs against each SDK's
        # dedicated wallet locally and in enabled staging-CI suites, never production.
        deposit_token_balances) return 0 ;;
        wsol_conversion)
            if [ "$LIGHTCONE_ENV" = "local" ]; then
                return 1
            fi
            if [ -n "${CI:-}" ] && [ "$LIGHTCONE_ENV" = "staging" ]; then
                return 1
            fi
            return 0
            ;;
        admin_*|faucet_claim|common) return 0 ;;
        *) return 1 ;;
    esac
}

# Auto-detect mkcert root CA for Node.js TLS when running locally.
# Node.js doesn't reliably read the system CA store on all platforms.
mkcert_ca=""
if [ "$LIGHTCONE_ENV" = "local" ]; then
    if [ -f "$HOME/.local/share/mkcert/rootCA.pem" ]; then
        mkcert_ca="$HOME/.local/share/mkcert/rootCA.pem"
    elif [ -f "/etc/ca-certificates/trust-source/anchors/mkcert-root.crt" ]; then
        mkcert_ca="/etc/ca-certificates/trust-source/anchors/mkcert-root.crt"
    fi
fi

# Determine Python command — use uv locally, plain python in CI
python_cmd="python"
if [ -z "${CI:-}" ] && command -v uv &>/dev/null; then
    python_cmd="uv run python"
elif [ -z "${CI:-}" ]; then
    if ! python -c "import solders" 2>/dev/null; then
        echo -e "${RED}Error: Python dependencies not installed and uv not found.${RESET}" >&2
        echo "Install uv (https://docs.astral.sh/uv/) and run: cd python && uv sync" >&2
        echo "Or activate a virtualenv with dependencies installed." >&2
        exit 1
    fi
fi

run_sdk() {
    local sdk="$1"
    local results_file="$2"
    local failures_file="${results_file}.failures"
    local passed=0 failed=0 skipped=0
    > "$failures_file"

    local wallet_var wallet_path sdk_label sdk_dir example_ext
    wallet_var=$(wallet_var_for_sdk "$sdk")
    wallet_path="${!wallet_var}"

    case "$sdk" in
        rs) sdk_label="Rust";       sdk_dir="$SCRIPT_DIR/rust";       example_ext="rs" ;;
        ts) sdk_label="TypeScript";  sdk_dir="$SCRIPT_DIR/typescript"; example_ext="ts" ;;
        py) sdk_label="Python";      sdk_dir="$SCRIPT_DIR/python";     example_ext="py" ;;
    esac

    echo -e "${BOLD}═══ $sdk_label Examples ═══${RESET}"
    echo ""

    for file in "$sdk_dir/examples"/*."$example_ext"; do
        [ -f "$file" ] || continue
        local name
        name=$(basename "$file" ".$example_ext")

        if should_skip "$name"; then
            if [ -n "${CI:-}" ]; then
                echo "::group::$name (skipped)"
                echo "::endgroup::"
            fi
            ((skipped++))
            continue
        fi

        [ -n "${CI:-}" ] && echo "::group::$name"
        echo -e "${BLUE}▶ $sdk/$name${RESET}"

        local run_exit=0
        case "$sdk" in
            rs)
                (
                    cd "$sdk_dir" || exit
                    if [ "$name" = "wsol_conversion" ] && [ "$LIGHTCONE_ENV" = "local" ]; then
                        unset SDK_API_URL SDK_WS_URL SDK_PROGRAM_ID
                    fi
                    LIGHTCONE_WALLET_PATH="$wallet_path" timeout "$TIMEOUT" \
                        cargo run --example "$name" --features "native,trigger_orders"
                ) || run_exit=$?
                ;;
            ts)
                (
                    cd "$sdk_dir" || exit
                    if [ "$name" = "wsol_conversion" ] && [ "$LIGHTCONE_ENV" = "local" ]; then
                        unset SDK_API_URL SDK_WS_URL SDK_PROGRAM_ID
                    fi
                    LIGHTCONE_WALLET_PATH="$wallet_path" \
                        NODE_EXTRA_CA_CERTS="${mkcert_ca:-}" \
                        timeout "$TIMEOUT" npx tsx "examples/$name.ts"
                ) || run_exit=$?
                ;;
            py)
                (
                    cd "$sdk_dir" || exit
                    if [ "$name" = "wsol_conversion" ] && [ "$LIGHTCONE_ENV" = "local" ]; then
                        unset SDK_API_URL SDK_WS_URL SDK_PROGRAM_ID
                    fi
                    LIGHTCONE_WALLET_PATH="$wallet_path" timeout "$TIMEOUT" \
                        $python_cmd "examples/$name.py"
                ) || run_exit=$?
                ;;
        esac

        if [ "$run_exit" -eq 0 ]; then
            echo -e "${GREEN}  ✓ passed${RESET}"
            ((passed++))
        else
            echo -e "${RED}  ✗ failed (exit $run_exit)${RESET}"
            echo "$sdk/$name" >> "$failures_file"
            ((failed++))
        fi

        [ -n "${CI:-}" ] && echo "::endgroup::"
    done

    echo "$passed $failed $skipped" > "$results_file"
    [ "$failed" -gt 0 ] && return 1 || return 0
}

print_summary() {
    local total_passed="$1" total_failed="$2" total_skipped="$3"
    shift 3
    local failure_files=("$@")

    echo ""
    echo -e "${BOLD}═══════════════════════════════════════════${RESET}"
    echo -e "${BOLD}  Examples Summary${RESET}"
    echo -e "${BOLD}═══════════════════════════════════════════${RESET}"
    echo -e "  ${GREEN}Passed:  $total_passed${RESET}"
    if [ "$total_failed" -gt 0 ]; then
        echo -e "  ${RED}Failed:  $total_failed${RESET}"
    else
        echo -e "  Failed:  0"
    fi
    echo -e "  ${YELLOW}Skipped: $total_skipped${RESET}"

    if [ "$total_failed" -gt 0 ]; then
        echo ""
        echo -e "  ${RED}Failed examples:${RESET}"
        for f in "${failure_files[@]}"; do
            if [ -f "$f" ] && [ -s "$f" ]; then
                while IFS= read -r name; do
                    echo -e "    ${RED}✗ $name${RESET}"
                done < "$f"
            fi
        done
    fi

    echo -e "${BOLD}═══════════════════════════════════════════${RESET}"
}

tmpdir=$(mktemp -d)
trap 'rm -rf "$tmpdir"' EXIT

if [ ${#sdks[@]} -eq 1 ]; then
    run_sdk "${sdks[0]}" "$tmpdir/results"
    sdk_exit=$?
    read -r total_passed total_failed total_skipped < "$tmpdir/results"
    print_summary "$total_passed" "$total_failed" "$total_skipped" "$tmpdir/results.failures"
    exit $sdk_exit
fi

echo -e "${BOLD}Running examples for all SDKs in parallel...${RESET}"
echo ""

pids=()
for sdk in "${sdks[@]}"; do
    (run_sdk "$sdk" "$tmpdir/$sdk.results" > "$tmpdir/$sdk.log" 2>&1) &
    pids+=($!)
done

any_failed=0
for i in "${!pids[@]}"; do
    if ! wait "${pids[$i]}" 2>/dev/null; then
        any_failed=1
    fi
    sdk="${sdks[$i]}"
    if [ -f "$tmpdir/$sdk.log" ]; then
        cat "$tmpdir/$sdk.log"
        echo ""
    fi
done

total_passed=0 total_failed=0 total_skipped=0
for sdk in "${sdks[@]}"; do
    if [ -f "$tmpdir/$sdk.results" ]; then
        read -r passed failed skipped < "$tmpdir/$sdk.results"
        total_passed=$((total_passed + passed))
        total_failed=$((total_failed + failed))
        total_skipped=$((total_skipped + skipped))
    fi
done

failure_files=()
for sdk in "${sdks[@]}"; do
    failure_files+=("$tmpdir/$sdk.results.failures")
done

print_summary "$total_passed" "$total_failed" "$total_skipped" "${failure_files[@]}"
exit $any_failed
