// Transactions — sign + broadcast v0 transactions with the client's signing
// strategy. The blockhash lifetime is fetched automatically; callers never set it.
//
// - **NativeSigner**: kit signer flow (fee-payer signer embedded in the message,
//   `signTransactionMessageWithSigners`), or raw-keypair signing for a
//   caller-assembled message.
// - **ExternalSigner**: the message compiles to an unsigned wire transaction;
//   the wallet adapter's `signTransaction` returns the signed wire bytes, which
//   broadcast as base64.

// kit's `getLatestBlockhash().send()` resolves to {context, value: {blockhash,
// lastValidBlockHeight}}. `setTransactionMessageLifetimeUsingBlockhash` wants the
// inner `value` object verbatim, so we project it out. (The RPC layer types the
// response loosely as JSON.t; the runtime shape is exactly the lifetime object.)
let blockhashLifetime: JSON.t => SolanaKitTx.blockhashLifetime = %raw(`function (response) {
  return response.value;
}`)

let fetchLifetime = async (client: Client.t): SolanaKitTx.blockhashLifetime => {
  let response = await SolanaKitRpc.getLatestBlockhash(client.rpc)->SolanaKitRpc.send
  blockhashLifetime(response)
}

let broadcastBase64 = (client: Client.t, wire: string): promise<string> =>
  SolanaKitRpc.sendTransaction(client.rpc, wire, {"encoding": "base64"})->SolanaKitRpc.send

// Sign a compiled-but-unsigned transaction through the external wallet adapter
// and broadcast the returned wire bytes.
let externalSignAndBroadcast = async (
  client: Client.t,
  external_: Client.ExternalSigner.t,
  unsigned: SolanaKitTx.signedTransaction,
): string => {
  let unsignedWire = SolanaKitTx.base64Wire(unsigned)
  let wireBytes = SolanaKitCodec.encode(SolanaKitCodec.getBase64Encoder(), unsignedWire)
  let signedBytes = await external_.signTransaction(wireBytes)
  let signedWire = SolanaKitCodec.decode(SolanaKitCodec.getBase64Decoder(), signedBytes)
  await broadcastBase64(client, signedWire)
}

// Sign + broadcast a single-instruction v0 transaction with the client's signing
// strategy as fee payer. `makeInstruction` is a thunk so any throwing PDA/ATA
// derivation is captured under the same error boundary as the RPC/sign calls.
let signAndSubmit = async (
  client: Client.t,
  makeInstruction: unit => promise<SolanaKit.instruction>,
): result<string, SdkError.t> =>
  switch client.signingStrategy {
  | None =>
    Error(
      SdkError.Signing(
        "no signing strategy configured on the client; call Client.useNativeSigner or Client.useExternalSigner first",
      ),
    )
  | Some(Client.SigningStrategy.NativeSigner({signer})) =>
    let run = async () => {
      let instruction = await makeInstruction()
      let lifetime = await fetchLifetime(client)
      let message =
        SolanaKitTx.create({"version": 0})
        ->SolanaKitTx.setFeePayerSigner(signer, _)
        ->SolanaKitTx.appendInstruction(instruction, _)
        ->SolanaKitTx.setLifetimeUsingBlockhash(lifetime, _)
      let signedTransaction = await SolanaKitTx.signWithSigners(message)
      await broadcastBase64(client, SolanaKitTx.base64Wire(signedTransaction))
    }
    switch await run() {
    | signature => Ok(signature)
    | exception JsExn(error) =>
      Error(SdkError.Other(error->JsExn.message->Option.getOr("failed to sign and send transaction")))
    }
  | Some(Client.SigningStrategy.ExternalSigner(external_)) =>
    let run = async () => {
      let instruction = await makeInstruction()
      let lifetime = await fetchLifetime(client)
      let unsigned =
        SolanaKitTx.create({"version": 0})
        ->SolanaKitTx.setFeePayer(external_.address, _)
        ->SolanaKitTx.appendInstruction(instruction, _)
        ->SolanaKitTx.setLifetimeUsingBlockhash(lifetime, _)
        ->SolanaKitTx.compile
      await externalSignAndBroadcast(client, external_, unsigned)
    }
    switch await run() {
    | signature => Ok(signature)
    | exception JsExn(error) =>
      Error(SdkError.Signing(error->JsExn.message->Option.getOr("external signer failed to sign transaction")))
    }
  }

// Sign + broadcast a caller-assembled message (e.g. `Position.Builders.unsignedTx`
// with further instructions appended). The fee payer must already be set and
// match the configured strategy's address; the blockhash lifetime is added here.
let signAndSubmitMessage = async (
  client: Client.t,
  message: SolanaKitTx.message,
): result<string, SdkError.t> =>
  switch client.signingStrategy {
  | None =>
    Error(
      SdkError.Signing(
        "no signing strategy configured on the client; call Client.useNativeSigner or Client.useExternalSigner first",
      ),
    )
  | Some(Client.SigningStrategy.NativeSigner({keypair})) =>
    let run = async () => {
      let lifetime = await fetchLifetime(client)
      let unsigned = message->SolanaKitTx.setLifetimeUsingBlockhash(lifetime, _)->SolanaKitTx.compile
      let signedTransaction = await SolanaKitTx.signWithKeyPairs([keypair], unsigned)
      await broadcastBase64(client, SolanaKitTx.base64Wire(signedTransaction))
    }
    switch await run() {
    | signature => Ok(signature)
    | exception JsExn(error) =>
      Error(SdkError.Other(error->JsExn.message->Option.getOr("failed to sign and send transaction")))
    }
  | Some(Client.SigningStrategy.ExternalSigner(external_)) =>
    let run = async () => {
      let lifetime = await fetchLifetime(client)
      let unsigned = message->SolanaKitTx.setLifetimeUsingBlockhash(lifetime, _)->SolanaKitTx.compile
      await externalSignAndBroadcast(client, external_, unsigned)
    }
    switch await run() {
    | signature => Ok(signature)
    | exception JsExn(error) =>
      Error(SdkError.Signing(error->JsExn.message->Option.getOr("external signer failed to sign transaction")))
    }
  }
