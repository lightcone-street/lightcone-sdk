// Claim testnet SOL + whitelisted deposit tokens for the wallet. Mirrors
// rust/examples/faucet_claim.rs. Testnet-only: succeeds only against environments
// whose backend has the faucet enabled (typically local / staging). This is a
// plain backend call (POST /api/claim), not an on-chain transaction — the backend
// performs the mint and returns the signature. ReScript surface (result core);
// the compiled .res.mjs is the JS example. See FaucetClaim.ts for the TS port.
let main = async () => {
  let client = Common__Example.client()

  // The faucet needs only the wallet's base58 address (no signing).
  let keypair = await SolanaKitKeys.createKeyPairFromBytes(Common__Example.walletSecretKey())
  let wallet = await SolanaKitKeys.getAddressFromPublicKey(keypair.publicKey)
  let walletAddress = SolanaKit.addressToString(wallet)

  switch await Faucet.Client.claim(client, ~walletAddress) {
  | Ok(result) => {
      Console.log(`claim tx: ${result.signature}`)
      Console.log(`sol: ${Float.toString(result.sol)}`)
      result.tokens->Array.forEach(token => Console.log(`  - ${token.symbol}: ${Float.toString(token.amount)}`))
    }
  | Error(error) => Console.error(SdkError.toMessage(error))
  }
}

let _ = main()
