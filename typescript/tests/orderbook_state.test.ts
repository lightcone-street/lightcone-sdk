import { describe, it } from "node:test";
import assert from "node:assert/strict";
import { OrderbookState } from "../src/domain/orderbook/state";
import type { OrderBook, WsBookLevel } from "../src/domain/orderbook/wire";
import { Side, type OrderBookId } from "../src/shared";

function level(side: Side, price: string, size: string): WsBookLevel {
  return { side, price, size };
}

function orderBook(
  isSnapshot: boolean,
  seq: number,
  bids: WsBookLevel[],
  asks: WsBookLevel[],
  resync = false,
): OrderBook {
  return {
    orderbook_id: "ob_test" as OrderBookId,
    is_snapshot: isSnapshot,
    seq,
    resync,
    bids,
    asks,
  };
}

describe("OrderbookState", () => {
  it("snapshot replaces state", () => {
    const snapshot = new OrderbookState("ob1" as OrderBookId);
    assert.deepStrictEqual(
      snapshot.apply(
        orderBook(true, 1, [level(Side.Bid, "50", "10")], [level(Side.Ask, "51", "5")]),
      ),
      { kind: "applied" },
    );
    assert.equal(snapshot.bids().size, 1);
    assert.equal(snapshot.asks().size, 1);
    assert.equal(snapshot.bestBid(), "50");
    assert.equal(snapshot.bestAsk(), "51");

    assert.deepStrictEqual(
      snapshot.apply(
        orderBook(true, 2, [level(Side.Bid, "49", "20")], [level(Side.Ask, "52", "8")]),
      ),
      { kind: "applied" },
    );
    assert.equal(snapshot.bids().size, 1);
    assert.equal(snapshot.asks().size, 1);
    assert.equal(snapshot.bestBid(), "49");
    assert.equal(snapshot.bestAsk(), "52");
  });

  it("delta merges with snapshot", () => {
    const snapshot = new OrderbookState("ob1" as OrderBookId);
    snapshot.apply(
      orderBook(true, 1, [level(Side.Bid, "50", "10")], [level(Side.Ask, "51", "5")]),
    );
    assert.deepStrictEqual(
      snapshot.apply(
        orderBook(
          false,
          2,
          [level(Side.Bid, "49", "15"), level(Side.Bid, "48", "3")],
          [level(Side.Ask, "52", "2")],
        ),
      ),
      { kind: "applied" },
    );
    assert.equal(snapshot.bids().size, 3);
    assert.equal(snapshot.asks().size, 2);
    assert.equal(snapshot.bestBid(), "50");
    assert.equal(snapshot.bestAsk(), "51");
  });

  it("zero size removes level", () => {
    const snapshot = new OrderbookState("ob1" as OrderBookId);
    snapshot.apply(
      orderBook(true, 1, [level(Side.Bid, "50", "10")], [level(Side.Ask, "51", "5")]),
    );
    assert.deepStrictEqual(
      snapshot.apply(orderBook(false, 2, [level(Side.Bid, "50", "0")], [])),
      { kind: "applied" },
    );
    assert.equal(snapshot.bids().size, 0);
    assert.equal(snapshot.bestBid(), undefined);
  });

  it("stale delta is dropped", () => {
    const snapshot = new OrderbookState("ob1" as OrderBookId);
    snapshot.apply(
      orderBook(true, 1, [level(Side.Bid, "50", "10")], [level(Side.Ask, "51", "5")]),
    );
    snapshot.apply(orderBook(false, 2, [level(Side.Bid, "49", "20")], []));
    assert.equal(snapshot.seq, 2);
    assert.equal(snapshot.bids().size, 2);

    // Stale delta (seq < current) should be ignored
    assert.deepStrictEqual(
      snapshot.apply(orderBook(false, 1, [level(Side.Bid, "50", "0")], [])),
      { kind: "ignored", reason: { kind: "stale_delta", current: 2, got: 1 } },
    );
    assert.equal(snapshot.seq, 2);
    assert.equal(snapshot.bids().size, 2);

    // Duplicate seq should also be ignored
    assert.deepStrictEqual(
      snapshot.apply(orderBook(false, 2, [level(Side.Bid, "50", "0")], [])),
      { kind: "ignored", reason: { kind: "stale_delta", current: 2, got: 2 } },
    );
    assert.equal(snapshot.bids().size, 2);

    // Snapshot always applies regardless of seq
    assert.deepStrictEqual(
      snapshot.apply(orderBook(true, 1, [level(Side.Bid, "48", "5")], [])),
      { kind: "applied" },
    );
    assert.equal(snapshot.seq, 1);
    assert.equal(snapshot.bids().size, 1);
  });

  it("gap delta is detected and not applied", () => {
    const snapshot = new OrderbookState("ob1" as OrderBookId);
    snapshot.apply(
      orderBook(true, 1, [level(Side.Bid, "50", "10")], [level(Side.Ask, "51", "5")]),
    );

    assert.deepStrictEqual(
      snapshot.apply(orderBook(false, 3, [level(Side.Bid, "49", "20")], [])),
      { kind: "refresh_required", reason: { kind: "sequence_gap", expected: 2, got: 3 } },
    );
    assert.equal(snapshot.seq, 1);
    assert.equal(snapshot.bids().size, 1);
    assert.equal(snapshot.bestBid(), "50");

    assert.deepStrictEqual(
      snapshot.apply(orderBook(false, 4, [level(Side.Bid, "48", "20")], [], true)),
      { kind: "refresh_required", reason: { kind: "server_resync", got: 4 } },
    );

    assert.deepStrictEqual(
      snapshot.apply(orderBook(false, 5, [level(Side.Bid, "47", "20")], [])),
      { kind: "ignored", reason: { kind: "already_awaiting_snapshot", got: 5 } },
    );
  });

  it("delta before snapshot is detected as gap", () => {
    const snapshot = new OrderbookState("ob1" as OrderBookId);

    assert.deepStrictEqual(
      snapshot.apply(orderBook(false, 1, [level(Side.Bid, "50", "10")], [])),
      { kind: "refresh_required", reason: { kind: "missing_snapshot", got: 1 } },
    );
    assert.equal(snapshot.seq, 0);
    assert.equal(snapshot.isEmpty(), true);
  });

  it("snapshot after gap restores state", () => {
    const snapshot = new OrderbookState("ob1" as OrderBookId);
    snapshot.apply(
      orderBook(true, 1, [level(Side.Bid, "50", "10")], [level(Side.Ask, "51", "5")]),
    );
    snapshot.apply(orderBook(false, 3, [level(Side.Bid, "49", "20")], []));

    assert.deepStrictEqual(
      snapshot.apply(
        orderBook(true, 10, [level(Side.Bid, "49", "5")], [level(Side.Ask, "51", "7")]),
      ),
      { kind: "applied" },
    );
    assert.equal(snapshot.seq, 10);
    assert.equal(snapshot.bestBid(), "49");
    assert.equal(snapshot.bestAsk(), "51");
  });

  it("resync requires refresh and leaves state unchanged", () => {
    const snapshot = new OrderbookState("ob1" as OrderBookId);
    snapshot.apply(
      orderBook(true, 1, [level(Side.Bid, "50", "10")], [level(Side.Ask, "51", "5")]),
    );

    assert.deepStrictEqual(
      snapshot.apply(orderBook(false, 2, [level(Side.Bid, "49", "20")], [], true)),
      { kind: "refresh_required", reason: { kind: "server_resync", got: 2 } },
    );
    assert.equal(snapshot.seq, 1);
    assert.equal(snapshot.bestBid(), "50");

    assert.deepStrictEqual(
      snapshot.apply(orderBook(false, 3, [level(Side.Bid, "48", "20")], [])),
      { kind: "ignored", reason: { kind: "already_awaiting_snapshot", got: 3 } },
    );
  });
});
