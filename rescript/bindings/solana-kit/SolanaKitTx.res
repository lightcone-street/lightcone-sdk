// `SolanaKit.Tx` — transaction message build / sign / serialize. Pipe-style
// builders: each takes (config, message) and returns a new message.
type message
type signedTransaction
type blockhashLifetime

@module("@solana/kit") external create: {"version": int} => message = "createTransactionMessage"
@module("@solana/kit")
external setFeePayerSigner: (SolanaKit.keyPairSigner, message) => message =
  "setTransactionMessageFeePayerSigner"
@module("@solana/kit")
external appendInstruction: (SolanaKit.instruction, message) => message =
  "appendTransactionMessageInstruction"
@module("@solana/kit")
external setLifetimeUsingBlockhash: (blockhashLifetime, message) => message =
  "setTransactionMessageLifetimeUsingBlockhash"
@module("@solana/kit")
external signWithSigners: message => promise<signedTransaction> = "signTransactionMessageWithSigners"
@module("@solana/kit")
external base64Wire: signedTransaction => string = "getBase64EncodedWireTransaction"
@module("@solana/kit")
external getSignature: signedTransaction => string = "getSignatureFromTransaction"
