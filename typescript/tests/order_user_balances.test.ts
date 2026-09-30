import { describe, it } from "node:test";
import assert from "node:assert/strict";
import {
  normalizeConditionalBalance,
  normalizeUserOrdersPayload,
  normalizeUserUpdate,
} from "../src/domain/order/wire";
import { parseMessageIn } from "../src/ws";
import { convertSnapshotOrders, UserOpenLimitOrders } from "../src/domain/order";

const marketBalance = {
  market_pubkey: "market-1",
  deposit_assets: [
    {
      deposit_asset: "usdc-mint",
      outcomes: [
        {
          outcome_index: 0,
          conditional_token: "trump-usdc-mint",
          balance: "125.000000",
          balance_idle: "40.000000",
          balance_on_book: "85.000000",
        },
        {
          outcome_index: 1,
          conditional_token: "kamala-usdc-mint",
          balance: "100.000000",
          balance_idle: "100.000000",
          balance_on_book: "0.000000",
        },
        {
          outcome_index: 2,
          conditional_token: "biden-usdc-mint",
          balance: "100.000000",
          balance_idle: "100.000000",
          balance_on_book: "0.000000",
        },
      ],
    },
  ],
};

const userOrder = (orderbookId: string, baseMint: string) => ({
  order_type: "limit" as const,
  order_hash: `order-${orderbookId}`,
  market_pubkey: "market-1",
  orderbook_id: orderbookId,
  side: "bid",
  amount_in: "10.000000",
  amount_out: "10.000000",
  remaining: "10.000000",
  filled: "0.000000",
  price: "1.000000",
  created_at: 0,
  expiration: 0,
  base_mint: baseMint,
  quote_mint: "trump-usdc-mint",
  outcome_index: 0,
  status: "OPEN",
});

describe("user market balances", () => {
  it("normalizes snapshots with multiple outcomes under one deposit asset", () => {
    const update = normalizeUserUpdate({
      event_type: "snapshot",
      orders: [
        userOrder("trump-btc-usdc", "trump-btc-mint"),
        userOrder("trump-eth-usdc", "trump-eth-mint"),
      ],
      market_balances: [marketBalance],
      global_deposits: [],
      notifications: [],
      nonce: 7,
    });

    assert.equal(update.event_type, "snapshot");
    assert.equal(update.market_balances.length, 1);
    assert.equal(update.market_balances[0].deposit_assets[0].deposit_asset, "usdc-mint");
    assert.equal(update.market_balances[0].deposit_assets[0].outcomes.length, 3);
    assert.deepEqual(
      update.orders.map((order) => order.orderbook_id),
      ["trump-btc-usdc", "trump-eth-usdc"]
    );
    assert.equal(
      update.market_balances[0].deposit_assets[0].outcomes[0].balance_on_book,
      "85.000000"
    );
    assert.equal("balances" in update, false);
  });

  it("normalizes live market_balance_update messages", () => {
    const message = parseMessageIn(
      JSON.stringify({
        type: "user",
        version: 1,
        data: {
          event_type: "market_balance_update",
          market_pubkey: "market-1",
          market_balance: marketBalance,
          timestamp: "2026-06-19T12:00:00Z",
        },
      })
    );

    assert.equal(message.type, "user");
    if (message.type !== "user") throw new Error("expected user message");
    assert.equal(message.data.event_type, "market_balance_update");
    if (message.data.event_type !== "market_balance_update") {
      throw new Error("expected market balance update");
    }
    assert.equal(message.data.market_pubkey, "market-1");
    assert.equal(
      message.data.market_balance.deposit_assets[0].outcomes[0].conditional_token,
      "trump-usdc-mint"
    );
  });

  it("normalizes REST user orders with market_balances", () => {
    const response = normalizeUserOrdersPayload({
      user_pubkey: "user-1",
      orders: [],
      market_balances: [marketBalance],
      has_more: false,
    });

    assert.equal(response.market_balances[0].market_pubkey, "market-1");
    assert.equal(
      response.market_balances[0].deposit_assets[0].outcomes[0].balance_on_book,
      "85.000000"
    );
    assert.equal("balances" in response, false);
  });

  it("uses conditional_token for embedded order/fill balance deltas", () => {
    const balance = normalizeConditionalBalance({
      outcome_index: 0,
      conditional_token: "trump-usdc-mint",
      idle: "40.000000",
      on_book: "85.000000",
    });

    assert.equal(balance.conditional_token, "trump-usdc-mint");
    assert.equal("mint" in balance, false);
  });

  it("rejects old balance payload names", () => {
    assert.throws(
      () =>
        normalizeUserUpdate({
          event_type: "balance_update",
          market_pubkey: "market-1",
          orderbook_id: "old-orderbook",
          balance: { outcomes: [] },
          timestamp: "2026-06-19T12:00:00Z",
        } as never),
      /balance_update/
    );

    assert.throws(
      () =>
        normalizeUserOrdersPayload({
          user_pubkey: "user-1",
          orders: [],
          balances: [],
          has_more: false,
        } as never),
      /market_balances/
    );
  });
});

