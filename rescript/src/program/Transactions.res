// Prepared v1 messages are never rebuilt after fee estimation or wallet signing.
module Simulation = {
  type t = {
    slot: bigint,
    unitsConsumed: option<bigint>,
    loadedAccountsDataSize: option<bigint>,
    logs: array<string>,
  }
}
module Confirmed = {
  type t = {signature: string, slot: bigint}
}
let field = RpcJson.field
let base64 = bytes => SolanaKitCodec.decode(SolanaKitCodec.getBase64Decoder(), bytes)
let bind = async (value, next) =>
  switch await value {
  | Error(error) => Error(error)
  | Ok(value) => await next(value)
  }
let contextWithResources = async (client, resources) =>
  switch V1Transaction.Resources.validate(resources) {
  | Error(error) => Error(error)
  | Ok() =>
    (
      await RpcJson.read(client, "getLatestBlockhash", [RpcJson.confirmed])
    )->Result.flatMap(result => {
      let value = field(result, "value")
      switch (
        RpcJson.str(field(value, "blockhash")),
        RpcJson.u64(field(value, "lastValidBlockHeight")),
      ) {
      | (Some(blockhash), Ok(lastValidBlockHeight)) =>
        Ok({V1Transaction.Context.blockhash, lastValidBlockHeight, resources})
      | _ => Error(SdkError.Decode("missing blockhash lifetime"))
      }
    })
  }
let context = async (client: Client.t) =>
  switch client.transactionResources {
  | None => Error(SdkError.Validation("explicit v1 transaction resources are required"))
  | Some(resources) => await contextWithResources(client, resources)
  }
let build = async (client, ~payer, ~instructions) =>
  (await context(client))->Result.flatMap(context =>
    V1Transaction.compile(~instructions, ~payer, ~context)
  )
let featureActive: (
  Uint8Array.t,
  bigint,
) => bool = %raw(`(bytes,slot) => bytes.length >= 9 && bytes[0] === 1 && new DataView(bytes.buffer,bytes.byteOffset,bytes.byteLength).getBigUint64(1,true) <= slot`)
let ensureV1Supported = async client =>
  (
    await RpcJson.read(
      client,
      "getAccountInfo",
      [JSON.String("txv1aq4pp281K9um3tnPgkfX8UqtFT6wcVW3hNezGLL"), RpcJson.accountOptions],
    )
  )->Result.flatMap(result => {
    let account = field(result, "value")
    let valid = switch (
      RpcJson.u64(field(field(result, "context"), "slot")),
      field(account, "data"),
    ) {
    | (Ok(slot), JSON.Array([JSON.String(data), JSON.String("base64")])) =>
      switch featureActive(SolanaKitCodec.encode(SolanaKitCodec.getBase64Encoder(), data), slot) {
      | value => value
      | exception _ => false
      }
    | _ => false
    }
    if (
      valid &&
      field(account, "owner") == JSON.String("Feature111111111111111111111111111111111111") &&
      field(account, "executable") == JSON.Boolean(false)
    ) {
      Ok()
    } else {
      Error(SdkError.Validation("Solana v1 feature is missing, malformed, or inactive"))
    }
  })
let estimateFee = async (client, tx) =>
  (
    await RpcJson.read(
      client,
      "getFeeForMessage",
      [JSON.String(base64(V1Transaction.messageBytes(tx))), RpcJson.confirmed],
    )
  )->Result.flatMap(result => RpcJson.u64(field(result, "value")))
let simulate = async (client, tx) =>
  await bind(V1Transaction.verifySignatures(tx), async () =>
    await bind(ensureV1Supported(client), async () => {
      let options = RpcJson.obj([
        ("encoding", JSON.String("base64")),
        ("commitment", JSON.String("confirmed")),
        ("sigVerify", JSON.Boolean(true)),
        ("replaceRecentBlockhash", JSON.Boolean(false)),
      ])

      (
        await RpcJson.read(
          client,
          "simulateTransaction",
          [JSON.String(base64(V1Transaction.toWireBytes(tx))), options],
        )
      )->Result.flatMap(
        result => {
          let value = field(result, "value")
          let hasStatus = switch value {
          | JSON.Object(fields) => Dict.get(fields, "err")->Option.isSome
          | _ => false
          }
          if !hasStatus || field(value, "err") != JSON.Null {
            Error(SdkError.Other("v1 simulation failed or omitted its status"))
          } else {
            RpcJson.u64(field(field(result, "context"), "slot"))->Result.map(
              slot => {
                let optional = key =>
                  RpcJson.u64(field(value, key))->Result.mapOr(None, value => Some(value))
                {
                  Simulation.slot,
                  unitsConsumed: optional("unitsConsumed"),
                  loadedAccountsDataSize: optional("loadedAccountsDataSize"),
                  logs: switch field(value, "logs") {
                  | JSON.Array(logs) => logs->Array.filterMap(RpcJson.str)
                  | _ => []
                  },
                }
              },
            )
          }
        },
      )
    })
  )
