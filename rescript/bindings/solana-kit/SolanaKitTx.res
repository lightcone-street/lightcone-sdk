// `SolanaKit.Tx` — transaction message build / sign / serialize. Pipe-style
// builders: each takes (config, message) and returns a new message.
type message
type signedTransaction
type blockhashLifetime

@module("@solana/kit") external create: {"version": int} => message = "createTransactionMessage"
@module("@solana/kit")
external setFeePayerSigner: (SolanaKit.keyPairSigner, message) => message =
  "setTransactionMessageFeePayerSigner"
// Address-only fee payer (no signer attached) — for unsigned messages the caller
// signs themselves.
@module("@solana/kit")
external setFeePayer: (SolanaKit.address, message) => message = "setTransactionMessageFeePayer"
@module("@solana/kit")
external appendInstruction: (SolanaKit.instruction, message) => message =
  "appendTransactionMessageInstruction"
@module("@solana/kit")
external setLifetimeUsingBlockhash: (blockhashLifetime, message) => message =
  "setTransactionMessageLifetimeUsingBlockhash"
@module("@solana/kit")
external signWithSigners: message => promise<signedTransaction> = "signTransactionMessageWithSigners"
// Compile a message into an (unsigned) transaction — signatures are zero
// placeholders until a signer fills them.
@module("@solana/kit")
external compile: message => signedTransaction = "compileTransaction"
// Sign a compiled transaction with raw WebCrypto keypairs.
@module("@solana/kit")
external signWithKeyPairs: (array<SolanaKit.cryptoKeyPair>, signedTransaction) => promise<signedTransaction> =
  "signTransaction"
@module("@solana/kit")
external base64Wire: signedTransaction => string = "getBase64EncodedWireTransaction"
@module("@solana/kit")
external getSignature: signedTransaction => string = "getSignatureFromTransaction"
