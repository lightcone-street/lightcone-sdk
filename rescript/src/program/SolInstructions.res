// Solana System and Tokenkeg instruction bytes, kept separate from Lightcone's ABI.
let mint = SolanaKit.address("So11111111111111111111111111111111111111112")
let bytes = OrderPayload.concatBytes
let u8 = value => SolanaKitCodec.encode(SolanaKitCodec.getU8Encoder(), value)
let u32 = value => SolanaKitCodec.encode(SolanaKitCodec.getU32Encoder(), value)
let u64 = value => SolanaKitCodec.encode(SolanaKitCodec.getU64Encoder(), value)
let address = value => SolanaKitCodec.encode(SolanaKitCodec.getAddressEncoder(), value)
let utf8 = value => SolanaKitCodec.encode(SolanaKitCodec.getUtf8Encoder(), value)
let meta = (address, role): SolanaKit.accountMeta => {address, role}
let ix = (programAddress, accounts, data): SolanaKit.instruction => {programAddress, accounts, data}
let canonical = async wallet => {
  let (account, _) = await SolanaProgramToken.findAssociatedTokenPda({
    owner: wallet,
    tokenProgram: Constants.tokenProgram,
    mint,
  })
  account
}
let transfer = (wallet, recipient, amount) =>
  ix(Constants.systemProgram, [meta(wallet, 3), meta(recipient, 1)], bytes([u32(2), u64(amount)]))
let createAta = (wallet, account, ~strict=false) =>
  ix(
    Constants.associatedTokenProgram,
    [
      meta(wallet, 3),
      meta(account, 1),
      meta(wallet, 0),
      meta(mint, 0),
      meta(Constants.systemProgram, 0),
      meta(Constants.tokenProgram, 0),
    ],
    u8(strict ? 0 : 1),
  )
let sync = account => ix(Constants.tokenProgram, [meta(account, 1)], u8(17))
let close = (wallet, account) =>
  ix(Constants.tokenProgram, [meta(account, 1), meta(wallet, 1), meta(wallet, 2)], u8(9))
let reverse: Uint8Array.t => Uint8Array.t = %raw(`a => Uint8Array.from(a).reverse()`)
let slice: (Uint8Array.t, int, int) => Uint8Array.t = %raw(`(a,start,end) => a.slice(start,end)`)
let nativeWithdrawSeed = (~blockhash, ~wallet, ~recipient, ~amount, ~attempt) => {
  let hash = NobleHashes.sha256(
    bytes([
      utf8("lightcone:wsol-withdraw:v1"),
      u8(0),
      address(blockhash),
      address(wallet),
      address(recipient),
      reverse(u64(amount)),
      u8(attempt),
    ]),
  )
  String.toLowerCase(SolanaKitCodec.decode(SolanaKitCodec.getBase16Decoder(), slice(hash, 0, 16)))
}
let temporaryAccount = (wallet, seed) =>
  SolanaKitCodec.decode(
    SolanaKitCodec.getAddressDecoder(),
    NobleHashes.sha256(bytes([address(wallet), utf8(seed), address(Constants.tokenProgram)])),
  )
let temporaryWithdraw = (
  ~wallet,
  ~recipient,
  ~amount,
  ~canonical,
  ~temporary,
  ~seed,
  ~rent,
  ~shortfall,
) => [
  ix(
    Constants.systemProgram,
    [meta(wallet, 3), meta(temporary, 1), meta(wallet, 2)],
    bytes([
      u32(3),
      address(wallet),
      u64(BigInt.fromInt(String.length(seed))),
      utf8(seed),
      u64(rent),
      u64(165n),
      address(Constants.tokenProgram),
    ]),
  ),
  ix(Constants.tokenProgram, [meta(temporary, 1), meta(mint, 0)], bytes([u8(18), address(wallet)])),
  ix(
    Constants.tokenProgram,
    [meta(canonical, 1), meta(temporary, 1), meta(wallet, 2)],
    bytes([u8(3), u64(shortfall)]),
  ),
  close(wallet, temporary),
  transfer(wallet, recipient, amount),
]
