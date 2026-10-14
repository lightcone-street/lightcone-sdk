// Exact JSON-RPC envelope. Reads may fail over on transport errors; send callers
// explicitly select once=true and retain the response for rejection classification.
let field = (json, key) =>
  switch json {
  | JSON.Object(fields) => Dict.get(fields, key)->Option.getOr(JSON.Null)
  | _ => JSON.Null
  }
let str = json =>
  switch json {
  | JSON.String(value) => Some(value)
  | _ => None
  }
let u64 = json =>
  ProtocolInt.U64.t_decode(json)->Result.mapError(_ => SdkError.Decode("expected exact RPC u64"))
let obj = fields => JSON.Object(Dict.fromArray(fields))
let confirmed = obj([("commitment", JSON.String("confirmed"))])
let accountOptions = obj([
  ("commitment", JSON.String("confirmed")),
  ("encoding", JSON.String("base64")),
])
let call = async (client: Client.t, method, params, ~once=false): result<JSON.t, SdkError.t> => {
  let body = ExactJson.stringify(
    obj([
      ("jsonrpc", JSON.String("2.0")),
      ("id", JSON.Number(1.)),
      ("method", JSON.String(method)),
      ("params", JSON.Array(params)),
    ]),
  )
  let tryOn = async target => {
    let url = switch target {
    | RpcFailover.Active.Primary => client.rpcUrl
    | Backup => client.backupRpcUrl->Option.getOr(client.rpcUrl)
    }
    let run = async () => {
      let response = await Fetch.fetch(
        url,
        {
          method: "POST",
          headers: Dict.fromArray([("Content-Type", "application/json")]),
          body,
          credentials: "omit",
          redirect: "error",
          signal: Fetch.timeoutSignal(30000),
        },
      )
      if !Fetch.ok(response) {
        Error(SdkError.Other("RPC HTTP request failed"))
      } else {
        switch ExactJson.parse(await Fetch.text(response)) {
        | Ok(json) => Ok(json)
        | Error(_) => Error(SdkError.Decode("malformed RPC JSON"))
        }
      }
    }
    switch await run() {
    | value => value
    | exception _ => Error(SdkError.Other("RPC transport failed"))
    }
  }
  if once {
    await tryOn(RpcFailover.active(client.rpcFailover))
  } else {
    await RpcFailover.withFailover(
      client.rpcFailover,
      ~hasBackup=Option.isSome(client.backupRpcUrl),
      ~tryOn,
    )
  }
}
let result = response =>
  if field(response, "error") != JSON.Null {
    Error(SdkError.Other("RPC rejected request"))
  } else {
    switch response {
    | JSON.Object(fields) =>
      switch Dict.get(fields, "result") {
      | Some(value) => Ok(value)
      | None => Error(SdkError.Decode("missing RPC result"))
      }
    | _ => Error(SdkError.Decode("invalid RPC response"))
    }
  }
let read = async (client, method, params) =>
  (await call(client, method, params))->Result.flatMap(result)
let balance = async (client, address) =>
  (
    await read(client, "getBalance", [JSON.String(SolanaKit.addressToString(address)), confirmed])
  )->Result.flatMap(value => u64(field(value, "value")))
let rent = async (client, size) =>
  (
    await read(
      client,
      "getMinimumBalanceForRentExemption",
      [JSON.Number(Int.toFloat(size)), confirmed],
    )
  )->Result.flatMap(u64)
