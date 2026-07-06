// Market domain — market discovery / metadata queries, the market token
// structures, and the orderbook-pair structure that markets own. Pure
// namespace: each alias re-exports its implementation module, whose `.resi` is
// the real interface.

type t = Market__Model.t
module Status = Market__Model.Status
module Resolution = Market__Model.Resolution
module Outcome = Market__Model.Outcome
module ConditionalToken = Market__Model.ConditionalToken
module DepositAsset = Market__Model.DepositAsset
module TokenMetadata = Market__Model.TokenMetadata
module DepositAssetPair = Market__Model.DepositAssetPair
module GlobalDepositAsset = Market__Model.GlobalDepositAsset
module OrderBookPair = Market__Model.OrderBookPair
module Impact = Market__Model.Impact
module MarketsResult = Market__Model.MarketsResult
module GlobalDepositAssetsResult = Market__Model.GlobalDepositAssetsResult
module Raw = Market__Raw
module Client = Market__Client

// ── Market resolution helpers ─────────────────────────────────────────────────
let isResolved: t => bool
let singleWinningOutcome: t => option<float>
let hasSingleWinningOutcome: t => bool

// ── Token helpers ─────────────────────────────────────────────────────────────
let usdcMainnet: Shared.pubkeyStr
let usdtMainnet: Shared.pubkeyStr
let usdcDevnetLc: Shared.pubkeyStr

let isUsdStablecoin: Shared.pubkeyStr => bool
// "$" for USD stablecoins, "" otherwise.
let currencySymbol: Shared.pubkeyStr => string

// A new array ordered for display: BTC/WBTC → ETH/WETH → SOL first, then the
// rest alphabetically by symbol. Pass `~symbolOf` (e.g. `token => token.symbol`,
// or `pair => pair.base.symbol` for composite pairs).
let sortByDisplayPriority: (array<'a>, ~symbolOf: 'a => string) => array<'a>

// Resolve three icon-url quality variants with cross-fallback. `None` only when
// all three inputs are absent.
let resolveIconUrls: (
  option<string>,
  option<string>,
  option<string>,
) => option<(string, string, string)>
