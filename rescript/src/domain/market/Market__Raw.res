// Market wire types — the exact JSON the backend sends for market discovery /
// metadata queries, plus the wire→domain conversions. Wire field names never
// change; decimal-valued fields stay strings; ids / counts / indices are
// floats. ISO-8601 wire timestamps are parsed to unix-ms floats in the
// conversions.

// ── Internal conversion helpers ───────────────────────────────────────────────
// ISO-8601 → unix milliseconds. Invalid strings yield NaN rather than throwing.
let parseTimestamp = (isoTimestamp: string): float => Date.fromString(isoTimestamp)->Date.getTime
let parseTimestampOpt = (isoTimestamp: option<string>): option<float> =>
  isoTimestamp->Option.map(value => parseTimestamp(value))

let formatValidationErrors = (label: string, identifier: string, messages: array<string>): string =>
  `${label} validation errors (${identifier}):\n` ++
    messages->Array.map(message => `  - ${message}`)->Array.join("\n")

// ── Wire types ────────────────────────────────────────────────────────────────
module OutcomeResponse = {
  @spice
  type t = {
    index: float,
    name: string,
    @spice.key("name_long") nameLong?: string,
    @spice.key("icon_url_low") iconUrlLow?: string,
    @spice.key("icon_url_medium") iconUrlMedium?: string,
    @spice.key("icon_url_high") iconUrlHigh?: string,
  }

  // Wire → domain outcome (resi-internal).
  let toOutcome = (source: t): result<Market__Model.Outcome.t, array<string>> =>
    switch Market__Model.resolveIconUrls(
      source.iconUrlLow,
      source.iconUrlMedium,
      source.iconUrlHigh,
    ) {
    | None => Error([`Missing icon URL for outcome: ${source.name}`])
    | Some((low, medium, high)) =>
      Ok({
        index: source.index,
        iconUrlLow: low,
        iconUrlMedium: medium,
        iconUrlHigh: high,
        name: source.name,
        nameLong: ?source.nameLong,
      })
    }
}

module ConditionalTokenResponse = {
  @spice
  type t = {
    id: float,
    @spice.key("outcome_index") outcomeIndex: float,
    @spice.key("token_address") tokenAddress: string,
    symbol?: string,
    uri?: string,
    outcome?: string,
    @spice.key("deposit_symbol") depositSymbol?: string,
    @spice.key("short_symbol") shortSymbol?: string,
    description?: string,
    @spice.key("icon_url_low") iconUrlLow?: string,
    @spice.key("icon_url_medium") iconUrlMedium?: string,
    @spice.key("icon_url_high") iconUrlHigh?: string,
    @spice.key("metadata_uri") metadataUri?: string,
    decimals?: float,
    @spice.key("created_at") createdAt: string,
  }
}

module DepositAssetResponse = {
  @spice
  type t = {
    @spice.key("display_name") displayName?: string,
    @spice.key("token_symbol") tokenSymbol?: string,
    symbol?: string,
    @spice.key("deposit_asset") depositAsset: Shared.pubkeyStr,
    id: float,
    @spice.key("market_pubkey") marketPubkey: Shared.pubkeyStr,
    vault: string,
    @spice.key("num_outcomes") numOutcomes: float,
    description?: string,
    @spice.key("icon_url_low") iconUrlLow?: string,
    @spice.key("icon_url_medium") iconUrlMedium?: string,
    @spice.key("icon_url_high") iconUrlHigh?: string,
    @spice.key("metadata_uri") metadataUri?: string,
    decimals?: float,
    @spice.key("min_order_size") minOrderSize?: string,
    @spice.key("conditional_mints") conditionalMints: array<ConditionalTokenResponse.t>,
    @spice.key("created_at") createdAt: string,
  }

  // Validated deposit asset + its conditional tokens + token metadata
  // (resi-internal: consumed by `MarketResponse.toMarket`).
  type validatedTokens = {
    token: Market__Model.DepositAsset.t,
    conditionals: array<Market__Model.ConditionalToken.t>,
    metadata: dict<Market__Model.TokenMetadata.t>,
  }

