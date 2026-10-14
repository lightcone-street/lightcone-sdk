// Dalek verify_strict compatibility: canonical scalar, nonweak key and R, and
// the uncofactored equation. This binding is shared by message and tx signing.
type curve
type nacl
@module("@noble/curves/ed25519") external ed25519: curve = "ed25519"
@module("tweetnacl") external nacl: nacl = "default"
let check: (
  curve,
  nacl,
  Uint8Array.t,
  Uint8Array.t,
  Uint8Array.t,
) => bool = %raw(`function(curve,nacl,signature,message,key) {
  try {
    if (signature.length !== 64 || key.length !== 32) return false;
    let scalar = 0n;
    for (let i=63;i>=32;i--) scalar = (scalar << 8n) | BigInt(signature[i]);
    if (scalar >= curve.CURVE.n || curve.Point.fromBytes(key,true).isSmallOrder() ||
        curve.Point.fromBytes(signature.subarray(0,32)).isSmallOrder()) return false;
    return nacl.sign.detached.verify(message,signature,key);
  } catch { return false; }
}`)
let verify = async (signature, message, key) => check(ed25519, nacl, signature, message, key)
