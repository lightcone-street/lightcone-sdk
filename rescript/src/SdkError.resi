// Unified SDK error type — the result-returning core uses `SdkError.t`; the gentype
// throwing facade converts it to a JS `Error` via `unwrap`. Public surface: the error
// variants + `toMessage` + the dual-surface helpers (`unwrap` / `throwAsJsError`) + the
// JSON helpers used by the transport/domains (`stripNulls` / `parseApiResponse`). The
// per-variant message builders and the rejection-code wire codec are internal.

module RejectionCode: {
  @genType
  type t =
    | InsufficientBalance
    | Expired
    | NonceMismatch
    | SelfTrade
    | MarketInactive
    | BelowMinOrderSize
    | InvalidNonce
    | BroadcastFailure
    | OrderNotFound
    | NotOrderMaker
    | OrderAlreadyFilled
    | OrderAlreadyCancelled
    | DuplicateOrder
    | PostOnlyWouldCross
    | FokNoFill
    | IocNoFill
    | WouldCrossUnavailableLiquidity
    | WouldCrossBook
    | MarketNotFound
    | OrderbookNotFound
    | TokenPairMismatch
    | InsufficientMarketFeeBuffer
    | SignatureExpired
    | Unknown(string)

  // Human-readable label (Title Case), e.g. "Insufficient Balance".
  let label: t => string
}

@genType
type apiRejectedDetails = {
  reason: string,
  rejectionCode?: RejectionCode.t,
  errorCode?: string,
  errorLogId?: string,
  requestId?: string,
}

@genType
type httpError =
  | ServerError({status: int, body: string})
  | RateLimited({retryAfterMs: option<int>})
  | Unauthorized
  | NotFound(string)
  | BadRequest(string)
  | Timeout
  | Network(string)
  | MaxRetriesExceeded({attempts: int, lastError: option<string>})

@genType
type wsError =
  | NotConnected
  | ConnectionFailed(string)
  | SendFailed(string)
  | DeserializationError(string)
  | ProtocolError(string)
  | Closed({code: option<int>, reason: string})

@genType
type authError =
  | NotAuthenticated
  | LoginFailed(string)
  | SignatureVerificationFailed
  | TokenExpired

@genType
type t =
  | Http(httpError)
  | Ws(wsError)
  | Auth(authError)
  | Validation(string)
  | Decode(string)
  | Program(string)
  | MissingMarketContext(string)
  | Signing(string)
  | UserCancelled
  | ApiRejected(apiRejectedDetails)
  | Other(string)

// Human-readable message (mirrors the Rust `Display` impls).
let toMessage: t => string

// Deep-strip JSON `null` values so spice's `field?: T` decoder treats null like an
// absent key. Used by the domain + WS decoders before decoding.
let stripNulls: JSON.t => JSON.t

// Parse the internally-tagged `ApiResponse` envelope ({status:"success",body} /
// {status:"error",error_details}); `bodyDecode` is the success body's spice decoder.
let parseApiResponse: (JSON.t => result<'a, Spice.decodeError>, JSON.t) => result<'a, t>

// Throw `t` as a real JS Error (name "LightconeError", carrying `.sdkError`).
let throwAsJsError: (string, t) => 'a

// The facade boundary: unwrap a result-promise into a value-promise that rejects.
let unwrap: promise<result<'a, t>> => promise<'a>