  let toValidatedTokens = (source: t): result<validatedTokens, array<string>> => {
    let errors: array<string> = []
    let conditionals: array<Market__Model.ConditionalToken.t> = []
    let metadata: dict<Market__Model.TokenMetadata.t> = Dict.make()
    let pubkey = source.depositAsset

    let (iconLow, iconMedium, iconHigh) = switch Market__Model.resolveIconUrls(
      source.iconUrlLow,
      source.iconUrlMedium,
      source.iconUrlHigh,
    ) {
    | Some(icons) => icons
    | None => {
        Array.push(errors, `Missing icon URL: ${source.depositAsset}`)
        ("", "", "")
      }
    }
    let shortSymbol = switch source.displayName {
    | Some(value) => value
    | None => {
        Array.push(errors, `Missing display name: ${source.depositAsset}`)
        ""
      }
    }
    let symbol = switch source.symbol {
    | Some(value) => value
    | None => {
        Array.push(errors, `Missing symbol: ${source.depositAsset}`)
        ""
      }
    }
    let decimals = switch source.decimals {
    | Some(value) => value
    | None => {
        Array.push(errors, `Missing decimals: ${source.depositAsset}`)
        0.0
      }
    }

    Dict.set(
      metadata,
      pubkey,
      {
        pubkey,
        symbol,
        shortSymbol,
        decimals,
        iconUrlLow: iconLow,
        iconUrlMedium: iconMedium,
        iconUrlHigh: iconHigh,
        name: shortSymbol,
      },
    )

    source.conditionalMints->Array.forEach(conditional => {
      let conditionalPubkey = conditional.tokenAddress
      let conditionalErrors: array<string> = []

      let conditionalDecimals = switch conditional.decimals {
      | Some(value) => value
      | None => {
          Array.push(conditionalErrors, `Missing decimals: ${conditionalPubkey}`)
          0.0
        }
      }
      let conditionalShortSymbol = switch conditional.shortSymbol->Option.orElse(conditional.symbol) {
      | Some(value) => value
      | None => {
          Array.push(conditionalErrors, `Missing short symbol: ${conditionalPubkey}`)
          ""
        }
      }
      let conditionalSymbol = conditional.symbol->Option.getOr(conditionalShortSymbol)
      let conditionalOutcome = switch conditional.outcome {
      | Some(value) => value
      | None => {
          Array.push(conditionalErrors, `Missing outcome: ${conditionalPubkey}`)
          ""
        }
      }

      if Array.length(conditionalErrors) > 0 {
        Array.push(errors, formatValidationErrors("Token", conditionalPubkey, conditionalErrors))
      } else {
        // Cross-fallback within the conditional's own icons, then fall back to the
        // parent deposit asset's icons.
        let (conditionalLow, conditionalMedium, conditionalHigh) = switch Market__Model.resolveIconUrls(
          conditional.iconUrlLow,
          conditional.iconUrlMedium,
          conditional.iconUrlHigh,
        ) {
        | Some(icons) => icons
        | None => (iconLow, iconMedium, iconHigh)
        }

        Dict.set(
          metadata,
          conditionalPubkey,
          {
            pubkey: conditionalPubkey,
            symbol: conditionalSymbol,
            shortSymbol: conditionalShortSymbol,
            decimals: conditionalDecimals,
            iconUrlLow: conditionalLow,
            iconUrlMedium: conditionalMedium,
            iconUrlHigh: conditionalHigh,
            name: conditionalOutcome,
          },
        )

        Array.push(
          conditionals,
          {
            id: conditional.id,
            depositSymbol: symbol,
            depositAsset: pubkey,
            outcomeIndex: conditional.outcomeIndex,
            iconUrlLow: conditionalLow,
            iconUrlMedium: conditionalMedium,
            iconUrlHigh: conditionalHigh,
            description: ?conditional.description,
            outcome: conditionalOutcome,
            mint: conditionalPubkey,
            name: conditionalOutcome,
            symbol: conditionalSymbol,
            shortSymbol: conditionalShortSymbol,
            decimals: conditionalDecimals,
          },
        )
      }
    })

    if Array.length(errors) > 0 {
      Error(errors)
    } else {
      Ok({
        token: {
          id: source.id,
          marketPda: source.marketPubkey,
          depositAsset: pubkey,
          numOutcomes: source.numOutcomes,
          name: shortSymbol,
          symbol,
          shortSymbol,
          description: ?source.description,
          decimals,
          minOrderSize: ?source.minOrderSize,
          iconUrlLow: iconLow,
          iconUrlMedium: iconMedium,
          iconUrlHigh: iconHigh,
        },
        conditionals,
        metadata,
      })
    }
  }
}

