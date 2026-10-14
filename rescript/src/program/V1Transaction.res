// A validated transaction owns its bytes and context. Accessors return copies;
// wallet acceptance compares the exact message and verifies every signature.
module Resources = {
  type t = {
    computeUnitLimit: int,
    loadedAccountsDataSizeLimit: int,
    priorityFeeLamports: bigint,
    heapSize?: int,
  }
  let validInteger: int => bool = %raw(`Number.isInteger`)
  let validate = (value: t): result<unit, SdkError.t> => {
    if (
      !validInteger(value.computeUnitLimit) ||
      value.computeUnitLimit < 1 ||
      value.computeUnitLimit > 1400000
    ) {
      Error(InvalidTransaction("compute limit must be 1..1400000"))
    } else if (
      !validInteger(value.loadedAccountsDataSizeLimit) ||
      value.loadedAccountsDataSizeLimit < 1 ||
      value.loadedAccountsDataSizeLimit > 67108864
    ) {
      Error(InvalidTransaction("loaded account data limit must be 1..67108864 bytes"))
    } else if value.priorityFeeLamports < 0n || value.priorityFeeLamports > 18446744073709551615n {
      Error(InvalidTransaction("priority fee must be u64 total lamports"))
    } else if (
      value.heapSize->Option.mapOr(false, heap =>
        !validInteger(heap) || heap < 32768 || heap > 262144 || mod(heap, 1024) != 0
      )
    ) {
      Error(InvalidTransaction("heap must be 32..256 KiB in 1 KiB increments"))
    } else {
      Ok()
    }
  }
}
module Context = {
  type t = {blockhash: string, lastValidBlockHeight: bigint, resources: Resources.t}
}
type t = {wire: Uint8Array.t, context: Context.t}
let copy: Uint8Array.t => Uint8Array.t = %raw(`bytes => Uint8Array.from(bytes)`)
let length: Uint8Array.t => int = %raw(`bytes => bytes.length`)
let first: Uint8Array.t => int = %raw(`bytes => bytes[0] ?? -1`)
let equalBytes: (
  Uint8Array.t,
  Uint8Array.t,
) => bool = %raw(`(a,b) => a.length === b.length && a.every((v,i) => v === b[i])`)
let rawCompare: (
  Uint8Array.t,
  Uint8Array.t,
) => int = %raw(`(a,b) => {for(let i=0;i<a.length;i++){if(a[i]!==b[i]) return a[i]-b[i];}return 0;}`)
let copyContext = (context: Context.t): Context.t => {
  blockhash: context.blockhash,
  lastValidBlockHeight: context.lastValidBlockHeight,
  resources: {
    computeUnitLimit: context.resources.computeUnitLimit,
    loadedAccountsDataSizeLimit: context.resources.loadedAccountsDataSizeLimit,
    priorityFeeLamports: context.resources.priorityFeeLamports,
    heapSize: ?context.resources.heapSize,
  },
}
let zeroBytes: int => Uint8Array.t = %raw(`n => new Uint8Array(n)`)
let configValues = (resources: Resources.t): array<SolanaKitV1.configValue> => {
  let values: array<SolanaKitV1.configValue> = []
  if resources.priorityFeeLamports != 0n {
    Array.push(values, SolanaKitV1.U64({value: resources.priorityFeeLamports}))
  }
  Array.push(values, SolanaKitV1.U32({value: resources.computeUnitLimit}))
  Array.push(values, SolanaKitV1.U32({value: resources.loadedAccountsDataSizeLimit}))
  resources.heapSize->Option.forEach(value => Array.push(values, SolanaKitV1.U32({value: value})))
  values
}
let configMask = (resources: Resources.t) =>
  (resources.priorityFeeLamports == 0n ? 0 : 3) + 12 + (resources.heapSize->Option.isSome ? 16 : 0)
let validateContext = (context: Context.t) =>
  Resources.validate(context.resources)->Result.flatMap(() => {
    if (
      context.lastValidBlockHeight <= 0n ||
      context.lastValidBlockHeight > 18446744073709551615n ||
      !SolanaKit.isAddress(context.blockhash) ||
      context.blockhash == "11111111111111111111111111111111"
    ) {
      Error(
        SdkError.InvalidTransaction(
          "recent blockhash and paired nonzero expiry height are required",
        ),
      )
    } else {
      Ok()
    }
  })
let configMatches = (message: SolanaKitV1.compiled, resources: Resources.t) => {
  if (
    message.configMask == configMask(resources) && message.configValues == configValues(resources)
  ) {
    true
  } else if resources.priorityFeeLamports == 0n && message.configMask == configMask(resources) + 3 {
    message.configValues == Array.concat([SolanaKitV1.U64({value: 0n})], configValues(resources))
  } else {
    false
  }
}
let validateMessage = (message: SolanaKitV1.compiled, context: Context.t): result<
  unit,
  SdkError.t,
