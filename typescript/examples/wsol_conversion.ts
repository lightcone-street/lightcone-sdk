/**
 * Fund-moving example for explicit native-keypair WSOL conversion.
 *
 * The flow wraps an exact amount, submits only freshly prepared messages,
 * retains each frozen projection until a complete covering refresh, then closes
 * the wallet's entire canonical WSOL account without an interactive pause.
 * Failures exit without automatic submission retries.
 */
import { tradingWallet } from "../src/auth";
import {
  asPubkeyStr,
  type DepositTokenBalancesSnapshot,
  type SolActionPlan,
  type SolBalanceComponents,
  WalletDepositBalancesState,
} from "../src";
import { getKeypair, login, restClient, runExample } from "./common";

/** Exact native amount converted to canonical WSOL per run (0.001 SOL). */
const WRAP_AMOUNT_LAMPORTS = 1_000_000n;

/** Maximum wait for the initial complete wallet balance snapshot. */
const INITIAL_SNAPSHOT_TIMEOUT_MS = 10_000;

/** Run the guarded wrap, full-account close, and covering-refresh lifecycle. */
async function main(): Promise<void> {
  requireNonProduction();
  const client = restClient();
  const keypair = getKeypair("LIGHTCONE_WALLET_PATH_TS");
  const session = await login(client, keypair);
  const wallet = tradingWallet(session.user, session.auth_method);
  if (wallet !== keypair.publicKey.toBase58()) {
    throw new Error(
      "native keypair does not control the authenticated Trading Wallet",
    );
  }
  client.setSigningStrategy({ type: "native", keypair });

  const walletAddress = asPubkeyStr(wallet);
  const state = new WalletDepositBalancesState();
  const ws = client.ws();
  // Attach before subscribing so the complete baseline cannot race the reducer.
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
      "initial wallet balance snapshot",
    );

    console.log("Trading Wallet:", wallet);
    console.log("pre-wrap components:", state.solComponents());

    // Preview is informational; rebuild live account, rent, blockhash, and fee
    // authority immediately before signing the unchanged prepared message.
    const wrapPreview = await client
      .positions()
      .planWrapSol(WRAP_AMOUNT_LAMPORTS, state);
    console.log("exact wrap lamports:", WRAP_AMOUNT_LAMPORTS);
    console.log("preview wrap fee lamports:", wrapPreview.costs.feeLamports);
    console.log(
      "preview wrap upfront rent lamports:",
      wrapPreview.costs.upfrontRentLamports,
    );
    const wrapPlan = await client
      .positions()
      .planWrapSol(WRAP_AMOUNT_LAMPORTS, state);
    console.log("final wrap fee lamports:", wrapPlan.costs.feeLamports);
    console.log(
      "final wrap upfront rent lamports:",
      wrapPlan.costs.upfrontRentLamports,
    );
    const frozenWrapProjection = projectedComponents(wrapPlan);
    const wrapped = await submitPreparedOnce(
      wrapPlan.transaction,
      (transaction) =>
        client.signAndSubmitPreparedTxConfirmedWithSlot(transaction),
    );
    console.log("wrap confirmed:", wrapped);
    console.log("frozen post-wrap projection:", frozenWrapProjection);

    // No further action is authorized until a complete snapshot covers this slot.
    await refreshCoveringSlot(
      state,
      walletAddress,
      wrapped.slot,
      (slot) => client.positions().depositTokenBalances(slot),
    );
    console.log("authoritative post-wrap components:", state.solComponents());

    // There is deliberately no prompt or pause: rebuild against refreshed state,
    // display the destructive scope, then submit this exact prepared transaction.
    const unwrapPlan = await client.positions().planUnwrapWsolAll(state);
    const returnedAccountLamports =
      unwrapPlan.expectedDelta.nativeLamports + unwrapPlan.costs.feeLamports;
    console.warn(
      "WARNING: unwrap-all closes the Trading Wallet's entire canonical WSOL account.",
    );
    console.warn(
      "All existing canonical WSOL is returned; a future WSOL action may pay account rent again.",
    );
    console.log("pre-close components:", unwrapPlan.availability.components);
    console.log("unwrap fee lamports:", unwrapPlan.costs.feeLamports);
    console.log("full account lamports returned:", returnedAccountLamports);
    // No pause or cached preview crosses the destructive boundary. Submit only
    // this final prepared message; errors exit without retrying it.
    const frozenUnwrapProjection = projectedComponents(unwrapPlan);
    const unwrapped = await submitPreparedOnce(
      unwrapPlan.transaction,
      (transaction) =>
        client.signAndSubmitPreparedTxConfirmedWithSlot(transaction),
    );
    console.log("unwrap-all confirmed:", unwrapped);
    console.log("frozen post-unwrap projection:", frozenUnwrapProjection);

    // Retain the final projection and deny another action until REST supplies a
    // complete cross-component snapshot at or beyond the processing slot.
    await refreshCoveringSlot(
      state,
      walletAddress,
      unwrapped.slot,
      (slot) => client.positions().depositTokenBalances(slot),
    );
    console.log("authoritative final components:", state.solComponents());

    ws.unsubscribe({
      type: "wallet_deposit_balances",
      wallet_address: walletAddress,
    });
  } finally {
    removeListener();
    await ws.disconnect();
  }
}

