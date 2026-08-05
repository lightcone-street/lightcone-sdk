import { describe, it } from "node:test";
import assert from "node:assert/strict";
import { OrderStatus } from "../src/domain/order";
import {
  normalizeConditionalBalance,
  normalizeUserOrdersPayload,
  normalizeUserUpdate,
} from "../src/domain/order/wire";
import { OrderUpdateType } from "../src/shared";
import { parseMessageIn } from "../src/ws";

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

  it("normalizes limit-order expiration events", () => {
    const message = parseMessageIn(
      JSON.stringify({
        type: "user",
        version: 1,
        data: {
          event_type: "order",
          order_type: "limit",
          market_pubkey: "market-1",
          orderbook_id: "orderbook-1",
          timestamp: "2026-08-05T12:00:00Z",
          type: "EXPIRATION",
          order: {
            order_hash: "order-1",
            price: "0.5",
            is_maker: true,
            remaining: "0",
            filled: "1",
            fill_amount: "0",
            side: "bid",
            created_at: 0,
            base_mint: "base",
            quote_mint: "quote",
            outcome_index: 0,
            status: "EXPIRED",
          },
        },
      })
    );

    assert.equal(message.type, "user");
    if (message.type !== "user") throw new Error("expected user message");
    assert.equal(message.data.event_type, "order");
    if (
      message.data.event_type !== "order" ||
      message.data.order_type !== "limit"
    ) {
      throw new Error("expected limit order update");
    }
    assert.equal(message.data.type, OrderUpdateType.Expiration);
    assert.equal(message.data.order.status, OrderStatus.Expired);
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