let rejection = (response, signature) => {
  let error = field(response, "error")
  let reason = RpcJson.str(field(error, "message"))
  let code = switch field(error, "code") {
  | JSON.Number(value) => Some(Float.toInt(value))
  | _ => None
  }
  let hasResult = switch response {
  | JSON.Object(fields) => Dict.get(fields, "result")->Option.isSome
  | _ => false
  }
  switch (code, reason) {
  | (Some(code), Some(reason)) =>
    let lower = String.toLowerCase(reason)
    if (
      !hasResult &&
      field(field(error, "data"), "err") != JSON.String("AlreadyProcessed") &&
      !String.includes(lower, "alreadyprocessed") &&
      !String.includes(lower, "already processed") &&
      !String.includes(lower, "already been processed") &&
      [
        -32700,
        -32600,
        -32601,
        -32602,
        -32002,
        -32003,
        -32005,
        -32006,
        -32013,
        -32015,
        -32016,
      ]->Array.includes(code)
    ) {
      Some(SdkError.SubmissionRejected({signature, code, reason}))
    } else {
      None
    }
  | _ => None
  }
}
let submitSigned = async (client, tx) =>
  await bind(V1Transaction.signature(tx), async signature =>
    await bind(simulate(client, tx), async _ => {
      let unknown = reason => Error(
        SdkError.SubmissionUnknown({
          signature,
          lastValidBlockHeight: V1Transaction.context(tx).lastValidBlockHeight,
          reason,
        }),
      )
      let options = RpcJson.obj([
        ("encoding", JSON.String("base64")),
        ("skipPreflight", JSON.Boolean(false)),
        ("preflightCommitment", JSON.String("confirmed")),
        ("maxRetries", JSON.Number(0.)),
      ])
      switch await RpcJson.call(
        client,
        "sendTransaction",
        [JSON.String(base64(V1Transaction.toWireBytes(tx))), options],
        ~once=true,
      ) {
      | Error(_) => unknown("transport failed during the single submission attempt")
      | Ok(response) =>
        switch rejection(response, signature) {
        | Some(error) => Error(error)
        | None =>
          if (
            field(response, "error") == JSON.Null &&
              field(response, "result") == JSON.String(signature)
          ) {
            Ok(signature)
          } else {
            unknown("RPC did not acknowledge the expected signature")
          }
        }
      }
    })
  )
let strategyAddress = strategy =>
  switch strategy {
  | Client.SigningStrategy.NativeSigner({address}) | ExternalSigner({address}) => address
  }
let validateSigner = (tx, strategy, sponsored) => {
  let native = switch strategy {
  | Client.SigningStrategy.NativeSigner(_) => true
  | _ => false
  }
  if sponsored && native {
    Error(SdkError.Validation("native transaction sponsorship is unsupported"))
  } else if !sponsored && V1Transaction.feePayer(tx) != Some(strategyAddress(strategy)) {
    Error(SdkError.Signing("signer does not control the transaction fee payer"))
  } else if !(V1Transaction.requiredSigners(tx)->Array.includes(strategyAddress(strategy))) {
    Error(SdkError.Signing("configured wallet is not a required transaction signer"))
  } else {
    Ok()
  }
}
let preflight = async (client, tx, strategy, sponsored) =>
  switch validateSigner(tx, strategy, sponsored) {
  | Error(error) => Error(error)
  | Ok() =>
    if sponsored {
      Ok()
    } else {
      switch await estimateFee(client, tx) {
      | Error(_) => Ok()
      | Ok(requiredLamports) =>
        switch await RpcJson.balance(client, strategyAddress(strategy)) {
        | Ok(availableLamports) if availableLamports < requiredLamports =>
          Error(SdkError.InsufficientSolForTransactionFees({availableLamports, requiredLamports}))
        | _ => Ok()
        }
      }
    }
  }