/**
 * Apply one plan delta without mutating authoritative wallet state.
 * Throws if either projected component would become negative.
 */
function projectedComponents(plan: SolActionPlan): SolBalanceComponents {
  const nativeLamports =
    plan.availability.components.nativeLamports +
    plan.expectedDelta.nativeLamports;
  const canonicalWsolLamports =
    plan.availability.components.canonicalWsolLamports +
    plan.expectedDelta.canonicalWsolLamports;
  if (nativeLamports < 0n || canonicalWsolLamports < 0n) {
    throw new Error("planner produced a negative frozen SOL projection");
  }
  return { nativeLamports, canonicalWsolLamports };
}

/** Submit one prepared message exactly once and propagate any uncertain failure. */
export async function submitPreparedOnce<TTransaction, TConfirmation>(
  transaction: TTransaction,
  submit: (transaction: TTransaction) => Promise<TConfirmation>,
): Promise<TConfirmation> {
  return submit(transaction);
}

/** Replace component observations with one authoritative covering REST snapshot. */
export async function refreshCoveringSlot(
  state: WalletDepositBalancesState,
  walletAddress: ReturnType<typeof asPubkeyStr>,
  confirmedSlot: number,
  fetchSnapshot: (minimumSlot: number) => Promise<DepositTokenBalancesSnapshot>,
): Promise<void> {
  const snapshot = await fetchSnapshot(confirmedSlot);
  validateCoveringSnapshotSlot(snapshot.context_slot, confirmedSlot);
  state.applyRestSnapshot(walletAddress, snapshot);
}

/** Reject a REST snapshot that cannot restore authority after confirmation. */
export function validateCoveringSnapshotSlot(
  snapshotSlot: number,
  confirmedSlot: number,
): void {
  if (snapshotSlot < confirmedSlot) {
    throw new Error(
      `wallet snapshot slot ${snapshotSlot} is below confirmed slot ${confirmedSlot}`,
    );
  }
}

/**
 * Poll local reducer state without transaction retry behavior.
 * Throws when the predicate remains false through the configured timeout.
 */
async function waitForState(
  predicate: () => boolean,
  description: string,
): Promise<void> {
  const deadline = Date.now() + INITIAL_SNAPSHOT_TIMEOUT_MS;
  while (!predicate()) {
    if (Date.now() >= deadline) {
      throw new Error(`timed out waiting for ${description}`);
    }
    await new Promise((resolve) => setTimeout(resolve, 50));
  }
}

/** Reject production and unsafe endpoint overrides before fund-moving work. */
export function requireNonProduction(
  environmentVariables: Readonly<
    Record<string, string | undefined>
  > = process.env,
): void {
  const environment =
    environmentVariables.LIGHTCONE_ENV?.toLowerCase() ?? "prod";
  if (environment !== "local" && environment !== "staging") {
    throw new Error("WSOL conversion example is disabled in production");
  }
  const ci = environmentVariables.CI !== undefined;
  const overrideName = [
    "SDK_API_URL",
    "SDK_WS_URL",
    "SDK_RPC_URL",
    "SDK_PROGRAM_ID",
  ].find(
    (name) =>
      environmentVariables[name] !== undefined &&
      !(
        (environment === "local" && name === "SDK_RPC_URL") ||
        (environment === "staging" &&
          ci &&
          (name === "SDK_API_URL" ||
            name === "SDK_WS_URL" ||
            name === "SDK_RPC_URL"))
      ),
  );
  if (overrideName) {
    throw new Error(
      `WSOL conversion requires built-in API, WebSocket, and program configuration; unset ${overrideName}`,
    );
  }
}

if (require.main === module) {
  void runExample(main);
}
