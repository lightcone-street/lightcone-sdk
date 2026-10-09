import { describe, it } from "node:test";
import assert from "node:assert/strict";
import Decimal from "decimal.js";

import {
  ApplyOutcome,
  closureScope,
  ClosureScope,
  closureUpdateFromWire,
  convertSnapshotOrders,
  limitOrderFromUpdate,
  OrderStatus,
  orderUpdatePrice,
  UserOpenLimitOrders,
  userUpdateFromWire,
  type ClosureUpdate,
  type OrderUpdate,
  type UserSnapshotOrder,
  type UserUpdate,
} from "../src/domain/order";
import { SdkError } from "../src/error";
import { decimalTextIsNegative, decimalTextToDecimal } from "../src/shared/decimal_text";
import { asOrderBookId, asPubkeyStr, FundingSource, Side, TimeInForce } from "../src/shared";
import { parseMessageIn } from "../src/ws";

const WALLET = "Wallet1111111111111111111111111111111111111";
const HASH = "4f1a4b1ab1c0c0ffee0000000000000000000000000000000000000000000001";
const MARKET = "A9Bxkkc4nah517EjgjwSafwspGnmU9s1Ei5PTo5ZkJd9";
const BOOK = "j749bQAbDsBAiyDs2Tj868heQj1b5KVp98ZrjiZd56a";

/** Decode a `user` frame through the WebSocket entry point. */
function user(data: Record<string, unknown>): UserUpdate {
  const message = parseMessageIn(JSON.stringify({ type: "user", version: 0.1, data }));
  assert.equal(message.type, "user");
  if (message.type !== "user") throw new Error("expected user frame");
  return message.data;
}

/** Live order fact as `redis_listener::handle_committed` relays it: envelope numbers, numeric side, scaled quantities. */
function liveOrderFact(
  revision: number,
  openBase: string,
  pendingBase: string,
  closedReason: string | null = null,
): Record<string, unknown> {
  return {
    effect_id: `${revision}:3`,
    committed_revision: revision,
    projection_generation: 1,
    actionable: true,
    event_type: "order",
    user_pubkey: WALLET,
    order_hash: HASH,
    market_pubkey: MARKET,
    orderbook_id: BOOK,
    base_mint: "BaseMint111111111111111111111111111111111111",
    quote_mint: "QuoteMint11111111111111111111111111111111111",
    side: 0,
    maker_amount: "5.500000",
    taker_amount: "10.00000000",
    original_base: "10.00000000",
    confirmed_base: "2.00000000",
    pending_base: pendingBase,
    open_base: openBase,
    executable_base: openBase,
    cancelled_base: "0.00000000",
    accepted_seq: 44,
    accepted_at: "2026-09-29T12:00:00+00:00",
    expiration: 0,
    tif: "GTC",
    funding_source: "global",
    funding_account: "Acct1111111111111111111111111111111111111111",
    closed_reason: closedReason,
    ready: true,
  };
}

function liveOrder(
  revision: number,
  openBase: string,
  pendingBase: string,
  closedReason: string | null = null,
): OrderUpdate {
  const update = user(liveOrderFact(revision, openBase, pendingBase, closedReason));
  if (update.event_type !== "order") throw new Error(`expected order, got ${update.event_type}`);
  return update;
}