let signWithSnapshot = async (client: Client.t, tx, strategy, sponsored, additionalSigners) => {
  switch strategy {
  | None => Error(SdkError.Signing("no signing strategy configured"))
  | Some(strategy) =>
    await bind(preflight(client, tx, strategy, sponsored), async () =>
      await bind(ensureV1Supported(client), async () => {
        let run = async () =>
          switch strategy {
          | NativeSigner({keypair}) =>
            await V1Transaction.sign(tx, Array.concat([keypair], additionalSigners))
          | ExternalSigner(wallet) =>
            if Array.length(additionalSigners) != 0 {
              Error(
                SdkError.Signing(
                  "external partial signing requires a prepared partial transaction",
                ),
              )
            } else {
              await V1Transaction.acceptSignedBytes(
                tx,
                await wallet.signTransaction(V1Transaction.toWireBytes(tx)),
              )
            }
          }
        switch await run() {
        | value => value
        | exception _ => Error(SdkError.Signing("wallet failed to sign v1 transaction"))
        }
      })
    )
  }
}
let signPrepared = (client: Client.t, tx, ~additionalSigners=[]) =>
  signWithSnapshot(client, tx, client.signingStrategy, client.sponsorshipEnabled, additionalSigners)
let signAndSubmitPrepared = async (client, tx) =>
  await bind(signPrepared(client, tx), signed => submitSigned(client, signed))
module Status = {
  type t = {slot: bigint, error: JSON.t, confirmed: bool}
}
let status = async (client, signature, history) => {
  let options = RpcJson.obj([("searchTransactionHistory", JSON.Boolean(history))])

  (
    await RpcJson.read(
      client,
      "getSignatureStatuses",
      [JSON.Array([JSON.String(signature)]), options],
    )
  )->Result.flatMap(result =>
    switch field(result, "value") {
    | JSON.Array([JSON.Null]) => Ok(None)
    | JSON.Array([value]) =>
      RpcJson.u64(field(value, "slot"))->Result.map(slot => Some({
        Status.slot,
        error: field(value, "err"),
        confirmed: [JSON.String("confirmed"), JSON.String("finalized")]->Array.includes(
          field(value, "confirmationStatus"),
        ),
      }))
    | _ => Error(SdkError.Decode("malformed signature status"))
    }
  )
}
let sleep: int => promise<unit> = %raw(`ms => new Promise(resolve => setTimeout(resolve, ms))`)
let confirm = async (
  client,
  signature,
  ~lastValidBlockHeight=?,
  ~pollIntervalMs=800,
  ~maxPolls=110,
): result<Confirmed.t, SdkError.t> => {
  let terminal = (status: Status.t) =>
    if status.error == JSON.Null {
      Ok({Confirmed.signature, slot: status.slot})
    } else {
      Error(SdkError.TransactionFailed({signature, error: ExactJson.stringify(status.error)}))
    }
  let rec poll = async (attempt, failures, over) => {
    if attempt >= maxPolls || failures >= 3 {
      Error(SdkError.ConfirmationTimeout({signature: signature}))
    } else {
      let next = async (failures, over) => {
        await sleep(pollIntervalMs)
        await poll(attempt + 1, failures, over)
      }
      switch await status(client, signature, false) {
      | Error(_) => await next(failures + 1, 0)
      | Ok(Some(value)) =>
        if value.confirmed {
          terminal(value)
        } else {
          await next(0, 0)
        }
      | Ok(None) =>
        switch lastValidBlockHeight {
        | None => await next(0, 0)
        | Some(bound) =>
          let height =
            (await RpcJson.read(client, "getBlockHeight", [RpcJson.confirmed]))->Result.flatMap(
              RpcJson.u64,
            )
          let over = switch height {
          | Ok(height) if height > bound => over + 1
          | _ => 0
          }
          if over < 2 {
            await next(0, over)
          } else {
            switch await status(client, signature, true) {
            | Ok(None) => Error(SdkError.TransactionExpired({signature: signature}))
            | Ok(Some(value)) =>
              if value.confirmed {
                terminal(value)
              } else {
                await next(0, 0)
              }
            | Error(_) => await next(0, over)
            }
          }
        }
      }
    }
  }
  await poll(0, 0, 0)
}
let signAndSubmitConfirmed = async (client, tx) =>
  await bind(signAndSubmitPrepared(client, tx), signature =>
    confirm(client, signature, ~lastValidBlockHeight=V1Transaction.context(tx).lastValidBlockHeight)
  )
let signAndSubmit = async (client, makeInstruction) => {
  let strategy = client.Client.signingStrategy
  let sponsored = client.sponsorshipEnabled
  let signer = Client.signerAddress(client)
  switch signer {
  | None => Error(SdkError.Signing("no signing strategy configured"))
  | Some(payer) =>
    let run = async () =>
      await bind(build(client, ~payer, ~instructions=[await makeInstruction()]), async tx =>
        await bind(signWithSnapshot(client, tx, strategy, sponsored, []), signed =>
          submitSigned(client, signed)
        )
      )
    switch await run() {
    | value => value
    | exception _ => Error(SdkError.Validation("instruction construction failed"))
    }
  }
}
let signAndSubmitMessage = signAndSubmitPrepared
