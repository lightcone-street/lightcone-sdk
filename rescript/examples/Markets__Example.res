// Browse markets — featured + first page. ReScript surface: the result-returning
// domain modules (idiomatic). Compiled output (Markets.res.mjs) is the JS example.
let main = async () => {
  let client = Common__Example.client()

  switch await Market.featured(client) {
  | Ok(featured) => Console.log(`Featured markets: ${Int.toString(Array.length(featured))}`)
  | Error(error) => Console.error(SdkError.toMessage(error))
  }

  switch await Market.get(client, ~limit=5) {
  | Ok({markets}) =>
    Console.log(`First ${Int.toString(Array.length(markets))} markets:`)
    markets->Array.forEach(market => Console.log(`  ${market.slug} — ${market.name}`))
  | Error(error) => Console.error(SdkError.toMessage(error))
  }
}

let _ = main()
