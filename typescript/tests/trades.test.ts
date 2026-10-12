import { describe, it } from "node:test";
import assert from "node:assert/strict";

import {
  TradeHistory,
  tradeFromResponse,
  tradeFromWs,
  tradeResponseFromWire,
  wsTradeFromWire,
  wsTradeId,
  wsTradePrice,
  type Trade,
  type TradeResponse,
} from "../src/domain/trade";
import { SdkError } from "../src/error";
import { parseMessageIn } from "../src/ws";
import { asOrderBookId, parseJsonExact, Side } from "../src/shared";

/** Fill fact as `redis_listener::handle_committed` relays it on `trades`. */
export function fillFact(): Record<string, unknown> {
  return {
    effect_id: "812:1",
    committed_revision: 812,
    projection_generation: 3,
    actionable: true,
    event_type: "fill",
    fill_id: "0d6a6f3e-5b9b-4b43-9f0f-6c7c1f3f9a10:2:0",
    execution_id: "5f7e3c2a-8a39-4f55-a1d6-2b6f0c1b9d11",
    evidence_id: "0d6a6f3e-5b9b-4b43-9f0f-6c7c1f3f9a10",
    event_index: 2,
    leg_index: 0,
    orderbook_id: "j749bQAbDsBAiyDs2Tj868heQj1b5KVp98ZrjiZd56a",
    market_pubkey: "A9Bxkkc4nah517EjgjwSafwspGnmU9s1Ei5PTo5ZkJd9",
    base_mint: "BaseMint111111111111111111111111111111111111",
    quote_mint: "QuoteMint11111111111111111111111111111111111",
    base_amount: "4.00000000",
    quote_amount: "2.200000",
    maker_order_hash: "aa".repeat(32),
    taker_order_hash: "bb".repeat(32),
    maker_wallet: "Maker111111111111111111111111111111111111111",
    taker_wallet: "Taker111111111111111111111111111111111111111",
    taker_side: 1,
    maker_fee_estimate: "-0.000110",
    taker_fee_estimate: "0.006600",
    fee_mint: "QuoteMint11111111111111111111111111111111111",
    maker_fee_bps: -5,
    taker_fee_bps: 30,
    executed_at: "2026-09-29T12:00:00+00:00",
  };
}

function sampleTradeResponse(): TradeResponse {
  return {
    id: 456n,
    trade_id: "5f7e3c2a-8a39-4f55-a1d6-2b6f0c1b9d11:0:3",
    orderbook_id: asOrderBookId("ob_123"),
    taker_pubkey: "taker123",
    maker_pubkey: "maker456",
    side: Side.Bid,
    size: "10.000000",
    price: "5.000000",
    taker_fee_estimate: "0.003250",
    maker_fee_estimate: "0",
    executed_at: 1_740_076_800_000n,
  };
}

function isValidation(error: unknown): boolean {
  return error instanceof SdkError && error.variant === "Validation";
}

