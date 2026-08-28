/**
 * Fund-moving local/staging example: plan and confirm a native-SOL withdrawal
 * without closing the persistent canonical WSOL account, then refresh a complete
 * wallet snapshot covering the confirmation slot.
 */
import { tradingWallet } from "../src/auth";
import {
  asPubkeyStr,
  WalletDepositBalancesState,
} from "../src";
import { restClient, getKeypair, login, runExample } from "./common";

/** Native SOL transferred per run, in lamports (0.001 SOL). */
const WITHDRAW_AMOUNT_LAMPORTS = 1_000_000n;

/** Run the fund-moving lifecycle against configured non-production SDK wallets. */
async function main() {
  requireNonProduction();
  // SDK wallets form a stable funding cycle: Rust -> TypeScript -> Python -> Rust.
  // The existing peer path avoids a recipient-specific setting and repeated top-offs.
  const recipient = getKeypair("LIGHTCONE_WALLET_PATH_PYTHON").publicKey;
  const client = restClient();
  const keypair = getKeypair("LIGHTCONE_WALLET_PATH_TS");
  if (recipient.equals(keypair.publicKey)) {
    throw new Error("TypeScript and Python SDK wallet paths must identify peers");
  }
  const session = await login(client, keypair);
  const wallet = tradingWallet(session.user, session.auth_method);

  const walletAddress = asPubkeyStr(wallet);
  const state = new WalletDepositBalancesState();
  const ws = client.ws();
  // Install the reducer before subscribing so the complete baseline cannot race
  // the listener; pre-baseline balance events remain safely ignored by state.
  const removeListener = ws.on((event) => {
    if (
      event.type === "Message" &&
      event.message.type === "wallet_deposit_balances"
    ) {
      state.applyEvent(event.message.data);
    }
  });
  await ws.connect();
  try {
    ws.subscribe({
      type: "wallet_deposit_balances",
      wallet_address: walletAddress,
    });
    await waitForState(
      () => state.contextSlot !== undefined,
      "initial wallet balance snapshot"
    );

    console.log("wallet:", wallet);
    console.log("context slot:", state.contextSlot);
    console.log("native SOL:", state.nativeSolBalance);
    console.log("native + canonical WSOL:", state.combinedSolBalance());
    console.log("tracked balances:", state.balances.size);

    const entries = [...state.balances.values()].sort((a, b) =>
      a.symbol.localeCompare(b.symbol)
    );
    for (const balance of entries) {
      console.log(
        `  ${balance.symbol.padStart(8)}  ${balance.mint.padEnd(42)}  idle=${balance.idle}`
      );
    }

    client.setSigningStrategy({ type: "native", keypair });
    const plan = await client.positions().planNativeSolWithdrawal(
      recipient,
      WITHDRAW_AMOUNT_LAMPORTS,
      state,
      false
    );
    console.log("spendable SOL lamports:", plan.availability.spendableLamports);
    console.log("reserved SOL lamports:", plan.availability.reserveLamports);
    const confirmed = await client.signAndSubmitPreparedTxConfirmedWithSlot(
      plan.transaction
    );
    console.log(
      `withdrew ${WITHDRAW_AMOUNT_LAMPORTS} lamports to ${recipient.toBase58()}:`,
      confirmed
    );

    // Confirmation does not mutate cached state. Observe the wallet stream at
    // or beyond the processing slot, then replace it with a complete slot-bounded
    // REST snapshot before publishing post-transaction state.
    await waitForState(
      () =>
        state.contextSlot !== undefined && state.contextSlot >= confirmed.slot,
      "post-withdraw wallet update"
    );
    const snapshot = await client
      .positions()
      .depositTokenBalances(confirmed.slot);
    state.applyRestSnapshot(walletAddress, snapshot);
    console.log(
      "post-withdraw native + canonical WSOL:",
      state.combinedSolBalance()
    );

    ws.unsubscribe({
      type: "wallet_deposit_balances",
      wallet_address: walletAddress,
    });
  } finally {
    // Disconnect is the definitive teardown when an earlier error skips the
    // explicit wire unsubscribe.
    removeListener();
    await ws.disconnect();
  }
}

async function waitForState(
  predicate: () => boolean,
  description: string
): Promise<void> {
  const deadline = Date.now() + 10_000;
  while (!predicate()) {
    if (Date.now() >= deadline) {
      throw new Error(`timed out waiting for ${description}`);
    }
    await new Promise((resolve) => setTimeout(resolve, 50));
  }
}

/** Reject production or endpoint overrides before any fund-moving side effect. */
function requireNonProduction(): void {
  const environment = process.env.LIGHTCONE_ENV?.toLowerCase() ?? "prod";
  if (environment !== "local" && environment !== "staging") {
    throw new Error("SOL action examples are disabled in production");
  }

  // Overrides can repoint a safe environment label at production infrastructure.
  const overrideName = [
    "SDK_API_URL",
    "SDK_WS_URL",
    "SDK_RPC_URL",
    "SDK_PROGRAM_ID",
  ].find((name) => process.env[name] !== undefined);
  if (overrideName) {
    throw new Error(
      `SOL action examples require built-in local/staging configuration; unset ${overrideName}`
    );
  }
}

void runExample(main);
