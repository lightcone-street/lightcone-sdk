// Compile-guard for README.md — a compile-only file (no test blocks). If a README
// snippet drifts from the actual binding signature, `rescript build` fails here.
let utf8: string => Uint8Array.t = %raw(`(text) => new TextEncoder().encode(text)`)
let toHex: Uint8Array.t => string = %raw(`(bytes) => Array.from(bytes).map((b) => b.toString(16).padStart(2, "0")).join("")`)

let _quickStart = () => {
  let digest = utf8("abc")->NobleHashes.keccak256
  let _hex: string = toHex(digest)
}

let _keccak = (bytes: Uint8Array.t): Uint8Array.t => NobleHashes.keccak256(bytes)
