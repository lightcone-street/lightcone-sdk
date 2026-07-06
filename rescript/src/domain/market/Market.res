// Market domain — market discovery / metadata queries, the market token
// structures, and the orderbook-pair structure that markets own. Pure namespace
// over the Market__* files.

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

let isResolved = Market__Model.isResolved
let singleWinningOutcome = Market__Model.singleWinningOutcome
let hasSingleWinningOutcome = Market__Model.hasSingleWinningOutcome
let usdcMainnet = Market__Model.usdcMainnet
let usdtMainnet = Market__Model.usdtMainnet
let usdcDevnetLc = Market__Model.usdcDevnetLc
let isUsdStablecoin = Market__Model.isUsdStablecoin
let currencySymbol = Market__Model.currencySymbol
let sortByDisplayPriority = Market__Model.sortByDisplayPriority
let resolveIconUrls = Market__Model.resolveIconUrls
