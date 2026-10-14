module U64: {
  @genType type t = bigint
  let t_decode: JSON.t => result<t, Spice.decodeError>
  let t_encode: t => JSON.t
  let t_encodeJson: t => JSON.t
}
module I64: {
  @genType type t = bigint
  let t_decode: JSON.t => result<t, Spice.decodeError>
  let t_encode: t => JSON.t
  let t_encodeJson: t => JSON.t
}
