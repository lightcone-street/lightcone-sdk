#!/usr/bin/env bash
set -euo pipefail

WALLET_DIR="$HOME/.config/solana"

WALLET_RS="$WALLET_DIR/lightcone-sdk-rs.json"
WALLET_TS="$WALLET_DIR/lightcone-sdk-ts.json"
WALLET_PY="$WALLET_DIR/lightcone-sdk-py.json"

if ! command -v solana-keygen &>/dev/null || ! command -v solana &>/dev/null; then
    echo "Error: Solana CLI not found. Install it:" >&2
    echo "  https://docs.solanalabs.com/cli/install" >&2
    exit 1
fi

mkdir -p "$WALLET_DIR"

echo "=== Creating wallets ==="
echo ""

for wallet_path in "$WALLET_RS" "$WALLET_TS" "$WALLET_PY"; do
    if [ -f "$wallet_path" ]; then
        echo "  Exists: $wallet_path"
    else
        solana-keygen new --no-passphrase --outfile "$wallet_path" --silent
        echo "  Created: $wallet_path"
    fi
done

RPC_URL="${SDK_RPC_URL:-https://api.devnet.solana.com}"

PUBKEY_RS=$(solana-keygen pubkey "$WALLET_RS")
PUBKEY_TS=$(solana-keygen pubkey "$WALLET_TS")
PUBKEY_PY=$(solana-keygen pubkey "$WALLET_PY")

echo ""
echo "=== Funding wallets with devnet SOL ==="
echo ""
echo "  Airdropping 2 SOL to $PUBKEY_RS..."
if timeout 30 solana airdrop 2 "$PUBKEY_RS" --url "$RPC_URL" 2>&1 | sed 's/^/    /'; then
    echo ""
    echo "  Transferring 0.5 SOL to $PUBKEY_TS..."
    solana transfer --keypair "$WALLET_RS" "$PUBKEY_TS" 0.5 --url "$RPC_URL" --allow-unfunded-recipient 2>&1 | sed 's/^/    /'
    echo ""
    echo "  Transferring 0.5 SOL to $PUBKEY_PY..."
    solana transfer --keypair "$WALLET_RS" "$PUBKEY_PY" 0.5 --url "$RPC_URL" --allow-unfunded-recipient 2>&1 | sed 's/^/    /'
    echo ""
else
    echo ""
    echo "  Airdrop failed — devnet faucet is often rate-limited."
    echo "  Send devnet SOL manually to each wallet:"
    echo "    $PUBKEY_RS"
    echo "    $PUBKEY_TS"
    echo "    $PUBKEY_PY"
    echo ""
fi

echo "=== Next steps ==="
echo ""
echo "1. Send 100 Lightcone USDC to each wallet:"
echo "    $PUBKEY_RS"
echo "    $PUBKEY_TS"
echo "    $PUBKEY_PY"
echo ""
echo "2. Add these to your shell profile (.bashrc / .zshrc):"
echo ""
echo "  export LIGHTCONE_WALLET_PATH=$WALLET_RS"
echo "  export LIGHTCONE_WALLET_PATH_TS=$WALLET_TS"
echo "  export LIGHTCONE_WALLET_PATH_PYTHON=$WALLET_PY"