> => {
  let count = Array.length(message.staticAccounts)
  let signers = message.header.numSignerAccounts
  let unique = Dict.make()
  message.staticAccounts->Array.forEach(address =>
    Dict.set(unique, SolanaKit.addressToString(address), true)
  )
  if (
    message.version != 1 ||
    count > 64 ||
    count != Dict.toArray(unique)->Array.length ||
    signers < 1 ||
    signers > 12 ||
    message.header.numReadonlySignerAccounts >= signers ||
    signers + message.header.numReadonlyNonSignerAccounts > count
  ) {
    Error(InvalidTransaction("invalid v1 address or signer counts"))
  } else if (
    message.lifetimeToken != context.blockhash || !configMatches(message, context.resources)
  ) {
    Error(InvalidTransaction("message does not match its blockhash and resource context"))
  } else if (
    message.numInstructions < 1 ||
    message.numInstructions > 64 ||
    message.numInstructions != Array.length(message.instructionHeaders) ||
    message.numInstructions != Array.length(message.instructionPayloads) ||
    message.numStaticAccounts != count
  ) {
    Error(InvalidTransaction("invalid v1 instruction counts"))
  } else if (
    message.instructionHeaders->Array.some(header =>
      header.programAccountIndex <= 0 ||
      header.programAccountIndex >= count ||
      message.staticAccounts
      ->Array.get(header.programAccountIndex)
      ->Option.mapOr(false, address =>
        SolanaKit.addressToString(address) == "ComputeBudget111111111111111111111111111111"
      )
    ) ||
      message.instructionPayloads->Array.some(payload =>
        payload.instructionAccountIndices->Array.some(index => index < 0 || index >= count)
      )
  ) {
    Error(InvalidTransaction("invalid instruction index or unsupported ComputeBudget instruction"))
  } else {
    Ok()
  }
}
let fromWireBytes = (wire, context): result<t, SdkError.t> => {
  switch validateContext(context) {
  | Error(error) => Error(error)
  | Ok() =>
    if first(wire) != 0x81 || length(wire) > 4096 {
      Error(InvalidTransaction("only Solana v1 transactions of at most 4096 bytes are supported"))
    } else {
      let run = () => {
        let wire = copy(wire)
        let decoded = SolanaKitCodec.decode(SolanaKitV1.transactionDecoder(), wire)
        let message = SolanaKitCodec.decode(SolanaKitV1.messageDecoder(), decoded.messageBytes)
        validateMessage(message, context)->Result.flatMap(() => {
          if (
            !equalBytes(
              SolanaKitCodec.encode(SolanaKitV1.messageEncoder(), message),
              decoded.messageBytes,
            ) ||
            !equalBytes(SolanaKitCodec.encode(SolanaKitV1.transactionEncoder(), decoded), wire)
          ) {
            Error(SdkError.InvalidTransaction("noncanonical or trailing transaction bytes"))
          } else {
            Ok({wire, context: copyContext(context)})
          }
        })
      }
      switch run() {
      | result => result
      | exception _ => Error(InvalidTransaction("invalid v1 transaction encoding"))
      }
    }
  }
}
type account = {address: SolanaKit.address, mutable signer: bool, mutable writable: bool}
let compile = (
  ~instructions: array<SolanaKit.instruction>,
  ~payer: SolanaKit.address,
  ~context: Context.t,
): result<t, SdkError.t> => {
  let run = () => {
    let accounts: dict<account> = Dict.make()
    let merge = (address, signer, writable) => {
      let key = SolanaKit.addressToString(address)
      switch Dict.get(accounts, key) {
      | None => Dict.set(accounts, key, {address, signer, writable})
      | Some(old) =>
        old.signer = old.signer || signer
        old.writable = old.writable || writable
      }
    }
    merge(payer, true, true)
    instructions->Array.forEach(ix => {
      merge(ix.programAddress, false, false)
      ix.accounts->Array.forEach(meta =>
        merge(meta.address, meta.role >= 2, mod(meta.role, 2) == 1)
      )
    })
    let group = a => a.address == payer ? 0 : a.signer ? a.writable ? 1 : 2 : a.writable ? 3 : 4
    let ordered = Dict.valuesToArray(accounts)->Array.toSorted((a, b) => {
      let rank = group(a) - group(b)
      rank != 0
        ? Int.toFloat(rank)
        : Int.toFloat(
            rawCompare(
              SolanaKitCodec.encode(SolanaKitCodec.getAddressEncoder(), a.address),
              SolanaKitCodec.encode(SolanaKitCodec.getAddressEncoder(), b.address),
            ),
          )
    })
    let index = address => ordered->Array.findIndex(a => a.address == address)
    let count = predicate => ordered->Array.filter(predicate)->Array.length
    let header: SolanaKitV1.header = {
      numSignerAccounts: count(a => a.signer),
      numReadonlySignerAccounts: count(a => a.signer && !a.writable),
      numReadonlyNonSignerAccounts: count(a => !a.signer && !a.writable),
    }
    let message: SolanaKitV1.compiled = {
      version: 1,
      header,
      lifetimeToken: context.blockhash,
      configMask: configMask(context.resources),
      configValues: configValues(context.resources),
      numInstructions: Array.length(instructions),
      numStaticAccounts: Array.length(ordered),
      staticAccounts: ordered->Array.map(a => a.address),
      instructionHeaders: instructions->Array.map(ix => {
        SolanaKitV1.programAccountIndex: index(ix.programAddress),
        numInstructionAccounts: Array.length(ix.accounts),
        numInstructionDataBytes: length(ix.data),
      }),
      instructionPayloads: instructions->Array.map(ix => {
        SolanaKitV1.instructionAccountIndices: ix.accounts->Array.map(meta => index(meta.address)),
        instructionData: ix.data,
      }),
    }
    validateContext(context)->Result.flatMap(() =>
      validateMessage(message, context)->Result.flatMap(() => {
        if (
          instructions->Array.some(
            ix =>
              Array.length(ix.accounts) > 255 ||
              length(ix.data) > 65535 ||
              ix.accounts->Array.some(meta => meta.role < 0 || meta.role > 3),
          )
        ) {
          Error(SdkError.InvalidTransaction("instruction exceeds v1 bounds"))
        } else {
          let bytes = SolanaKitCodec.encode(SolanaKitV1.messageEncoder(), message)
          fromWireBytes(
            OrderPayload.concatBytes([bytes, zeroBytes(64 * header.numSignerAccounts)]),
            context,
          )
        }
      })
    )
  }
  switch run() {
  | result => result
  | exception _ => Error(InvalidTransaction("failed to compile canonical v1 transaction"))
  }
}
let context = tx => copyContext(tx.context)
let toWireBytes = tx => copy(tx.wire)
let decoded = tx => SolanaKitCodec.decode(SolanaKitV1.transactionDecoder(), tx.wire)
let messageBytes = tx => copy(decoded(tx).messageBytes)
let compiled = tx => SolanaKitCodec.decode(SolanaKitV1.messageDecoder(), messageBytes(tx))
let requiredSigners = tx => {
  let m = compiled(tx)
  Array.slice(m.staticAccounts, ~start=0, ~end=m.header.numSignerAccounts)
}
let feePayer = tx => requiredSigners(tx)->Array.get(0)
let verifySignatures = async tx => {
  let transaction = decoded(tx)
  let valid = await Promise.all(
    requiredSigners(tx)->Array.map(async address => {
      switch Dict.get(transaction.signatures, SolanaKit.addressToString(address))->Option.flatMap(
        Null.toOption,
      ) {
      | None => false
      | Some(signature) =>
        let bytes = SolanaKitCodec.encode(SolanaKitCodec.getAddressEncoder(), address)
        await StrictEd25519.verify(signature, transaction.messageBytes, bytes)
      }
    }),
  )
  valid->Array.every(value => value)
    ? Ok()
    : Error(SdkError.Signing("missing or invalid required transaction signature"))
}
let acceptSignedBytes = async (tx, wire) =>
  switch fromWireBytes(wire, tx.context) {
  | Error(error) => Error(error)
  | Ok(signed) =>
    if !equalBytes(messageBytes(tx), messageBytes(signed)) {
      Error(SdkError.Signing("wallet changed the transaction message"))
    } else {
      (await verifySignatures(signed))->Result.map(() => signed)
    }
  }