describe("user channel decoding", () => {
  it("decodes a live order fact with a numeric side and envelope numbers", () => {
    const order = liveOrder(812, "7.00000000", "1.00000000");
    assert.equal(order.commit.committed_revision, 812n);
    assert.equal(order.commit.projection_generation, 1n);
    assert.ok(order.commit.actionable);
    assert.equal(order.side, Side.Bid);
    assert.equal(order.accepted_seq, 44n);
    assert.equal(order.open_base, "7.00000000");
    assert.equal(order.funding_source, FundingSource.Global);
    assert.equal(order.tif, TimeInForce.Gtc);
    assert.equal(orderUpdatePrice(order), "0.55");
    assert.ok(order.ready);
  });

  it("decodes a snapshot with string revisions and empty markers", () => {
    const update = user({
      event_type: "snapshot",
      orders: [
        {
          order_hash: HASH,
          market_pubkey: MARKET,
          orderbook_id: BOOK,
          side: "ask",
          maker_amount: "4.00000000",
          taker_amount: "2.200000",
          original_base: "4.00000000",
          confirmed_base: "0.00000000",
          pending_base: "0.00000000",
          open_base: "4.00000000",
          cancelled_base: "0.00000000",
          closed_reason: "",
          committed_revision: "812",
          accepted_seq: "45",
          price: "0.550000",
          created_at: 1790685521784,
          expiration: 0,
          base_mint: "BaseMint111111111111111111111111111111111111",
          quote_mint: "QuoteMint11111111111111111111111111111111111",
          tif: "IOC",
          funding_source: "conditional",
        },
      ],
      funding_accounts: [],
      next_cursor: "",
      has_more: false,
      next_funding_cursor: "",
      funding_has_more: false,
      committed_revision: "812",
      projection_generation: "1",
      notifications: [],
    });
    assert.equal(update.event_type, "snapshot");
    if (update.event_type !== "snapshot") return;
    assert.equal(update.committed_revision, 812n);
    assert.equal(update.next_cursor, undefined);
    assert.equal(update.next_funding_cursor, undefined);
    const order = update.orders[0]!;
    assert.equal(order.side, Side.Ask);
    assert.equal(order.closed_reason, undefined);
    assert.equal(order.accepted_seq, 45n);
    assert.equal(order.tif, TimeInForce.Ioc);
    assert.equal(order.funding_source, FundingSource.Conditional);
  });

  it("keeps a funding fact's wide aggregates exact", () => {
    const update = user({
      effect_id: "7:2",
      committed_revision: 7,
      projection_generation: 1,
      actionable: false,
      event_type: "funding",
      user_pubkey: WALLET,
      account: "Acct1111111111111111111111111111111111111111",
      mint: "Cond1111111111111111111111111111111111111111",
      source: "conditional",
      market_pubkey: MARKET,
      raw_observed: "18446744073709.551615",
      usable_observed: "1.000000",
      order_reserved: "340282366920938463463374607431768.211455",
      execution_reserved: "0.000001",
      signed_remaining: "-0.000001",
      available: "0.000000",
      ready: false,
      accepted_boundary: "boundary",
      slot: "7",
      blockhash: null,
    });
    assert.equal(update.event_type, "funding");
    if (update.event_type !== "funding") return;
    assert.ok(!update.commit.actionable);
    assert.ok(!update.ready);
    assert.equal(update.available, "0.000000");
    assert.equal(update.raw_observed, "18446744073709.551615");
    assert.equal(update.order_reserved, "340282366920938463463374607431768.211455");
    assert.ok(update.signed_remaining && decimalTextIsNegative(update.signed_remaining));
    assert.ok(decimalTextToDecimal(update.signed_remaining!).eq("-0.000001"));
    assert.equal(update.slot, 7n);
    assert.equal(update.blockhash, undefined);
  });

  it("decodes a global funding fact without a market", () => {
    const update = user({
      effect_id: "8:0",
      committed_revision: 8,
      projection_generation: 1,
      actionable: true,
      event_type: "funding",
      user_pubkey: WALLET,
      account: "Glob1111111111111111111111111111111111111111",
      mint: "7SrxsoXjNR7Y8T3koJCt1yV4FrNUumoAUrJExDt6tQez",
      source: "global",
      market_pubkey: null,
      raw_observed: null,
      usable_observed: null,
      order_reserved: "0.000000",
      execution_reserved: "0.000000",
      signed_remaining: null,
      available: "0.000000",
      ready: false,
      accepted_boundary: null,
      slot: null,
      blockhash: null,
    });
    assert.equal(update.event_type, "funding");
    if (update.event_type !== "funding") return;
    assert.equal(update.source, FundingSource.Global);
    assert.equal(update.market_pubkey, undefined);
    assert.equal(update.raw_observed, undefined);
    assert.equal(update.slot, undefined);
  });

  it("decodes closure and recovery facts", () => {
    const closure = user({
      effect_id: "9:0",
      committed_revision: 9,
      projection_generation: 1,
      actionable: true,
      event_type: "closure",
      scope_kind: 5,
      scope_key: `${WALLET}:${BOOK}`,
      accepted_seq: 44,
      reason: "cancel_all",
    });
    assert.equal(closure.event_type, "closure");
    if (closure.event_type === "closure") {
      assert.equal(closureScope(closure), ClosureScope.WalletBook);
      assert.equal(closure.accepted_seq, 44n);
    }

    const recovery = user({
      effect_id: "10:0",
      committed_revision: 10,
      projection_generation: 2,
      actionable: true,
      event_type: "recovery_completed",
      recovered_generation: 2,
    });
    assert.equal(recovery.event_type, "recovery_completed");
    if (recovery.event_type === "recovery_completed") {
      assert.equal(recovery.recovered_generation, 2n);
      assert.equal(recovery.commit.projection_generation, 2n);
    }
  });

  it("delivers a user fill with the trades payload", () => {
    const update = userUpdateFromWire({
      effect_id: "812:1",
      committed_revision: 812,
      projection_generation: 3,
      actionable: true,
      event_type: "fill",
      fill_id: "f",
      execution_id: "e",
      evidence_id: "v",
      event_index: 2,
      leg_index: 0,
      orderbook_id: BOOK,
      market_pubkey: MARKET,
      base_mint: "b",
      quote_mint: "q",
      base_amount: "4",
      quote_amount: "2.2",
      maker_order_hash: "aa",
      taker_order_hash: "bb",
      maker_wallet: "m",
      taker_wallet: "t",
      taker_side: 1,
      maker_fee_estimate: "0",
      taker_fee_estimate: "0",
      fee_mint: "q",
      maker_fee_bps: 0,
      taker_fee_bps: 0,
      executed_at: "2026-09-29T12:00:00Z",
    });
    assert.equal(update.event_type, "fill");
    if (update.event_type === "fill") assert.equal(update.taker_side, Side.Ask);
  });

  it("decodes removed and future user events as unknown", () => {
    for (const eventType of ["nonce", "market_balance_update", "global_deposit_update", "future_event"]) {
      const update = user({ event_type: eventType, new_nonce: 3, timestamp: "2026-06-19T12:34:56Z" });
      assert.deepEqual(update, { event_type: "unknown", raw_event_type: eventType });
    }
  });

  it("surfaces a malformed fact as a field-named decode error", () => {
    assert.throws(
      () => userUpdateFromWire({ ...liveOrderFact(1, "1", "0"), accepted_seq: "-1" }),
      (error) => error instanceof SdkError && error.variant === "Serde" && /accepted_seq/.test(error.message),
    );
  });
});

