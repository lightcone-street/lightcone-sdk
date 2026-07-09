// Generic Fetch-based HTTP transport. Public surface: the retry policy, an opaque
// client handle + its session accessors, and the GET/POST verbs. The URL building,
// x-request-id generation, cookie capture, retry/backoff loop, and `ApiResponse`
// unwrapping are all internal.

module RetryPolicy: {
  @genType
  type t =
    | NoRetry
    | Idempotent
}

// Opaque to TypeScript: the configured HTTP transport is a capability (base URL + a mutable
// cookie store holding the session token), driven through the verbs below — not data to
// inspect or build, and structurally exposing it would surface the session token.
// `authToken` gives read access to the token where a consumer actually needs it.
@genType.opaque
type t

let make: string => t
let baseUrl: t => string
let authToken: t => option<string>
let setAuthToken: (t, string) => unit
let clearAuthToken: t => unit

// GET defaults to the idempotent retry policy; POST defaults to no retries.
// `decode` is the success body's spice decoder; the `ApiResponse` envelope is
// unwrapped internally.
let get: (
  t,
  ~path: string,
  ~query: array<(string, string)>=?,
  ~retry: RetryPolicy.t=?,
  ~cookieHeader: string=?,
  ~decode: JSON.t => result<'a, Spice.decodeError>,
) => promise<result<'a, SdkError.t>>

let post: (
  t,
  ~path: string,
  ~body: JSON.t,
  ~retry: RetryPolicy.t=?,
  ~cookieHeader: string=?,
  ~decode: JSON.t => result<'a, Spice.decodeError>,
) => promise<result<'a, SdkError.t>>