// Deposit assets registered for one market (returned directly; no domain
// conversion).
module DepositMintsResponse = {
  @spice
  type t = {
    @spice.key("market_pubkey") marketPubkey: Shared.pubkeyStr,
    @spice.key("deposit_assets") depositAssets: array<DepositAssetResponse.t>,
    total: float,
  }
}

// REST single-orderbook wire type. Consumed by the market conversion to build
// `Market__Model.OrderBookPair.t`; not returned directly by any client
// function.
module OrderbookResponse = {
  @spice
  type t = {
    id: float,
    @spice.key("market_pubkey") marketPubkey: Shared.pubkeyStr,
    @spice.key("orderbook_id") orderbookId: Shared.orderBookId,
    @spice.key("base_token") baseToken: string,
    @spice.key("quote_token") quoteToken: string,
    @spice.key("outcome_index") outcomeIndex?: float,
    @spice.key("tick_size") tickSize: float,
    @spice.key("total_bids") totalBids: float,
    @spice.key("total_asks") totalAsks: float,
    @spice.key("last_trade_price") lastTradePrice?: string,
    @spice.key("last_trade_time") lastTradeTime?: string,
    active: bool,
    @spice.key("created_at") createdAt: string,
    @spice.key("updated_at") updatedAt: string,
  }

  // Match the orderbook's base/quote mints against the market's conditional
  // tokens (resi-internal).
  let toOrderBookPair = (
    source: t,
    tokens: array<Market__Model.ConditionalToken.t>,
  ): result<Market__Model.OrderBookPair.t, array<string>> => {
    let errors: array<string> = []
    let baseMint = source.baseToken
    let quoteMint = source.quoteToken
    let base = tokens->Array.find(token => token.mint == baseMint)
    let quote = tokens->Array.find(token => token.mint == quoteMint)

    switch base {
    | None =>
      Array.push(errors, `Base token not found: orderbook: ${source.orderbookId}, base: ${baseMint}`)
    | Some(_) => ()
    }
    switch quote {
    | None =>
      Array.push(errors, `Quote token not found: orderbook: ${source.orderbookId}, quote: ${quoteMint}`)
    | Some(_) => ()
    }

    switch (base, quote) {
    | (Some(baseToken), Some(quoteToken)) =>
      Ok({
        id: source.id,
        outcomeIndex: source.outcomeIndex->Option.getOr(baseToken.outcomeIndex),
        marketPubkey: source.marketPubkey,
        orderbookId: source.orderbookId,
        base: baseToken,
        quote: quoteToken,
        tickSize: source.tickSize,
        totalBids: source.totalBids,
        totalAsks: source.totalAsks,
        lastTradePrice: ?source.lastTradePrice,
        lastTradeTime: ?parseTimestampOpt(source.lastTradeTime),
        active: source.active,
      })
    | _ => Error(errors)
    }
  }
}

module SearchOrderbook = {
  @spice
  type t = {
    @spice.key("orderbook_id") orderbookId: Shared.orderBookId,
    @spice.key("outcome_name") outcomeName: string,
    @spice.key("outcome_name_long") outcomeNameLong?: string,
    @spice.key("outcome_index") outcomeIndex: float,
    @spice.key("deposit_base_asset") depositBaseAsset: Shared.pubkeyStr,
    @spice.key("deposit_quote_asset") depositQuoteAsset: Shared.pubkeyStr,
    @spice.key("deposit_base_symbol") depositBaseSymbol: string,
    @spice.key("deposit_quote_symbol") depositQuoteSymbol: string,
    @spice.key("base_icon_url_low") baseIconUrlLow?: string,
    @spice.key("base_icon_url_medium") baseIconUrlMedium?: string,
    @spice.key("base_icon_url_high") baseIconUrlHigh?: string,
    @spice.key("quote_icon_url_low") quoteIconUrlLow?: string,
    @spice.key("quote_icon_url_medium") quoteIconUrlMedium?: string,
    @spice.key("quote_icon_url_high") quoteIconUrlHigh?: string,
    @spice.key("conditional_base_mint") conditionalBaseMint: Shared.pubkeyStr,
    @spice.key("conditional_quote_mint") conditionalQuoteMint: Shared.pubkeyStr,
    @spice.key("outcome_icon_url_low") outcomeIconUrlLow?: string,
    @spice.key("outcome_icon_url_medium") outcomeIconUrlMedium?: string,
    @spice.key("outcome_icon_url_high") outcomeIconUrlHigh?: string,
    @spice.key("conditional_base_symbol") conditionalBaseSymbol?: string,
    @spice.key("conditional_quote_symbol") conditionalQuoteSymbol?: string,
    @spice.key("latest_mid_price") latestMidPrice?: string,
  }
}

