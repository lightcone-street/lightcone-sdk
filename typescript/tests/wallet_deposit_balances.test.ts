import assert from "node:assert/strict";
import { describe, it } from "node:test";
import type { DepositTokenBalance } from "../src/domain/position";
import {
  WRAPPED_SOL_MINT,
  WalletDepositBalancesState,
} from "../src/domain/position/state";
import type { PubkeyStr } from "../src/shared";
import {
  parseMessageIn,
  subscribeWalletDepositBalances,
  unsubscribeWalletDepositBalances,
} from "../src/ws";
import { subscriptionKey, unsubscribeMatches } from "../src/ws/subscriptions";

const wallet = "WalletA" as PubkeyStr;

function balance(mint: PubkeyStr, idle: string): DepositTokenBalance {
  return {
    mint,
    idle,
    symbol: "WSOL",
    name: "Wrapped SOL",
  };
}

describe("wallet deposit balance wire contract", () => {
  it("decodes the outer channel and all nested event fields", () => {
    const snapshot = parseMessageIn(
      JSON.stringify({
        type: "wallet_deposit_balances",
        version: 0.1,
        data: {
          event_type: "wallet_deposit_balance_snapshot",
          wallet_address: wallet,
          context_slot: 123,
          balances: {},
          native_sol_balance: "1.234567890",
        },
      })
    );
    assert.equal(snapshot.type, "wallet_deposit_balances");
    if (snapshot.type !== "wallet_deposit_balances") return;
    assert.equal(snapshot.data.event_type, "wallet_deposit_balance_snapshot");
    assert.equal(snapshot.data.wallet_address, wallet);

    const nullableIcons = parseMessageIn(
      JSON.stringify({
        type: "wallet_deposit_balances",
        version: 0.1,
        data: {
          event_type: "wallet_deposit_balance_update",
          wallet_address: wallet,
          context_slot: 123,
          balance: {
            ...balance("MintA" as PubkeyStr, "1.000000000"),
            icon_url_low: null,
          },
        },
      })
    );
    assert.equal(nullableIcons.type, "wallet_deposit_balances");
    if (
      nullableIcons.type === "wallet_deposit_balances" &&
      nullableIcons.data.event_type === "wallet_deposit_balance_update"
    ) {
      assert.equal(nullableIcons.data.balance.icon_url_low, null);
    }

    const update = parseMessageIn(
      JSON.stringify({
        type: "wallet_deposit_balances",
        version: 0.1,
        data: {
          event_type: "wallet_deposit_balance_update",
          wallet_address: wallet,
          context_slot: 124,
          balance: balance("MintA" as PubkeyStr, "1.000000000"),
        },
      })
    );
    assert.equal(update.type, "wallet_deposit_balances");
    if (update.type !== "wallet_deposit_balances") return;
    assert.equal(update.data.event_type, "wallet_deposit_balance_update");
    if (update.data.event_type === "wallet_deposit_balance_update") {
      assert.equal(update.data.balance.mint, "MintA");
    }

    const native = parseMessageIn(
      JSON.stringify({
        type: "wallet_deposit_balances",
        version: 0.1,
        data: {
          event_type: "wallet_native_sol_balance_update",
          wallet_address: wallet,
          context_slot: 125,
          native_sol_balance: "2.000000001",
        },
      })
    );
    assert.equal(native.type, "wallet_deposit_balances");
    if (
      native.type === "wallet_deposit_balances" &&
      native.data.event_type === "wallet_native_sol_balance_update"
    ) {
      assert.equal(native.data.native_sol_balance, "2.000000001");
    }

    const status = parseMessageIn(
      JSON.stringify({
        type: "wallet_deposit_balances",
        version: 0.1,
        data: {
          event_type: "wallet_deposit_balance_status",
          wallet_address: wallet,
          status: "metadata_unavailable",
          code: "METADATA_UNAVAILABLE",
        },
      })
    );
    assert.equal(status.type, "wallet_deposit_balances");
    if (
      status.type === "wallet_deposit_balances" &&
      status.data.event_type === "wallet_deposit_balance_status"
    ) {
      assert.equal(status.data.code, "METADATA_UNAVAILABLE");
    }
  });

  it("rejects unknown events and non-string native SOL", () => {
    for (const data of [
      {
        event_type: "unknown",
        wallet_address: wallet,
      },
      {
        event_type: "wallet_native_sol_balance_update",
        wallet_address: wallet,
        context_slot: 1,
        native_sol_balance: 1,
      },
      {
        event_type: "wallet_native_sol_balance_update",
        wallet_address: wallet,
        context_slot: 1,
        native_sol_balance: "1.0",
      },
    ]) {
      assert.throws(() =>
        parseMessageIn(
          JSON.stringify({
            type: "wallet_deposit_balances",
            version: 0.1,
            data,
          })
        )
      );
    }
  });
});

