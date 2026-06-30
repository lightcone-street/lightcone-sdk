// Generic Fetch-based HTTP transport. Public surface: the retry policy, an opaque
// client handle + its session accessors, and the GET/POST verbs. The URL building,
// x-request-id generation, cookie capture, retry/backoff loop, and `ApiResponse`
// unwrapping are all internal.

@genType
type retryPolicy =
  | NoRetry
  | Idempotent

// Opaque HTTP client handle — constructed by `make`, threaded through the verbs.
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
  ~retry: retryPolicy=?,
  ~cookieHeader: string=?,
  ~decode: JSON.t => result<'a, Spice.decodeError>,
) => promise<result<'a, SdkError.t>>

let post: (
  t,
  ~path: string,
  ~body: JSON.t,
  ~retry: retryPolicy=?,
  ~cookieHeader: string=?,
  ~decode: JSON.t => result<'a, Spice.decodeError>,
) => promise<result<'a, SdkError.t>>