// Search / featured wire row (returned directly; no domain conversion).
module MarketSearchResult = {
  @spice
  type t = {
    slug: string,
    @spice.key("market_name") marketName: string,
    @spice.key("market_status") marketStatus: Market__Model.Status.t,
    category?: string,
    @spice.default([]) tags: array<string>,
    @spice.key("featured_rank") featuredRank: float,
    description?: string,
    @spice.key("icon_url_low") iconUrlLow?: string,
    @spice.key("icon_url_medium") iconUrlMedium?: string,
    @spice.key("icon_url_high") iconUrlHigh?: string,
    orderbooks: array<SearchOrderbook.t>,
  }
}

// ── Internal pair derivation (wire-independent) ───────────────────────────────
// Display priority for sorting: BTC/WBTC → 0, ETH/WETH → 1, SOL → 2, rest → tail.
let displayPriority = (symbol: string): int =>
  switch String.toUpperCase(symbol) {
  | "BTC" | "WBTC" => 0
  | "ETH" | "WETH" => 1
  | "SOL" => 2
  | _ => 255
  }

// Order deposit-asset pairs by base-token display priority, then alphabetically.
let sortPairsByDisplayPriority = (
  pairs: array<Market__Model.DepositAssetPair.t>,
): array<Market__Model.DepositAssetPair.t> =>
  pairs->Array.toSorted((left, right) => {
    let leftPriority = displayPriority(left.base.symbol)
    let rightPriority = displayPriority(right.base.symbol)
    if leftPriority == rightPriority {
      String.compare(left.base.symbol, right.base.symbol)
    } else {
      Int.compare(leftPriority, rightPriority)
    }
  })

// Unique base/quote deposit-asset pairs across the market's orderbook pairs,
// deduplicated by `(base_pubkey, quote_pubkey)`. Pairs whose base or quote
// deposit asset is absent from `depositAssets` are skipped.
let deriveDepositAssetPairs = (
  depositAssets: array<Market__Model.DepositAsset.t>,
  orderbookPairs: array<Market__Model.OrderBookPair.t>,
): array<Market__Model.DepositAssetPair.t> => {
  let seen: dict<Market__Model.DepositAssetPair.t> = Dict.make()
  orderbookPairs->Array.forEach(pair => {
    let base = depositAssets->Array.find(asset => asset.depositAsset == pair.base.depositAsset)
    let quote = depositAssets->Array.find(asset => asset.depositAsset == pair.quote.depositAsset)
    switch (base, quote) {
    | (Some(baseAsset), Some(quoteAsset)) => {
        let key = `${baseAsset.depositAsset}-${quoteAsset.depositAsset}`
        switch Dict.get(seen, key) {
        | Some(_) => ()
        | None => Dict.set(seen, key, {id: key, base: baseAsset, quote: quoteAsset})
        }
      }
    | _ => ()
    }
  })
  Dict.valuesToArray(seen)
}

// ── Market wire type + conversion ─────────────────────────────────────────────
module MarketResponse = {
  @spice
  type t = {
    @spice.key("market_name") marketName?: string,
    slug?: string,
    description?: string,
    definition?: string,
    outcomes: array<OutcomeResponse.t>,
    @spice.key("banner_image_url_low") bannerImageUrlLow?: string,
    @spice.key("banner_image_url_medium") bannerImageUrlMedium?: string,
    @spice.key("banner_image_url_high") bannerImageUrlHigh?: string,
    @spice.key("icon_url_low") iconUrlLow?: string,
    @spice.key("icon_url_medium") iconUrlMedium?: string,
    @spice.key("icon_url_high") iconUrlHigh?: string,
    category?: string,
    tags?: array<string>,
    @spice.key("featured_rank") featuredRank?: float,
    @spice.key("market_pubkey") marketPubkey: Shared.pubkeyStr,
    @spice.key("market_id") marketId: float,
    oracle: string,
    @spice.key("question_id") questionId: string,
    @spice.key("condition_id") conditionId: string,
    @spice.key("market_status") marketStatus: string,
    resolution?: Market__Model.Resolution.t,
    @spice.key("created_at") createdAt: string,
    @spice.key("activated_at") activatedAt?: string,
    @spice.key("settled_at") settledAt?: string,
    @spice.key("deposit_assets") depositAssets: array<DepositAssetResponse.t>,
    orderbooks: array<OrderbookResponse.t>,
  }

