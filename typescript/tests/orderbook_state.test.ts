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

  it("lower seq snapshot still applies (last-write-wins)", () => {
    const snapshot = new OrderbookState("ob1" as OrderBookId);
    snapshot.apply(orderBook(true, 42, [level(Side.Bid, "50", "10")], []));
    assert.equal(snapshot.seq, 42);

    // A snapshot with a lower seq (e.g. queued behind a re-subscribe) still
    // replaces the book — seq never gates.
    assert.deepStrictEqual(
      snapshot.apply(orderBook(true, 7, [level(Side.Bid, "49", "20")], [])),
      { kind: "applied" },
    );
    assert.equal(snapshot.seq, 7);
    assert.equal(snapshot.bestBid(), "49");
  });

  it("post-resync seq 0 snapshot applies", () => {
    const snapshot = new OrderbookState("ob1" as OrderBookId);
    snapshot.apply(
      orderBook(true, 42, [level(Side.Bid, "50", "10")], [level(Side.Ask, "51", "5")]),
    );

    assert.deepStrictEqual(snapshot.apply(orderBook(false, 0, [], [], true)), {
      kind: "refresh_required",
      reason: { kind: "server_resync" },
    });
    // Resync leaves the book untouched.
    assert.equal(snapshot.seq, 42);
    assert.equal(snapshot.bids().size, 1);

    // The fresh snapshot after re-subscribing is always seq 0 and MUST
    // apply — gating on seq here would freeze the book forever.
    assert.deepStrictEqual(
      snapshot.apply(
        orderBook(true, 0, [level(Side.Bid, "48", "5")], [level(Side.Ask, "52", "2")]),
      ),
      { kind: "applied" },
    );
    assert.equal(snapshot.seq, 0);
    assert.equal(snapshot.bestBid(), "48");
    assert.equal(snapshot.bestAsk(), "52");
  });

  it("data frames replace regardless of snapshot flag", () => {
    const snapshot = new OrderbookState("ob1" as OrderBookId);
    snapshot.apply(
      orderBook(true, 1, [level(Side.Bid, "50", "10")], [level(Side.Ask, "51", "5")]),
    );

    // Every non-resync data frame is a snapshot by contract — the
    // is_snapshot flag is not consulted, so a server omitting it cannot
    // freeze the book.
    assert.deepStrictEqual(
      snapshot.apply(orderBook(false, 2, [level(Side.Bid, "49", "20")], [])),
      { kind: "applied" },
    );
    assert.equal(snapshot.seq, 2);
    assert.equal(snapshot.bids().size, 1);
    assert.equal(snapshot.asks().size, 0);
    assert.equal(snapshot.bestBid(), "49");
  });

  it("zero size levels are skipped", () => {
    const snapshot = new OrderbookState("ob1" as OrderBookId);
    assert.deepStrictEqual(
      snapshot.apply(
        orderBook(
          true,
          1,
          [level(Side.Bid, "50", "10"), level(Side.Bid, "49", "0")],
          [level(Side.Ask, "51", "5")],
        ),
      ),
      { kind: "applied" },
    );
    assert.equal(snapshot.bids().size, 1);
    assert.equal(snapshot.bestBid(), "50");
  });

  it("mid price and spread", () => {
    const snapshot = new OrderbookState("ob1" as OrderBookId);
    snapshot.apply(
      orderBook(true, 1, [level(Side.Bid, "50", "10")], [level(Side.Ask, "52", "5")]),
    );
    assert.equal(snapshot.midPrice(), "51");
    assert.equal(snapshot.spread(), "2");
  });

  it("clear resets state", () => {
    const snapshot = new OrderbookState("ob1" as OrderBookId);
    snapshot.apply(
      orderBook(true, 1, [level(Side.Bid, "50", "10")], [level(Side.Ask, "51", "5")]),
    );
    snapshot.clear();
    assert.equal(snapshot.isEmpty(), true);
    assert.equal(snapshot.seq, 0);
  });
});