describe("trade wire", () => {
  it("decodes a fill fact and derives its price and REST id", () => {
    const trade = wsTradeFromWire(fillFact());
    assert.equal(trade.commit.committed_revision, 812n);
    assert.equal(trade.taker_side, Side.Ask);
    assert.equal(trade.base_amount, "4.00000000");
    assert.equal(wsTradePrice(trade), "0.55");
    assert.equal(trade.maker_fee_estimate, "-0.000110");
    assert.equal(wsTradeId(trade), "5f7e3c2a-8a39-4f55-a1d6-2b6f0c1b9d11:0:3");
  });

  it("decodes the trades channel", () => {
    const message = parseMessageIn(JSON.stringify({ type: "trades", version: 0.1, data: fillFact() }));
    assert.equal(message.type, "trades");
    if (message.type !== "trades") return;
    assert.equal(message.data.leg_index, 0);
    assert.equal(message.data.fill_id, "0d6a6f3e-5b9b-4b43-9f0f-6c7c1f3f9a10:2:0");
    assert.equal(message.data.commit.committed_revision, 812n);
    assert.equal(wsTradePrice(message.data), "0.55");
    assert.equal(message.data.executed_at.toISOString(), "2026-09-29T12:00:00.000Z");
  });

  it("decodes a user-channel fill as the trades payload", () => {
    const message = parseMessageIn(
      JSON.stringify({ type: "user", version: 0.1, data: { ...fillFact(), event_type: "fill" } }),
    );
    assert.equal(message.type, "user");
    if (message.type !== "user" || message.data.event_type !== "fill") {
      throw new Error("expected a user fill");
    }
    assert.equal(wsTradeId(message.data), "5f7e3c2a-8a39-4f55-a1d6-2b6f0c1b9d11:0:3");
    assert.equal(message.data.taker_side, Side.Ask);
  });

  it("decodes the backend's fractional-second RFC 3339 times", () => {
    for (const [executedAt, iso] of [
      ["2026-09-29T12:00:00.123456+00:00", "2026-09-29T12:00:00.123Z"],
      ["2026-09-29T12:00:00.987654321+00:00", "2026-09-29T12:00:00.987Z"],
    ] as const) {
      const trade = wsTradeFromWire({ ...fillFact(), executed_at: executedAt });
      assert.equal(trade.executed_at.toISOString(), iso);
    }
  });

  it("uses the REST fee estimate names and keeps a large id exact", () => {
    const trade = tradeResponseFromWire(
      parseJsonExact(
        '{"id":9007199254740993,"trade_id":"5f7e3c2a-8a39-4f55-a1d6-2b6f0c1b9d11:0:3",' +
          '"orderbook_id":"j749bQAbDsBAiyDs2Tj868heQj1b5KVp98ZrjiZd56a",' +
          '"taker_pubkey":"Taker111111111111111111111111111111111111111",' +
          '"maker_pubkey":"Maker111111111111111111111111111111111111111","side":"ask",' +
          '"size":"4.00000000","price":"0.5500","taker_fee_estimate":"0.006600",' +
          '"maker_fee_estimate":"-0.000110","executed_at":1790685600000}',
      ),
    );
    assert.equal(trade.id, 9_007_199_254_740_993n);
    assert.equal(trade.taker_fee_estimate, "0.006600");
    assert.equal(trade.maker_fee_estimate, "-0.000110");
  });

  it("rejects a rounded JSON number id", () => {
    assert.throws(
      () => tradeResponseFromWire({ ...sampleTradeResponse(), id: 2 ** 53 }),
      (error) => error instanceof SdkError && error.variant === "Serde" && /trade\.id/.test(error.message),
    );
  });
});

describe("trade conversion", () => {
  it("converts a REST trade", () => {
    const trade = tradeFromResponse(sampleTradeResponse());
    assert.equal(trade.orderbookId, "ob_123");
    assert.equal(trade.tradeId, "5f7e3c2a-8a39-4f55-a1d6-2b6f0c1b9d11:0:3");
    assert.equal(trade.cursorId, 456n);
    assert.equal(trade.price, "5.000000");
    assert.equal(trade.size, "10.000000");
    assert.equal(trade.side, Side.Bid);
  });

  it("converts a WS fill fact to the REST trade identity", () => {
    const trade = tradeFromWs(wsTradeFromWire(fillFact()));
    assert.equal(trade.orderbookId, "j749bQAbDsBAiyDs2Tj868heQj1b5KVp98ZrjiZd56a");
    assert.equal(trade.tradeId, "5f7e3c2a-8a39-4f55-a1d6-2b6f0c1b9d11:0:3");
    assert.equal(trade.cursorId, undefined);
    assert.equal(trade.price, "0.55");
    assert.equal(trade.size, "4.00000000");
    assert.equal(trade.side, Side.Ask);
  });

  it("rejects invalid values instead of defaulting them", () => {
    assert.throws(() => tradeFromResponse({ ...sampleTradeResponse(), price: "not-a-price" }), isValidation);
    assert.throws(() => tradeFromResponse({ ...sampleTradeResponse(), size: "" }), isValidation);
    assert.throws(
      () => tradeFromResponse({ ...sampleTradeResponse(), executed_at: (1n << 63n) - 1n }),
      isValidation,
    );
    assert.throws(
      () => tradeFromWs(wsTradeFromWire({ ...fillFact(), base_amount: "0" })),
      isValidation,
    );
  });
});

