// `SolanaKit.Codec` — byte encoders/decoders for manual instruction packing.
// Integer codecs are little-endian by default (matching the Rust SDK's
// `to_le_bytes`); u64/i64 carry their value as a bigint. base16/base58 treat the
// hex/base58 STRING as the "value" (encode str->bytes, decode bytes->str), so
// bytes->hex uses the *decoder*.

@send external encode: (SolanaKit.encoder<'a>, 'a) => Uint8Array.t = "encode"
@send external decode: (SolanaKit.decoder<'a>, Uint8Array.t) => 'a = "decode"
external encoderOfCodec: SolanaKit.codec<'a> => SolanaKit.encoder<'a> = "%identity"
external decoderOfCodec: SolanaKit.codec<'a> => SolanaKit.decoder<'a> = "%identity"

@module("@solana/kit") external getU8Encoder: unit => SolanaKit.encoder<int> = "getU8Encoder"
@module("@solana/kit") external getU16Encoder: unit => SolanaKit.encoder<int> = "getU16Encoder"
@module("@solana/kit") external getU32Encoder: unit => SolanaKit.encoder<int> = "getU32Encoder"
@module("@solana/kit") external getU64Encoder: unit => SolanaKit.encoder<bigint> = "getU64Encoder"
@module("@solana/kit") external getI64Encoder: unit => SolanaKit.encoder<bigint> = "getI64Encoder"
@module("@solana/kit") external getU8Decoder: unit => SolanaKit.decoder<int> = "getU8Decoder"
@module("@solana/kit") external getU16Decoder: unit => SolanaKit.decoder<int> = "getU16Decoder"
@module("@solana/kit") external getU32Decoder: unit => SolanaKit.decoder<int> = "getU32Decoder"
@module("@solana/kit") external getU64Decoder: unit => SolanaKit.decoder<bigint> = "getU64Decoder"
@module("@solana/kit") external getI64Decoder: unit => SolanaKit.decoder<bigint> = "getI64Decoder"

@module("@solana/kit") external getBytesEncoder: unit => SolanaKit.encoder<Uint8Array.t> = "getBytesEncoder"
@module("@solana/kit") external getBytesDecoder: unit => SolanaKit.decoder<Uint8Array.t> = "getBytesDecoder"
@module("@solana/kit")
external fixEncoderSize: (SolanaKit.encoder<Uint8Array.t>, int) => SolanaKit.encoder<Uint8Array.t> =
  "fixEncoderSize"
@module("@solana/kit")
external fixDecoderSize: (SolanaKit.decoder<Uint8Array.t>, int) => SolanaKit.decoder<Uint8Array.t> =
  "fixDecoderSize"

@module("@solana/kit") external getUtf8Encoder: unit => SolanaKit.encoder<string> = "getUtf8Encoder"
@module("@solana/kit") external getUtf8Decoder: unit => SolanaKit.decoder<string> = "getUtf8Decoder"
@module("@solana/kit") external getBase16Encoder: unit => SolanaKit.encoder<string> = "getBase16Encoder"
@module("@solana/kit") external getBase16Decoder: unit => SolanaKit.decoder<string> = "getBase16Decoder"
@module("@solana/kit") external getBase58Encoder: unit => SolanaKit.encoder<string> = "getBase58Encoder"
@module("@solana/kit") external getBase58Decoder: unit => SolanaKit.decoder<string> = "getBase58Decoder"

// Address <-> 32 raw bytes (for PDA seeds and instruction account packing).
@module("@solana/kit")
external getAddressEncoder: unit => SolanaKit.encoder<SolanaKit.address> = "getAddressEncoder"
@module("@solana/kit")
external getAddressDecoder: unit => SolanaKit.decoder<SolanaKit.address> = "getAddressDecoder"
