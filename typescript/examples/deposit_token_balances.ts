/**
 * Fund-moving local/staging example: wrap 0.1 SOL, wait for authoritative
 * wallet state, then close the entire canonical WSOL account, including any
 * pre-existing balance. A failure after submission does not prove rollback;
 * inspect authoritative balances before retrying because funds may have moved.
 */
import { tradingWallet } from "../src/auth";
import {
  asPubkeyStr,
  shared,
  WRAPPED_SOL_MINT,
  WalletDepositBalancesState,
} from "../src";
import { restClient, getKeypair, login, runExample } from "./common";

const WRAP_AMOUNT = "0.1";

async function main() {
  requireNonProduction();
  const client = restClient();
  const keypair = getKeypair();
  const session = await login(client, keypair);
  const wallet = tradingWallet(session.user, session.auth_method);

  const walletAddress = asPubkeyStr(wallet);
  const state = new WalletDepositBalancesState();
  const ws = client.ws();
  // Install the reducer before subscribing so the complete baseline cannot race
  // the listener; pre-baseline component events remain safely ignored by state.
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
    // Confirmation does not mutate state. Wait for authoritative WS changes
    // before using the refreshed cache to authorize the next conversion.
    const expectedWsolLamports =
      canonicalWsolLamports(state) + shared.exactScaledInteger(WRAP_AMOUNT, 9);
    const wrapSignature = await client.positions().wrapSol(WRAP_AMOUNT, state);
    console.log(`wrapped ${WRAP_AMOUNT} SOL:`, wrapSignature);
    await waitForState(
      () => canonicalWsolLamports(state) === expectedWsolLamports,
      "post-wrap WSOL update"
    );
    console.log(
      "post-wrap native + canonical WSOL:",
      state.combinedSolBalance()
    );

    console.log(
      "closing the full canonical WSOL account; partial unwrap is not supported"
    );
    const unwrapSignature = await client.positions().unwrapWsol(state);
    console.log("unwrapped full canonical WSOL account:", unwrapSignature);
    await waitForState(
      () => canonicalWsolLamports(state) === 0n,
      "post-unwrap WSOL removal"
    );
    console.log(
      "post-unwrap native + canonical WSOL:",
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
  // Conversion predicates compare exact canonical WSOL lamports, so native-only
  // or unrelated positive updates cannot release the barrier.
  const deadline = Date.now() + 10_000;
  while (!predicate()) {
    if (Date.now() >= deadline) {
      throw new Error(`timed out waiting for ${description}`);
    }
    await new Promise((resolve) => setTimeout(resolve, 50));
  }
}

function canonicalWsolLamports(state: WalletDepositBalancesState): bigint {
  const balance = state.balances.get(WRAPPED_SOL_MINT);
  return balance ? shared.exactScaledInteger(balance.idle, 9) : 0n;
}

function requireNonProduction(): void {
  const environment = process.env.LIGHTCONE_ENV?.toLowerCase() ?? "prod";
  if (environment !== "local" && environment !== "staging") {
    throw new Error("SOL conversion examples are disabled in production");
  }
}

void runExample(main);