describe("WalletDepositBalancesState", () => {
  it("initializes from REST and applies complete snapshots despite lower slots", () => {
    const state = new WalletDepositBalancesState();
    state.applyRestSnapshot(wallet, {
      context_slot: 200,
      balances: {},
      native_sol_balance: "1.000000000",
    });
    state.applyEvent({
      event_type: "wallet_deposit_balance_snapshot",
      wallet_address: wallet,
      context_slot: 100,
      balances: {
        [WRAPPED_SOL_MINT]: balance(WRAPPED_SOL_MINT, "0.500000000"),
      } as Record<PubkeyStr, DepositTokenBalance>,
      native_sol_balance: "1.500000000",
    });

    assert.equal(state.contextSlot, 100);
    assert.equal(state.combinedSolBalance(), "2.000000000");
    assert.equal(state.nativeSolBalance, "1.500000000");
    assert.equal(state.balances.get(WRAPPED_SOL_MINT)?.idle, "0.500000000");
  });

  it("optionally ignores complete snapshots below a minimum slot", () => {
    const state = new WalletDepositBalancesState();
    state.applyRestSnapshot(wallet, {
      context_slot: 200,
      balances: {},
      native_sol_balance: "1.000000000",
    });

    assert.deepEqual(
      state.applyRestSnapshot(
        wallet,
        {
          context_slot: 99,
          balances: {},
          native_sol_balance: "2.000000000",
        },
        100
      ),
      { kind: "ignored" }
    );
    assert.equal(state.contextSlot, 200);
    assert.deepEqual(
      state.applyEvent(
        {
          event_type: "wallet_deposit_balance_snapshot",
          wallet_address: wallet,
          context_slot: 99,
          balances: {},
          native_sol_balance: "2.000000000",
        },
        100
      ),
      { kind: "ignored" }
    );
    assert.equal(state.contextSlot, 200);

    assert.deepEqual(
      state.applyRestSnapshot(
        wallet,
        {
          context_slot: 100,
          balances: {},
          native_sol_balance: "3.000000000",
        },
        100
      ),
      { kind: "applied" }
    );
    assert.equal(state.contextSlot, 100);
    assert.equal(state.nativeSolBalance, "3.000000000");

    assert.deepEqual(
      state.applyEvent(
        {
          event_type: "wallet_deposit_balance_snapshot",
          wallet_address: wallet,
          context_slot: 100,
          balances: {},
          native_sol_balance: "4.000000000",
        },
        100
      ),
      { kind: "applied" }
    );
    assert.equal(state.nativeSolBalance, "4.000000000");
  });

  it("does not apply the snapshot floor to components or no-floor calls", () => {
    const state = new WalletDepositBalancesState();
    state.applyRestSnapshot(wallet, {
      context_slot: 100,
      balances: {},
      native_sol_balance: "1.000000000",
    });

    assert.deepEqual(
      state.applyEvent(
        {
          event_type: "wallet_native_sol_balance_update",
          wallet_address: wallet,
          context_slot: 50,
          native_sol_balance: "2.000000000",
        },
        100
      ),
      { kind: "applied" }
    );
    assert.deepEqual(
      state.applyEvent({
        event_type: "wallet_deposit_balance_snapshot",
        wallet_address: wallet,
        context_slot: 25,
        balances: {},
        native_sol_balance: "3.000000000",
      }),
      { kind: "applied" }
    );
    assert.equal(state.contextSlot, 25);
    assert.equal(state.nativeSolBalance, "3.000000000");
  });

  it("uses absolute component updates and ignores mismatched/status events", () => {
    const state = new WalletDepositBalancesState();
    state.applyRestSnapshot(wallet, {
      context_slot: 1,
      balances: {},
      native_sol_balance: "1.000000000",
    });
    assert.deepEqual(
      state.applyEvent({
        event_type: "wallet_native_sol_balance_update",
        wallet_address: "WalletB" as PubkeyStr,
        context_slot: 2,
        native_sol_balance: "9.000000000",
      }),
      { kind: "ignored" }
    );
    assert.equal(state.nativeSolBalance, "1.000000000");

    state.applyEvent({
      event_type: "wallet_deposit_balance_update",
      wallet_address: wallet,
      context_slot: 3,
      balance: balance("MintA" as PubkeyStr, "0.000000000001"),
    });
    assert.equal(state.balances.has("MintA" as PubkeyStr), true);
    state.applyEvent({
      event_type: "wallet_deposit_balance_update",
      wallet_address: wallet,
      context_slot: 4,
      balance: balance("MintA" as PubkeyStr, "0.000000000000"),
    });
    assert.equal(state.balances.has("MintA" as PubkeyStr), false);

    const beforeInvalid = {
      contextSlot: state.contextSlot,
      balances: new Map(state.balances),
    };
    assert.deepEqual(
      state.applyEvent({
        event_type: "wallet_deposit_balance_update",
        wallet_address: wallet,
        context_slot: 5,
        balance: balance("MintA" as PubkeyStr, "-1"),
      }),
      { kind: "rejected" }
    );
    assert.equal(state.contextSlot, beforeInvalid.contextSlot);
    assert.deepEqual(state.balances, beforeInvalid.balances);

    const before = state.nativeSolBalance;
    assert.deepEqual(
      state.applyEvent({
        event_type: "wallet_deposit_balance_status",
        wallet_address: wallet,
        status: "reconnecting",
        code: "SOLANA_WALLET_BALANCE_STREAM_RECONNECTING",
      }),
      { kind: "ignored" }
    );
    assert.equal(state.nativeSolBalance, before);
  });

  it("sums arbitrarily large exact balances without floating point", () => {
    const state = new WalletDepositBalancesState();
    state.applyRestSnapshot(wallet, {
      context_slot: 1,
      balances: {},
      native_sol_balance: "18446744073.709551616",
    });
    assert.equal(state.combinedSolBalance(), "18446744073.709551616");
    assert.throws(() => state.solComponents(), /transaction u64 range/);
  });
});

describe("wallet deposit balance subscriptions", () => {
  it("uses the unchanged wallet-address request shape and stable identity", () => {
    const subscribe = subscribeWalletDepositBalances(wallet);
    const unsubscribe = unsubscribeWalletDepositBalances(wallet);
    assert.deepEqual(subscribe, {
      method: "subscribe",
      params: { type: "wallet_deposit_balances", wallet_address: wallet },
    });
    assert.equal(
      subscriptionKey(subscribe.params),
      "wallet_deposit_balances:WalletA"
    );
    if (unsubscribe.method !== "unsubscribe") return;
    assert.equal(unsubscribeMatches(subscribe.params, unsubscribe.params), true);
  });
});