describe("limit order conversion", () => {
  function snapshotOrder(hash: string, open: number, pending: number): UserSnapshotOrder {
    return {
      order_hash: hash,
      market_pubkey: asPubkeyStr("mkt1"),
      orderbook_id: asOrderBookId("ob1"),
      side: Side.Ask,
      maker_amount: "10",
      taker_amount: "5",
      original_base: "10",
      confirmed_base: String(10 - open - pending),
      pending_base: String(pending),
      open_base: String(open),
      cancelled_base: "0",
      closed_reason: undefined,
      committed_revision: 3n,
      accepted_seq: 1n,
      price: "0.5",
      created_at: new Date(1_790_685_521_784),
      expiration: 0n,
      base_mint: asPubkeyStr("b"),
      quote_mint: asPubkeyStr("q"),
      tif: TimeInForce.Gtc,
      funding_source: FundingSource.Conditional,
    };
  }

  it("converts a live order fact's quantities and price", () => {
    const order = limitOrderFromUpdate(liveOrder(812, "7.00000000", "1.00000000"));
    assert.equal(order.orderHash, HASH);
    assert.equal(order.side, Side.Bid);
    assert.equal(order.price, "0.55");
    assert.equal(order.size, "10.00000000");
    assert.equal(order.filledSize, "2.00000000");
    assert.equal(order.pendingSize, "1.00000000");
    assert.equal(order.remainingSize, "7.00000000");
    assert.equal(order.status, OrderStatus.Pending);
    assert.equal(order.committedRevision, 812n);
    assert.equal(order.acceptedSeq, 44n);
    assert.equal(order.fundingSource, FundingSource.Global);
  });

  it("lets a closed reason win status precedence", () => {
    const order = limitOrderFromUpdate(liveOrder(813, "0.00000000", "1.00000000", "expired"));
    assert.equal(order.status, OrderStatus.Closed);
    assert.equal(order.closedReason, "expired");
  });

  it("rejects a live order fact without a limit price", () => {
    const update = { ...liveOrder(814, "7.00000000", "0.00000000"), taker_amount: "0" };
    const isValidation = (error: unknown) => error instanceof SdkError && error.variant === "Validation";
    assert.throws(() => limitOrderFromUpdate(update), isValidation);
    const openOrders = new UserOpenLimitOrders();
    assert.throws(() => openOrders.apply(update), isValidation);
    assert.ok(openOrders.isEmpty());
  });

  it("keeps only live orders from a snapshot", () => {
    const open = convertSnapshotOrders([
      snapshotOrder("resting", 4, 0),
      snapshotOrder("claiming", 0, 2),
      snapshotOrder("done", 0, 0),
    ]);
    const orders = open.get(asPubkeyStr("mkt1"), asOrderBookId("ob1"))!;
    assert.deepEqual(orders.map((order) => order.orderHash), ["resting", "claiming"]);
    assert.equal(orders[0]!.status, OrderStatus.Open);
    assert.equal(orders[1]!.status, OrderStatus.Pending);
  });
});

