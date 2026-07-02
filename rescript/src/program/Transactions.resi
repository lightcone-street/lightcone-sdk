// Transactions — sign + broadcast v0 transactions with the client's signing
// strategy (Rust `LightconeClient::sign_and_submit_tx`). Native signers use the
// kit signer flow; external signers receive the unsigned wire bytes and return
// the signed wire bytes. The blockhash lifetime is fetched automatically. The
// lifetime projector and broadcast helpers are internal.

// Sign + broadcast a single-instruction v0 transaction (the strategy's address
// is the fee payer). Returns the transaction signature.
let signAndSubmit: (
  Client.t,
  unit => promise<SolanaKit.instruction>,
) => promise<result<string, SdkError.t>>

// Sign + broadcast a caller-assembled message (e.g. `PositionBuilders.unsignedTx`
// with further instructions appended). The fee payer must already be set and
// match the configured strategy's address.
let signAndSubmitMessage: (Client.t, SolanaKitTx.message) => promise<result<string, SdkError.t>>
