// Snapshot of current prices for every active deposit mint in `global_deposit_tokens`.
// Ported from the REST portion of the Rust `examples/deposit_asset_prices.rs` (the
// live WS subscription it follows up with is part of the separately-ported streaming
// layer). Public endpoint, no auth. ReScript surface: `PriceHistory`; the compiled
// DepositAssetPrices.res.mjs is the JS example.
let main = async () => {
  let client = Common__Example.client()

  switch await PriceHistory.getDepositAssetPricesSnapshot(client) {
  | Ok({prices}) =>
    let entries = prices->Dict.toArray
    Console.log(`deposit-asset-prices-snapshot: ${Int.toString(Array.length(entries))} entries`)
    entries
    ->Array.slice(~start=0, ~end=10)
    ->Array.forEach(((mint, price)) => Console.log(`  ${mint} -> ${price}`))
  | Error(error) => Console.error(SdkError.toMessage(error))
  }
}

let _ = main()
