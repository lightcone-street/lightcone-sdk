# `@lightcone-sdk/solana-kit` binding tests

Runtime tests for the `@solana/kit` binding — exercise the **actual bindings** as a ReScript
consumer and run the compiled output under **Bun**.

## Run

```bash
./node_modules/.bin/rescript build
bun test ./bindings/solana-kit/tests/SolanaKitTest.res.mjs
```

The `./` prefix is required: `bun test` treats a bare path as a name filter.

## Coverage matrix

**Behaviorally tested:** `address` / `isAddress` / `addressToString`; `Codec.getU64Encoder`
(LE length) + `Codec.encode`/`Codec.decode` + `Codec.getUtf8Encoder`/`Codec.getBase16Decoder`;
`Keys.createKeyPairFromPrivateKeyBytes` + `Keys.signBytes` + `Keys.verifySignature` (ed25519
roundtrip) + `Keys.getAddressFromPublicKey`; `Pda.getProgramDerivedAddress` ((address, bump)).

**Smoke only:** none.

**Not runtime-tested (reason):**
- `Tx.*` — a full transaction needs a fee-payer signer + a recent blockhash; exercised end-to-end
  by the SDK's on-chain examples instead.
- `Rpc.*` — needs a live Solana RPC endpoint; covered by the SDK's `read_onchain` example.
- The remaining `Codec` integer/byte codecs (`getU16`/`getU32`/`getI64`/`getBytes`/`fix*`) — same
  shape as the tested `getU64Encoder`; compile-guarded via `SolanaKitReadmeChecks.res`.