  // Wire → domain market. Every validation problem is collected; any error
  // makes the whole market invalid.
  let toMarket = (source: t): result<Market__Model.t, string> => {
    let errors: array<string> = []

    // Outcomes
    let outcomes: array<Market__Model.Outcome.t> = []
    source.outcomes->Array.forEach(outcomeResponse =>
      switch OutcomeResponse.toOutcome(outcomeResponse) {
      | Ok(validated) => Array.push(outcomes, validated)
      | Error(messages) =>
        messages->Array.forEach(message => Array.push(errors, `Outcome: ${message}`))
      }
    )

    // Tokens (deposit assets + conditional tokens + metadata)
    let depositAssets: array<Market__Model.DepositAsset.t> = []
    let conditionalTokens: array<Market__Model.ConditionalToken.t> = []
    let tokenMetadata: dict<Market__Model.TokenMetadata.t> = Dict.make()
    source.depositAssets->Array.forEach(depositAssetResponse =>
      switch DepositAssetResponse.toValidatedTokens(depositAssetResponse) {
      | Ok(validated) => {
          Array.push(depositAssets, validated.token)
          validated.conditionals->Array.forEach(conditional =>
            Array.push(conditionalTokens, conditional)
          )
          validated.metadata
          ->Dict.toArray
          ->Array.forEach(((key, value)) => Dict.set(tokenMetadata, key, value))
        }
      | Error(messages) => messages->Array.forEach(message => Array.push(errors, `Token: ${message}`))
      }
    )

    // Sort deposit assets and conditional tokens by symbol.
    let depositAssets = depositAssets->Array.toSorted((left, right) =>
      String.compare(left.symbol, right.symbol)
    )
    let conditionalTokens = conditionalTokens->Array.toSorted((left, right) =>
      String.compare(left.symbol, right.symbol)
    )

    // Orderbooks (resolved against the sorted conditional tokens).
    let orderbookPairs: array<Market__Model.OrderBookPair.t> = []
    source.orderbooks->Array.forEach(orderbookResponse =>
      switch OrderbookResponse.toOrderBookPair(orderbookResponse, conditionalTokens) {
      | Ok(pair) => Array.push(orderbookPairs, pair)
      | Error(messages) =>
        messages->Array.forEach(message => Array.push(errors, `OrderBook: ${message}`))
      }
    )

    let slug = switch source.slug {
    | Some(value) => value
    | None => {
        Array.push(errors, "Missing slug")
        ""
      }
    }
    let name = switch source.marketName {
    | Some(value) => value
    | None => {
        Array.push(errors, "Missing name")
        ""
      }
    }
    let status = switch Market__Model.Status.fromString(source.marketStatus) {
    | Some(value) => value
    | None => {
        Array.push(errors, "Invalid status")
        Market__Model.Status.Pending
      }
    }
    let description = switch source.description {
    | Some(value) => value
    | None => {
        Array.push(errors, "Missing description")
        ""
      }
    }
    let definition = switch source.definition {
    | Some(value) => value
    | None => {
        Array.push(errors, "Missing definition")
        ""
      }
    }
    let (iconLow, iconMedium, iconHigh) = switch Market__Model.resolveIconUrls(
      source.iconUrlLow,
      source.iconUrlMedium,
      source.iconUrlHigh,
    ) {
    | Some(icons) => icons
    | None => {
        Array.push(errors, "Missing icon URL")
        ("", "", "")
      }
    }
    let (bannerLow, bannerMedium, bannerHigh) = switch Market__Model.resolveIconUrls(
      source.bannerImageUrlLow,
      source.bannerImageUrlMedium,
      source.bannerImageUrlHigh,
    ) {
    | Some(icons) => icons
    | None => {
        Array.push(errors, "Missing banner URL")
        ("", "", "")
      }
    }

    let depositAssetPairs = sortPairsByDisplayPriority(
      deriveDepositAssetPairs(depositAssets, orderbookPairs),
    )
    if Array.length(depositAssetPairs) == 0 {
      Array.push(errors, "Missing deposit asset pairs")
    }

    if Array.length(errors) > 0 {
      Error(formatValidationErrors("Market", source.marketPubkey, errors))
    } else {
      Ok({
        id: source.marketId,
        pubkey: source.marketPubkey,
        featuredRank: ?source.featuredRank,
        slug,
        name,
        status,
        createdAt: parseTimestamp(source.createdAt),
        activatedAt: ?parseTimestampOpt(source.activatedAt),
        settledAt: ?parseTimestampOpt(source.settledAt),
        resolution: ?source.resolution,
        description,
        definition,
        tags: source.tags->Option.getOr([]),
        outcomes,
        iconUrlLow: iconLow,
        iconUrlMedium: iconMedium,
        iconUrlHigh: iconHigh,
        bannerImageUrlLow: bannerLow,
        bannerImageUrlMedium: bannerMedium,
        bannerImageUrlHigh: bannerHigh,
        category: ?source.category,
        orderbookIds: orderbookPairs->Array.map(pair => pair.orderbookId),
        orderbookPairs,
        depositAssets,
        depositAssetPairs,
        conditionalTokens,
        tokenMetadata,
      })
    }
  }
}

