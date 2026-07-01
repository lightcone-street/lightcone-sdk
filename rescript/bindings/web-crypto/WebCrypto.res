// Binding to the Web Crypto global `crypto` (Node 19+, Bun, and browsers). Currently just
// `randomUUID`, used for client-generated ids — the HTTP `x-request-id` (`Http`) and order
// salts (`Order`). Keeping it here means `src/` holds no inline `crypto` `@val external`.

@scope("crypto") @val external randomUUID: unit => string = "randomUUID"
