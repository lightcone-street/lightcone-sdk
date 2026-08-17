import assert from "node:assert/strict";
import { describe, it } from "node:test";
import {
  ASSOCIATED_TOKEN_PROGRAM_ID,
  decodeCloseAccountInstruction,
  decodeSyncNativeInstruction,
  getAssociatedTokenAddressSync,
  NATIVE_MINT,
  TOKEN_PROGRAM_ID,
} from "@solana/spl-token";
import {
  Keypair,
  PublicKey,
  SystemInstruction,
  Transaction,
  type Connection,
} from "@solana/web3.js";
import type { DepositTokenBalance } from "../src/domain/position";
import type { AuthCredentials } from "../src/auth";
import { Positions } from "../src/domain/position/client";
import {
  WRAPPED_SOL_MINT,
  WalletDepositBalancesState,
} from "../src/domain/position/state";
import type { PubkeyStr } from "../src/shared";
import type { ClientContext } from "../src/context";
import { DepositSource } from "../src/shared";
import { RpcFailoverState } from "../src/rpcFailover";
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

describe("self-custody SOL conversion", () => {
  function conversionHarness(): {
    positions: Positions;
    state: WalletDepositBalancesState;
    sent: Transaction[];
    context: ClientContext;
  } {
    const keypair = Keypair.generate();
    const walletAddress = keypair.publicKey.toBase58() as PubkeyStr;
    const sent: Transaction[] = [];
    const connection = {
      getLatestBlockhash: async () => ({
        blockhash: "11111111111111111111111111111111",
        lastValidBlockHeight: 100,
      }),
      sendRawTransaction: async (wire: Buffer | Uint8Array) => {
        sent.push(Transaction.from(wire));
        return "confirmed-signature";
      },
      getSignatureStatuses: async () => ({
        value: [
          {
            slot: 42,
            confirmations: 1,
            err: null,
            confirmationStatus: "confirmed",
            status: { Ok: null },
          },
        ],
      }),
    } as unknown as Connection;
    const context = {
      http: { baseUrl: () => "https://api.example.test" },
      programId: PublicKey.default,
      primaryConnection: connection,
      rpcFailoverState: new RpcFailoverState(),
      depositSource: DepositSource.Global,
      signingStrategy: { type: "native", keypair },
      authCredentials: {
        user_id: "user",
        wallet_address: walletAddress,
        expires_at: new Date(Date.now() + 60_000),
      },
    } as unknown as ClientContext;
    const state = new WalletDepositBalancesState();
    state.applyRestSnapshot(walletAddress, {
      context_slot: 1,
      native_sol_balance: "2.000000000",
      balances: {
        [WRAPPED_SOL_MINT]: balance(WRAPPED_SOL_MINT, "1.000000000"),
      } as Record<PubkeyStr, DepositTokenBalance>,
    });
    return { positions: new Positions(context), state, sent, context };
  }

  it("builds maintained create-transfer-sync instructions and confirms", async () => {
    const { positions, state, sent } = conversionHarness();
    const before = state.combinedSolBalance();
    assert.equal(await positions.wrapSol("0.123456789", state), "confirmed-signature");

    const transaction = sent[0];
    assert.ok(transaction);
    assert.equal(transaction.instructions.length, 3);
    assert.equal(
      transaction.instructions[0]?.programId.equals(ASSOCIATED_TOKEN_PROGRAM_ID),
      true
    );
    assert.deepEqual([...transaction.instructions[0]!.data], [1]);
    const transfer = SystemInstruction.decodeTransfer(transaction.instructions[1]!);
    assert.equal(transfer.lamports, 123_456_789n);
    const sync = decodeSyncNativeInstruction(transaction.instructions[2]!);
    const canonical = getAssociatedTokenAddressSync(NATIVE_MINT, transaction.feePayer!);
    assert.equal(sync.keys.account.pubkey.equals(canonical), true);
    assert.equal(state.combinedSolBalance(), before);
  });

  it("builds one full canonical close instruction and confirms", async () => {
    const { positions, state, sent } = conversionHarness();
    assert.equal(await positions.unwrapWsol(state), "confirmed-signature");

    const transaction = sent[0];
    assert.ok(transaction);
    assert.equal(transaction.instructions.length, 1);
    const close = decodeCloseAccountInstruction(
      transaction.instructions[0]!,
      TOKEN_PROGRAM_ID
    );
    const canonical = getAssociatedTokenAddressSync(NATIVE_MINT, transaction.feePayer!);
    assert.equal(close.keys.account.pubkey.equals(canonical), true);
    assert.equal(close.keys.destination.pubkey.equals(transaction.feePayer!), true);
    assert.equal(close.keys.authority.pubkey.equals(transaction.feePayer!), true);
  });

  it("requires positive cached WSOL before signing an unwrap", async () => {
    for (const idle of [undefined, "0.000000000"]) {
      const { positions, state, sent } = conversionHarness();
      if (idle === undefined) {
        state.balances.delete(WRAPPED_SOL_MINT);
      } else {
        state.balances.set(WRAPPED_SOL_MINT, balance(WRAPPED_SOL_MINT, idle));
      }

      await assert.rejects(
        () => positions.unwrapWsol(state),
        /must be greater than zero/
      );
      assert.equal(sent.length, 0);
    }
  });

  it("rejects precision, bounds, and cached-balance failures before signing", async () => {
    const { positions, state, sent } = conversionHarness();
    await assert.rejects(() => positions.wrapSol("-1", state));
    await assert.rejects(() => positions.wrapSol("0.0000000001", state));
    await assert.rejects(() => positions.wrapSol("18446744073.709551616", state));
    await assert.rejects(() => positions.wrapSol("3", state));
    assert.equal(sent.length, 0);
  });

  it("requires unexpired matching credentials before signing", async () => {
    const { positions, state, sent, context } = conversionHarness();
    const credentials = (context as { authCredentials: AuthCredentials })
      .authCredentials;

    (context as { authCredentials: AuthCredentials | undefined }).authCredentials =
      undefined;
    await assert.rejects(() => positions.wrapSol("0.1", state));

    (context as { authCredentials: AuthCredentials }).authCredentials = {
      ...credentials,
      expires_at: new Date(Date.now() - 1),
    };
    await assert.rejects(() => positions.wrapSol("0.1", state));

    (context as { authCredentials: AuthCredentials }).authCredentials = {
      ...credentials,
      wallet_address: Keypair.generate().publicKey.toBase58() as PubkeyStr,
    };
    await assert.rejects(() => positions.wrapSol("0.1", state));
    assert.equal(sent.length, 0);
  });

  it("propagates submission failures without mutating state", async () => {
    const { positions, state, context } = conversionHarness();
    const before = state.combinedSolBalance();
    (
      context.primaryConnection as unknown as {
        sendRawTransaction: () => Promise<string>;
      }
    ).sendRawTransaction = async () => {
      throw new Error("submission failed");
    };

    await assert.rejects(
      () => positions.wrapSol("0.1", state),
      /submission failed/
    );
    assert.equal(state.combinedSolBalance(), before);
  });
});
