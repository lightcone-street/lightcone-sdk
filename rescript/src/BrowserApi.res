// Browser-first package entry. Server credentials and signed-message login live in /server.
@genType let make = TypeScriptApi.make
@genType let makeForEnv = TypeScriptApi.makeForEnv
@genType let useExternalSigner = TypeScriptApi.useExternalSigner
@genType let clearSigningStrategy = TypeScriptApi.clearSigningStrategy
@genType let clearOrderNonce = TypeScriptApi.clearOrderNonce
@genType let signerAddress = TypeScriptApi.signerAddress
@genType let setOrderNonce = TypeScriptApi.setOrderNonce
@genType let unwrap = TypeScriptApi.unwrap
@genType
module MarketClient = {
  include TypeScriptApi.MarketClient
}
@genType
module OrderbookClient = {
  include TypeScriptApi.OrderbookClient
}
@genType
module TradeClient = {
  include TypeScriptApi.TradeClient
}
@genType
module OrderClient = {
  include TypeScriptApi.OrderClient
}
@genType
module PositionClient = {
  include TypeScriptApi.PositionClient
}
@genType
module MetricsClient = {
  include TypeScriptApi.MetricsClient
}
@genType
module PriceHistoryClient = {
  include TypeScriptApi.PriceHistoryClient
}
@genType
module NotificationClient = {
  include TypeScriptApi.NotificationClient
}
@genType
module ReferralClient = {
  include TypeScriptApi.ReferralClient
}
@genType
module FaucetClient = {
  include TypeScriptApi.FaucetClient
}
@genType
module RpcClient = {
  include TypeScriptApi.RpcClient
}
@genType
module WsClient = {
  include TypeScriptApi.WsClient
}
@genType
module LiveOrderbook = {
  include TypeScriptApi.LiveOrderbook
}
@genType
module LivePriceHistory = {
  include TypeScriptApi.LivePriceHistory
}
@genType
module LiveDepositPrice = {
  include TypeScriptApi.LiveDepositPrice
}
@genType
module LiveOpenLimitOrders = {
  include TypeScriptApi.LiveOpenLimitOrders
}
@genType
module LiveTrades = {
  include TypeScriptApi.LiveTrades
}
@genType
module LiveUserBalances = {
  include TypeScriptApi.LiveUserBalances
}
@genType
module LiveWalletBalances = {
  include TypeScriptApi.LiveWalletBalances
}
@genType
module TransactionClient = {
  include TypeScriptApi.TransactionClient
}
@genType
module SolClient = {
  include TypeScriptApi.SolClient
}
@genType
module AuthClient = {
  let getNonce = TypeScriptApi.AuthClient.getNonce
  let checkSession = TypeScriptApi.AuthClient.checkSession
  let logout = TypeScriptApi.AuthClient.logout
  let isAuthenticated = TypeScriptApi.AuthClient.isAuthenticated
  let registerPrivy = TypeScriptApi.AuthClient.registerPrivy
  let disconnectX = TypeScriptApi.AuthClient.disconnectX
  let connectXUrl = TypeScriptApi.AuthClient.connectXUrl
  let credentials = TypeScriptApi.AuthClient.credentials
  let updateMaxSlippagePreference = TypeScriptApi.AuthClient.updateMaxSlippagePreference
  let setCredentialRestorer = TypeScriptApi.AuthClient.setCredentialRestorer
  let clearAuth = TypeScriptApi.AuthClient.clearAuth
  let tradingWallet = TypeScriptApi.AuthClient.tradingWallet
  let displayName = TypeScriptApi.AuthClient.displayName
  let walletDisplayName = TypeScriptApi.AuthClient.walletDisplayName
  let avatarUrl = TypeScriptApi.AuthClient.avatarUrl
  let classifyRegisterPrivyConflict = TypeScriptApi.AuthClient.classifyRegisterPrivyConflict
}
