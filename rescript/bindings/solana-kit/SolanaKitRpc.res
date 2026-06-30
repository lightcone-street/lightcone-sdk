// `SolanaKit.Rpc` — Solana JSON-RPC. Calls are lazy: build a request, then
// `.send()`. Responses come back as `JSON.t` and are decoded at the SDK layer.
type t
type pending<'a>

@module("@solana/kit") external make: string => t = "createSolanaRpc"
@send external send: pending<'a> => promise<'a> = "send"

@send external getLatestBlockhash: t => pending<JSON.t> = "getLatestBlockhash"
@send external getAccountInfo: (t, SolanaKit.address, {..}) => pending<JSON.t> = "getAccountInfo"
@send
external getMultipleAccounts: (t, array<SolanaKit.address>, {..}) => pending<JSON.t> = "getMultipleAccounts"
@send external sendTransaction: (t, string, {..}) => pending<string> = "sendTransaction"