let sign = async (tx, keypairs) => {
  let run = async () => {
    let pairs = await Promise.all(
      keypairs->Array.map(async keypair => (
        await SolanaKitKeys.getAddressFromPublicKey(keypair.SolanaKit.publicKey),
        keypair,
      )),
    )
    let signatures = await Promise.all(
      requiredSigners(tx)->Array.map(async address =>
        switch pairs->Array.find(((key, _)) => key == address) {
        | None => Error(SdkError.Signing("missing required signer"))
        | Some((_, keypair)) =>
          Ok(await SolanaKitKeys.signBytes(keypair.privateKey, messageBytes(tx)))
        }
      ),
    )
    switch signatures->Array.findMap(result =>
      switch result {
      | Error(error) => Some(error)
      | _ => None
      }
    ) {
    | Some(error) => Error(error)
    | None =>
      await acceptSignedBytes(
        tx,
        OrderPayload.concatBytes(
          Array.concat(
            [messageBytes(tx)],
            signatures->Array.filterMap(result => result->Result.mapOr(None, value => Some(value))),
          ),
        ),
      )
    }
  }
  switch await run() {
  | result => result
  | exception _ => Error(SdkError.Signing("transaction signing failed"))
  }
}
let signature = async tx =>
  (await verifySignatures(tx))->Result.flatMap(() =>
    switch feePayer(tx) {
    | None => Error(SdkError.Signing("missing fee payer"))
    | Some(payer) =>
      switch Dict.get(decoded(tx).signatures, SolanaKit.addressToString(payer))->Option.flatMap(
        Null.toOption,
      ) {
      | None => Error(SdkError.Signing("missing fee payer signature"))
      | Some(bytes) => Ok(SolanaKitCodec.decode(SolanaKitCodec.getBase58Decoder(), bytes))
      }
    }
  )