describe("limit order decoding", () => {
  const supported = () => userOrder("book-1", "base-1");
  const restPayload = (orders: ReturnType<typeof userOrder>[]) => ({
    user_pubkey: "user-1",
    orders,
    market_balances: [marketBalance],
    next_cursor: "next-page",
    has_more: true,
  });
  const frame = (data: unknown) => JSON.stringify({ type: "user", version: 1, data });

  it("preserves limit REST orders, balances, and pagination", () => {
    const response = normalizeUserOrdersPayload(restPayload([supported()]));
    assert.equal(response.orders.length, 1);
    assert.equal(response.orders[0].order_hash, "order-book-1");
    assert.deepEqual(response.market_balances, [marketBalance]);
    assert.equal(response.next_cursor, "next-page");
    assert.equal(response.has_more, true);
    assert.equal(convertSnapshotOrders(response.orders).isEmpty(), false);

    const empty = normalizeUserOrdersPayload(restPayload([]));
    assert.deepEqual(empty.orders, []);
    assert.deepEqual(empty.market_balances, [marketBalance]);
    assert.equal(empty.next_cursor, "next-page");
    assert.equal(empty.has_more, true);
  });

  it("preserves all sibling account data through the WebSocket entry point", () => {
    const notifications = [{ id: "notice", notification_type: "global", title: "Notice", message: "Fixture" }];
    const globalDeposits = [{ mint: "quote", balance: "9.250000" }];
    const message = parseMessageIn(frame({
      event_type: "snapshot",
      orders: [supported()],
      market_balances: [marketBalance],
      global_deposits: globalDeposits,
      notifications,
      nonce: 17,
    }));
    assert.ok(message.type === "user" && message.data.event_type === "snapshot");
    assert.equal(message.data.orders.length, 1);
    assert.equal(message.data.orders[0].order_hash, "order-book-1");
    assert.deepEqual(message.data.market_balances, [marketBalance]);
    assert.deepEqual(message.data.global_deposits, globalDeposits);
    assert.deepEqual(message.data.notifications, notifications);
    assert.equal(message.data.nonce, 17);
  });

  it("rejects limit orders with missing amounts", () => {
    for (const field of ["amount_in", "amount_out"] as const) {
      const invalid = { ...supported(), [field]: undefined } as never;
      assert.throws(() => normalizeUserOrdersPayload(restPayload([supported(), invalid])));
      assert.throws(() => parseMessageIn(frame({
        event_type: "snapshot", orders: [supported(), invalid], market_balances: [marketBalance],
      })));
    }
  });

  it("processes live limit orders", () => {
    const state = new UserOpenLimitOrders();
    const message = parseMessageIn(frame({
      event_type: "order", order_type: "limit", market_pubkey: "market-1", orderbook_id: "book-1",
      timestamp: "2026-01-01T00:00:00Z", order: { ...supported(), is_maker: true, fill_amount: "0" },
    }));
    assert.ok(message.type === "user" && message.data.event_type === "order");
    state.upsert(message.data);
    assert.equal(state.isEmpty(), false);
  });

  it("rejects invalid wire kinds rather than relabelling them as limit", () => {
    for (const orderType of ["market", undefined]) {
      const invalid = { ...supported(), order_type: orderType } as never;
      assert.throws(() => normalizeUserOrdersPayload(restPayload([invalid])), /order_type limit/);
      assert.throws(() => parseMessageIn(frame({
        event_type: "snapshot", orders: [invalid], market_balances: [marketBalance],
      })), /order_type limit/);
      assert.throws(() => parseMessageIn(frame({
        event_type: "order", order_type: orderType, market_pubkey: "market-1", orderbook_id: "book-1",
        timestamp: "2026-01-01T00:00:00Z", order: { ...supported(), is_maker: true, fill_amount: "0" },
      })), /order_type limit/);
    }
  });

  it("rejects live limit orders without an order payload", () => {
    for (const order of [null, undefined]) {
      assert.throws(() => parseMessageIn(frame({
        event_type: "order", order_type: "limit", market_pubkey: "market-1", orderbook_id: "book-1",
        timestamp: "2026-01-01T00:00:00Z", order,
      })));
    }
  });
});