// Response for `GET /api/markets`.
module MarketsResponse = {
  @spice
  type t = {
    markets: array<MarketResponse.t>,
    @spice.key("next_cursor") nextCursor?: float,
    @spice.key("has_more") hasMore: bool,
  }
}

// Response for the single-market endpoints (by slug / by pubkey).
module SingleMarketResponse = {
  @spice
  type t = {market: MarketResponse.t}
}

// ── Global deposit-asset wire types + conversion ──────────────────────────────
module GlobalDepositAssetResponse = {
  @spice
  type t = {
    id: float,
    mint: Shared.pubkeyStr,
    @spice.key("display_name") displayName?: string,
    symbol?: string,
    description?: string,
    @spice.key("icon_url_low") iconUrlLow?: string,
    @spice.key("icon_url_medium") iconUrlMedium?: string,
    @spice.key("icon_url_high") iconUrlHigh?: string,
    decimals?: float,
    @spice.key("whitelist_index") whitelistIndex: float,
    active: bool,
  }

  let toGlobalDepositAsset = (source: t): result<Market__Model.GlobalDepositAsset.t, string> => {
    let errors: array<string> = []

    let name = switch source.displayName {
    | Some(value) => value
    | None => {
        Array.push(errors, `Missing display name: ${source.mint}`)
        ""
      }
    }
    let symbol = switch source.symbol {
    | Some(value) => value
    | None => {
        Array.push(errors, `Missing symbol: ${source.mint}`)
        ""
      }
    }
    let (low, medium, high) = switch Market__Model.resolveIconUrls(
      source.iconUrlLow,
      source.iconUrlMedium,
      source.iconUrlHigh,
    ) {
    | Some(icons) => icons
    | None => {
        Array.push(errors, `Missing icon URL: ${source.mint}`)
        ("", "", "")
      }
    }
    let decimals = switch source.decimals {
    | Some(value) => value
    | None => {
        Array.push(errors, `Missing decimals: ${source.mint}`)
        0.0
      }
    }

    if Array.length(errors) > 0 {
      Error(formatValidationErrors("Token", source.mint, errors))
    } else {
      Ok({
        id: source.id,
        depositAsset: source.mint,
        shortSymbol: name,
        name,
        symbol,
        description: ?source.description,
        decimals,
        iconUrlLow: low,
        iconUrlMedium: medium,
        iconUrlHigh: high,
        whitelistIndex: source.whitelistIndex,
        active: source.active,
      })
    }
  }
}

// Response for `GET /api/global-deposit-assets`.
module GlobalDepositAssetsListResponse = {
  @spice
  type t = {
    assets: array<GlobalDepositAssetResponse.t>,
    total: float,
  }
}
