// Auth client — nonce + ed25519 signed-message login, session validation,
// logout, and the Privy / X account endpoints. Public surface: the client
// functions; wire decoding lives in `Auth__Raw`, message building / signing in
// `Auth__Native`.

// Fetch a single-use nonce for the sign-in challenge.
let getNonce: Client.t => promise<result<string, SdkError.t>>

let loginWithMessage: (
  Client.t,
  Auth__Model.SignedLogin.t,
  ~useEmbeddedWallet: bool=?,
) => promise<result<Auth__Model.Session.t, SdkError.t>>

// Convenience: nonce → sign → login, using the client's configured signing
// strategy (native keypair or external wallet adapter).
let login: (
  Client.t,
  ~useEmbeddedWallet: bool=?,
) => promise<result<Auth__Model.Session.t, SdkError.t>>

let checkSession: (
  Client.t,
  ~cookieHeader: string=?,
) => promise<result<Auth__Model.Session.t, SdkError.t>>

let isAuthenticated: Client.t => bool

let logout: Client.t => promise<result<unit, SdkError.t>>

// Register a Privy-authenticated user in the backend DB (idempotent; called
// after Privy login when `is_new_user: true`).
let registerPrivy: Client.t => promise<result<unit, SdkError.t>>

// Disconnect the user's linked X (Twitter) account.
let disconnectX: Client.t => promise<result<unit, SdkError.t>>

// The URL for linking an X (Twitter) account via OAuth.
let connectXUrl: Client.t => string
