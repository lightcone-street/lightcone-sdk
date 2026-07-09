// Exercise several Metrics endpoints end-to-end — a wire-type smoke test for the
// public metrics surface (no auth required): if any field decode is wrong this
// example fails. Ported from the Rust `examples/metrics_all.rs`. ReScript surface:
// the result-returning `Metrics` domain module; the compiled MetricsAll.res.mjs is
// the JS example. Each call is an independent `switch` so one failure is isolated.
let main = async () => {
  let client = Common__Example.client()

  // ── Platform ──────────────────────────────────────────────────────────────
  switch await Metrics.Client.platform(client) {
  | Ok(platform) =>
    Console.log(
      `platform: volume_24h_usd=${platform.volume24hUsd}, volume_7d_usd=${platform.volume7dUsd}, ` ++
      `open_interest_usd=${platform.openInterestUsd}, active_markets=${Float.toString(platform.activeMarkets)}, ` ++
      `active_orderbooks=${Float.toString(platform.activeOrderbooks)}`,
    )
    Console.log(`  deposit token volumes: ${Int.toString(Array.length(platform.depositTokenVolumes))}`)
  | Error(error) => Console.error(SdkError.toMessage(error))
  }

  // ── Markets list ──────────────────────────────────────────────────────────
  switch await Metrics.Client.markets(client) {
  | Ok({markets, total}) =>
    Console.log(`markets: ${Int.toString(Array.length(markets))} entries (total=${Float.toString(total)})`)
    markets
    ->Array.slice(~start=0, ~end=3)
    ->Array.forEach(entry =>
      Console.log(
        `  - ${entry.marketName->Option.getOr("?")} — volume_24h_usd=${entry.volume24hUsd} ` ++
        `(platform_share_24h=${entry.platformVolumeShare24hPct}%)`,
      )
    )
  | Error(error) => Console.error(SdkError.toMessage(error))
  }

  // ── Categories ────────────────────────────────────────────────────────────
  switch await Metrics.Client.categories(client) {
  | Ok({categories}) =>
    Console.log(`categories: ${Int.toString(Array.length(categories))}`)
    switch categories[0] {
    | Some(category) =>
      Console.log(
        `  first '${category.category}': volume_24h_usd=${category.volume24hUsd}, ` ++
        `unique_traders_24h=${Float.toString(category.uniqueTraders24h)}`,
      )
    | None => ()
    }
  | Error(error) => Console.error(SdkError.toMessage(error))
  }

  // ── Deposit tokens (platform-wide) ────────────────────────────────────────
  switch await Metrics.Client.depositTokens(client) {
  | Ok({depositTokens}) =>
    Console.log(`deposit tokens: ${Int.toString(Array.length(depositTokens))}`)
    depositTokens
    ->Array.slice(~start=0, ~end=3)
    ->Array.forEach(token =>
      Console.log(`  ${token.symbol->Option.getOr("?")} — volume_24h_usd=${token.volume24hUsd}`)
    )
  | Error(error) => Console.error(SdkError.toMessage(error))
  }

  // ── Orderbook tickers (batch BBO + midpoint over REST) ────────────────────
  switch await Metrics.Client.orderbookTickers(client) {
  | Ok({tickers}) =>
    Console.log(`orderbook tickers: ${Int.toString(Array.length(tickers))}`)
    switch tickers[0] {
    | Some(ticker) =>
      Console.log(
        `  ${ticker.orderbookId}: best_bid=${ticker.bestBid->Option.getOr("-")} ` ++
        `best_ask=${ticker.bestAsk->Option.getOr("-")} midpoint=${ticker.midpoint->Option.getOr("-")}`,
      )
    | None => ()
    }
  | Error(error) => Console.error(SdkError.toMessage(error))
  }
}

let _ = main()
