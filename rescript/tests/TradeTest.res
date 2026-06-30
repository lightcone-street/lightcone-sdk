open RescriptBun.Test
open RescriptBun.Test.Expect

// Mirrors the Rust `test_trade_response_conversion` fixture: decode the REST
// envelope with the spice codec, then convert wire → domain.
describe("Trade domain", () => {
  test("decodes a REST trades response and converts to the domain shape", () => {
    let json = JSON.parseOrThrow(`{
      "orderbook_id": "ob_123",
      "trades": [{
        "id": 456,
        "trade_id": "taker_hash_maker_hash",
        "orderbook_id": "ob_123",
        "taker_pubkey": "taker123",
        "maker_pubkey": "maker456",
        "side": "bid",
        "size": "10.000000",
        "price": "5.000000",
        "taker_fee": "0.003250",
        "executed_at": 1740076800000
      }],
      "next_cursor": 456,
      "has_more": true
    }`)

    switch Trade.tradesResponse_decode(json) {
    | Ok(response) =>
      let page = Trade.pageOfTrades(response.trades, response.nextCursor, response.hasMore)
      expect(Array.length(page.trades))->toBe(1)
      let trade = page.trades->Array.getUnsafe(0)
      expect(trade.tradeId)->toBe("taker_hash_maker_hash")
      expect(trade.price)->toBe("5.000000")
      expect(trade.size)->toBe("10.000000")
      expect(trade.side)->toBe(Shared.Side.Bid)
      expect(trade.cursorId)->toEqual(Some(456.0))
      expect(trade.sequence)->toBe(0.0)
      expect(page.hasMore)->toBe(true)
      expect(page.nextCursor)->toEqual(Some(456.0))
    | Error(error) => expect("unexpected decode error: " ++ error.message)->toBe("decoded ok")
    }
  })
})