describe("UserOpenLimitOrders", () => {
  function tracked(container: UserOpenLimitOrders) {
    return container.get(asPubkeyStr(MARKET), asOrderBookId(BOOK)) ?? [];
  }

  function closure(scopeKind: number, scopeKey: string, acceptedSeq: number): ClosureUpdate {
    return closureUpdateFromWire({
      effect_id: "900:0",
      committed_revision: 900,
      projection_generation: 1,
      actionable: true,
      scope_kind: scopeKind,
      scope_key: scopeKey,
      accepted_seq: acceptedSeq,
      reason: "cancel_all",
    });
  }

  it("inserts, updates, and removes by liveness", () => {
    const container = new UserOpenLimitOrders();
    assert.equal(container.apply(liveOrder(10, "8.00000000", "0.00000000")), ApplyOutcome.Inserted);
    assert.equal(container.apply(liveOrder(11, "5.00000000", "3.00000000")), ApplyOutcome.Updated);
    const orders = tracked(container);
    assert.equal(orders.length, 1);
    assert.equal(orders[0]!.remainingSize, "5.00000000");
    assert.equal(orders[0]!.pendingSize, "3.00000000");
    assert.equal(orders[0]!.side, Side.Bid);

    // Pending fills keep a fully matched order live until confirmation.
    assert.equal(container.apply(liveOrder(12, "0.00000000", "3.00000000")), ApplyOutcome.Updated);
    assert.equal(container.apply(liveOrder(13, "0.00000000", "0.00000000")), ApplyOutcome.Removed);
    assert.ok(container.isEmpty());
    assert.equal(container.apply(liveOrder(14, "0.00000000", "0.00000000")), ApplyOutcome.Ignored);
  });

  it("ignores older revisions", () => {
    const container = new UserOpenLimitOrders();
    container.apply(liveOrder(20, "5.00000000", "0.00000000"));
    assert.equal(container.apply(liveOrder(19, "8.00000000", "0.00000000")), ApplyOutcome.Stale);
    assert.equal(tracked(container)[0]!.remainingSize, "5.00000000");
  });

  it("closes orders in scope up to the cutoff", () => {
    const container = new UserOpenLimitOrders();
    container.apply(liveOrder(10, "8.00000000", "0.00000000"));
    // Cutoff below the order's accepted_seq (44) leaves it resting.
    assert.equal(container.applyClosure(closure(2, BOOK, 43)), 0);
    assert.equal(tracked(container).length, 1);
    // Other books are out of scope.
    assert.equal(container.applyClosure(closure(2, "OtherBook", 44)), 0);
    assert.equal(container.applyClosure(closure(5, `wallet:${BOOK}`, 44)), 1);
    assert.ok(container.isEmpty());
  });

  it("keeps pending claims visible after a closure", () => {
    const container = new UserOpenLimitOrders();
    container.apply(liveOrder(10, "5.00000000", "3.00000000"));
    assert.equal(container.applyClosure(closure(4, "wallet", 50)), 1);
    const order = tracked(container)[0]!;
    assert.equal(order.remainingSize, "0");
    assert.equal(order.cancelledSize, "5.00000000");
    assert.equal(order.status, OrderStatus.Closed);
    assert.equal(order.closedReason, "cancel_all");
    assert.equal(order.committedRevision, 900n);
    // A fact older than the closure cannot reopen the order.
    assert.equal(container.apply(liveOrder(11, "5.00000000", "3.00000000")), ApplyOutcome.Stale);
  });

  it("closes exactly, independent of the global decimal precision", () => {
    const previous = Decimal.precision;
    Decimal.set({ precision: 6 });
    try {
      const container = new UserOpenLimitOrders();
      container.apply(liveOrder(10, "12344.678912345", "1.00000000"));
      assert.equal(container.applyClosure(closure(4, "wallet", 50)), 1);
      assert.equal(tracked(container)[0]!.cancelledSize, "12344.678912345");
    } finally {
      Decimal.set({ precision: previous });
    }
  });

  it("replaces closed orders instead of changing held references", () => {
    const container = new UserOpenLimitOrders();
    container.apply(liveOrder(10, "5.00000000", "3.00000000"));
    const heldArray = tracked(container);
    const heldOrder = heldArray[0]!;
    assert.equal(container.applyClosure(closure(4, "wallet", 50)), 1);
    assert.equal(heldOrder.remainingSize, "5.00000000");
    assert.equal(heldOrder.status, OrderStatus.Pending);
    assert.equal(heldArray[0], heldOrder);
    assert.notEqual(tracked(container)[0], heldOrder);
    assert.equal(tracked(container)[0]!.status, OrderStatus.Closed);
  });

  it("does not let older state reopen an order a closure removed", () => {
    const container = new UserOpenLimitOrders();
    container.apply(liveOrder(10, "8.00000000", "0.00000000"));
    assert.equal(container.applyClosure(closure(4, "wallet", 50)), 1);
    assert.ok(container.isEmpty());
    // A REST page or snapshot captured before the closure arrives late.
    assert.equal(container.apply(liveOrder(11, "8.00000000", "0.00000000")), ApplyOutcome.Stale);
    assert.ok(container.isEmpty());
  });

  it("does not let older state reopen a removed order", () => {
    const container = new UserOpenLimitOrders();
    container.apply(liveOrder(10, "8.00000000", "0.00000000"));
    assert.equal(
      container.apply(liveOrder(12, "0.00000000", "0.00000000", "cancelled")),
      ApplyOutcome.Removed,
    );
    assert.equal(container.apply(liveOrder(11, "8.00000000", "0.00000000")), ApplyOutcome.Stale);
    assert.equal(container.apply(liveOrder(12, "8.00000000", "0.00000000")), ApplyOutcome.Stale);
    assert.ok(container.isEmpty());
    // Newer committed state stays authoritative.
    assert.equal(container.apply(liveOrder(13, "8.00000000", "0.00000000")), ApplyOutcome.Inserted);
  });

  it("applies a closure to older state seeded after it", () => {
    const container = new UserOpenLimitOrders();
    assert.equal(container.applyClosure(closure(4, "wallet", 50)), 0);
    assert.equal(container.apply(liveOrder(11, "8.00000000", "0.00000000")), ApplyOutcome.Ignored);
    assert.ok(container.isEmpty());

    // Pending fills keep the closed order visible, as with a tracked order.
    container.clear();
    container.applyClosure(closure(4, "wallet", 50));
    assert.equal(container.apply(liveOrder(11, "5.00000000", "3.00000000")), ApplyOutcome.Inserted);
    const order = tracked(container)[0]!;
    assert.equal(order.remainingSize, "0");
    assert.equal(order.cancelledSize, "5.00000000");
    assert.equal(order.status, OrderStatus.Closed);
    assert.equal(order.committedRevision, 900n);

    // Orders accepted after the cutoff, and newer state, are unaffected.
    container.clear();
    container.applyClosure(closure(4, "wallet", 43));
    assert.equal(container.apply(liveOrder(11, "8.00000000", "0.00000000")), ApplyOutcome.Inserted);
    container.clear();
    container.applyClosure(closure(4, "wallet", 50));
    assert.equal(container.apply(liveOrder(901, "8.00000000", "0.00000000")), ApplyOutcome.Inserted);
  });

  it("forgets retired orders and closures on clear", () => {
    const container = new UserOpenLimitOrders();
    container.apply(liveOrder(10, "8.00000000", "0.00000000"));
    container.applyClosure(closure(4, "wallet", 50));
    container.clear();
    assert.equal(container.apply(liveOrder(11, "8.00000000", "0.00000000")), ApplyOutcome.Inserted);
  });

  it("requests a refresh for closure scopes without order data", () => {
    const container = new UserOpenLimitOrders();
    container.apply(liveOrder(10, "5.00000000", "0.00000000"));
    assert.equal(container.applyClosure(closure(3, "mint", 50)), undefined);
    assert.equal(container.applyClosure(closure(99, "x", 50)), undefined);
    assert.equal(tracked(container).length, 1);
  });

  it("removes and clears", () => {
    const container = new UserOpenLimitOrders();
    container.apply(liveOrder(10, "5.00000000", "0.00000000"));
    assert.ok(container.getByHash(HASH));
    container.remove(HASH);
    assert.ok(container.isEmpty());
    container.apply(liveOrder(11, "5.00000000", "0.00000000"));
    container.clear();
    assert.ok(container.isEmpty());
  });

  it("never mutates the caller's order", () => {
    const container = new UserOpenLimitOrders();
    container.applyClosure(closure(4, "wallet", 50));
    const order = limitOrderFromUpdate(liveOrder(11, "5.00000000", "3.00000000"));
    container.applyOrder(order);
    assert.equal(order.remainingSize, "5.00000000");
    assert.equal(order.status, OrderStatus.Pending);
  });
});