describe("TradeHistory", () => {
  function at(seconds: number): Date {
    return new Date((1_790_000_000 + seconds) * 1000);
  }

  function makeTrade(id: string, price: number, size: number, seconds: number): Trade {
    return {
      orderbookId: asOrderBookId("ob1"),
      tradeId: id,
      cursorId: undefined,
      timestamp: at(seconds),
      price: String(price),
      size: String(size),
      side: Side.Bid,
    };
  }

  function ids(history: TradeHistory): string[] {
    return history.trades().map((trade) => trade.tradeId);
  }

  it("adds trades newest first", () => {
    const history = new TradeHistory(asOrderBookId("ob1"), 10);
    history.push(makeTrade("t1", 50, 5, 1));
    history.push(makeTrade("t2", 51, 3, 2));
    assert.equal(history.len(), 2);
    assert.equal(history.latest()?.tradeId, "t2");
  });

  it("evicts the oldest trade when full", () => {
    const history = new TradeHistory(asOrderBookId("ob1"), 3);
    for (const [id, seconds] of [["t1", 1], ["t2", 2], ["t3", 3], ["t4", 4]] as const) {
      history.push(makeTrade(id, 50, 1, seconds));
    }
    assert.deepEqual(ids(history), ["t4", "t3", "t2"]);
  });

  it("orders by execution time", () => {
    const history = new TradeHistory(asOrderBookId("ob1"), 10);
    history.push(makeTrade("t3", 52, 3, 3));
    history.push(makeTrade("t1", 50, 1, 1));
    history.push(makeTrade("t2", 51, 2, 2));
    assert.deepEqual(ids(history), ["t3", "t2", "t1"]);
  });

  it("keeps the newest arrival first for equal timestamps", () => {
    const history = new TradeHistory(asOrderBookId("ob1"), 10);
    history.push(makeTrade("a", 50, 1, 5));
    history.push(makeTrade("b", 50, 1, 5));
    assert.deepEqual(ids(history), ["b", "a"]);
  });

  it("drops an older trade when full", () => {
    const history = new TradeHistory(asOrderBookId("ob1"), 3);
    for (const [id, seconds] of [["t5", 5], ["t4", 4], ["t3", 3], ["t2", 2]] as const) {
      history.push(makeTrade(id, 50, 1, seconds));
    }
    assert.deepEqual(ids(history), ["t5", "t4", "t3"]);
  });

  it("deduplicates REST and WS trades", () => {
    const history = new TradeHistory(asOrderBookId("ob1"), 10);
    history.push(makeTrade("exec:0:1", 50, 1, 1));
    history.push({ ...makeTrade("exec:0:1", 50, 1, 1), cursorId: 77n });
    assert.equal(history.len(), 1);
    assert.equal(history.latest()?.cursorId, 77n);
  });

  it("never changes a pushed trade", () => {
    const shared = makeTrade("exec:0:1", 50, 1, 1);
    const first = new TradeHistory(asOrderBookId("ob1"), 10);
    const second = new TradeHistory(asOrderBookId("ob1"), 10);
    first.push(shared);
    second.push(shared);
    first.push({ ...shared, cursorId: 77n });
    assert.equal(first.latest()?.cursorId, 77n);
    assert.equal(second.latest()?.cursorId, undefined);
    assert.equal(shared.cursorId, undefined);
  });

  it("replaces all trades", () => {
    const history = new TradeHistory(asOrderBookId("ob1"), 10);
    history.push(makeTrade("t1", 50, 1, 1));
    history.replace([makeTrade("a", 49, 1, 2), makeTrade("b", 50, 2, 1)]);
    assert.equal(history.len(), 2);
    assert.equal(history.latest()?.tradeId, "a");
  });

  it("treats zero capacity as disabled", () => {
    const history = new TradeHistory(asOrderBookId("ob1"), 0);
    history.push(makeTrade("t1", 50, 1, 1));
    assert.ok(history.isEmpty());
  });
});
