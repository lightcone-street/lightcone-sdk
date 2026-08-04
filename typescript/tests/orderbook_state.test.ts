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
  seq: bigint,
  bids: WsBookLevel[],
  asks: WsBookLevel[],
  resync = false,
): OrderBook {
  return {
    orderbook_id: "ob1" as OrderBookId,
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
        orderBook(true, 1n, [level(Side.Bid, "50", "10")], [level(Side.Ask, "51", "5")]),
      ),
      { kind: "applied" },
    );
    assert.equal(snapshot.bids().size, 1);
    assert.equal(snapshot.asks().size, 1);
    assert.equal(snapshot.bestBid(), "50");
    assert.equal(snapshot.bestAsk(), "51");

    assert.deepStrictEqual(
      snapshot.apply(
        orderBook(true, 2n, [level(Side.Bid, "49", "20")], [level(Side.Ask, "52", "8")]),
      ),
      { kind: "applied" },
    );
    assert.equal(snapshot.bids().size, 1);
    assert.equal(snapshot.asks().size, 1);
    assert.equal(snapshot.bestBid(), "49");
    assert.equal(snapshot.bestAsk(), "52");
  });

  it("discards equal and lower revisions", () => {
    const snapshot = new OrderbookState("ob1" as OrderBookId);
    snapshot.apply(orderBook(true, 42n, [level(Side.Bid, "50", "10")], []));
    assert.equal(snapshot.seq, 42n);

    // Lower/equal revisions cannot overwrite the accepted snapshot.
    assert.deepStrictEqual(
      snapshot.apply(orderBook(true, 7n, [level(Side.Bid, "49", "20")], [])),
      { kind: "discarded_stale" },
    );
    assert.deepStrictEqual(snapshot.apply(orderBook(true, 42n, [], [])), {
      kind: "discarded_stale",
    });
    assert.equal(snapshot.seq, 42n);
    assert.equal(snapshot.bestBid(), "50");
  });

  it("accepts a lower fresh revision after resubscribe", () => {
    const snapshot = new OrderbookState("ob1" as OrderBookId);
    snapshot.apply(
      orderBook(true, 42n, [level(Side.Bid, "50", "10")], [level(Side.Ask, "51", "5")]),
    );

    assert.deepStrictEqual(snapshot.apply(orderBook(false, 0n, [], [], true)), {
      kind: "refresh_required",
      reason: { kind: "server_resync" },
    });
    // Resync leaves the book untouched.
    assert.equal(snapshot.seq, 42n);
    assert.equal(snapshot.bids().size, 1);

    // A lower real engine revision in the fresh generation must apply.
    snapshot.beginGeneration();
    assert.deepStrictEqual(
      snapshot.apply(
        orderBook(true, 7n, [level(Side.Bid, "48", "5")], [level(Side.Ask, "52", "2")]),
      ),
      { kind: "applied" },
    );
    assert.equal(snapshot.seq, 7n);
    assert.equal(snapshot.bestBid(), "48");
    assert.equal(snapshot.bestAsk(), "52");
  });

  it("data frames replace regardless of snapshot flag", () => {
    const snapshot = new OrderbookState("ob1" as OrderBookId);
    snapshot.apply(
      orderBook(true, 1n, [level(Side.Bid, "50", "10")], [level(Side.Ask, "51", "5")]),
    );

    // Every non-resync data frame is a snapshot by contract — the
    // is_snapshot flag is not consulted, so a server omitting it cannot
    // freeze the book.
    assert.deepStrictEqual(
      snapshot.apply(orderBook(false, 2n, [level(Side.Bid, "49", "20")], [])),
      { kind: "applied" },
    );
    assert.equal(snapshot.seq, 2n);
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
          1n,
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
      orderBook(true, 1n, [level(Side.Bid, "50", "10")], [level(Side.Ask, "52", "5")]),
    );
    assert.equal(snapshot.midPrice(), "51");
    assert.equal(snapshot.spread(), "2");
  });

  it("clear resets state", () => {
    const snapshot = new OrderbookState("ob1" as OrderBookId);
    snapshot.apply(
      orderBook(true, 1n, [level(Side.Bid, "50", "10")], [level(Side.Ask, "51", "5")]),
    );
    snapshot.clear();
    assert.equal(snapshot.isEmpty(), true);
    assert.equal(snapshot.seq, 0n);
  });

  it("accepts forward gaps and preserves truncation metadata", () => {
    const snapshot = new OrderbookState("ob1" as OrderBookId);
    snapshot.apply(orderBook(true, 10n, [], []));
    const update = orderBook(true, 15n, [level(Side.Bid, "50", "1")], []);
    update.bids_truncated = true;
    assert.deepStrictEqual(snapshot.apply(update), { kind: "applied" });
    assert.equal(snapshot.seq, 15n);
    assert.equal(snapshot.bidsTruncated, true);
    assert.equal(snapshot.asksTruncated, false);
  });

  it("keeps aggregation generations separate", () => {
    const full = new OrderbookState("ob1" as OrderBookId);
    const grouped = new OrderbookState("ob1" as OrderBookId, {
      nSigFigs: 5,
      mantissa: 2,
    });
    const fullFrame = orderBook(true, 100n, [], []);
    const groupedFrame = orderBook(true, 3n, [], []);
    groupedFrame.n_sig_figs = 5;
    groupedFrame.mantissa = 2;
    assert.deepStrictEqual(full.apply(fullFrame), { kind: "applied" });
    assert.deepStrictEqual(grouped.apply(groupedFrame), { kind: "applied" });
    assert.deepStrictEqual(full.apply(groupedFrame), { kind: "subscription_mismatch" });
    assert.equal(full.seq, 100n);
    assert.equal(grouped.seq, 3n);
  });
});
