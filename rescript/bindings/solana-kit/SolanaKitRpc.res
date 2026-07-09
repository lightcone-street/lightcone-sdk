// `SolanaKit.Rpc` — Solana JSON-RPC. Calls are lazy: build a request, then
// `.send()`. Responses come back as `JSON.t` and are decoded at the SDK layer.
//
// Opaque: `t` is the raw `@solana/kit` RPC client — a foreign Proxy object (the one that
// answers truthy for every property access, which is why it must never be wrapped in an
// `option`). It has no structural meaning for a consumer; reads go through `Rpc.*`.
@genType.opaque
type t
type pending<'a>

@module("@solana/kit") external make: string => t = "createSolanaRpc"
@send external send: pending<'a> => promise<'a> = "send"

@send external getLatestBlockhash: t => pending<JSON.t> = "getLatestBlockhash"
@send external getAccountInfo: (t, SolanaKit.address, {..}) => pending<JSON.t> = "getAccountInfo"
@send
external getMultipleAccounts: (t, array<SolanaKit.address>, {..}) => pending<JSON.t> = "getMultipleAccounts"
@send external sendTransaction: (t, string, {..}) => pending<string> = "sendTransaction"
