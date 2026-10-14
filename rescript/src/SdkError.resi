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
    | TradingRulesUnavailable
    | OrderFieldOutOfRange
    | PriceNotExactlyRepresentable
    | PriceOutOfRange
    | InvalidPriceDecimals
    | InvalidPriceSignificantFigures
    | InvalidSizeDecimals
    | Unknown(string)

  // Human-readable label (Title Case), e.g. "Insufficient Balance".
  let label: t => string
}

// Structured rejection details — the payload of the `ApiRejected(…)` variant of `t`
// below (module and constructor deliberately share the name).
module LinkedIdentityType: {
  @genType @spice
  type t =
    | @spice.as("email") Email
    | @spice.as("google") Google
    | @spice.as("x") X
    | @spice.as("wallet") Wallet
}

module ApiRejected: {
  @genType
  type t = {
    reason: string,
    rejectionCode?: RejectionCode.t,
    errorCode?: string,
    errorLogId?: string,
    requestId?: string,
    httpStatus?: int,
    retryAfterMs?: float,
    existingMethod?: LinkedIdentityType.t,
  }
}

module HttpError: {
  @genType
  type t =
    | ServerError({status: int, body: string})
    | RateLimited({retryAfterMs: option<float>})
    | Unauthorized
    | NotFound(string)
    | BadRequest(string)
    | Timeout
    | Network(string)
    | MaxRetriesExceeded({attempts: int, lastError: option<string>})
}

module WsError: {
  @genType
  type t =
    | NotConnected
    | ConnectionFailed(string)
    | SendFailed(string)
    | DeserializationError(string)
    | ProtocolError(string)
    | Closed({code: option<int>, reason: string})
}

module AuthError: {
  @genType
  type t =
    | NotAuthenticated
    | LoginFailed(string)
    | SignatureVerificationFailed
    | TokenExpired
}

@genType
type t =
  | Http(HttpError.t)
  | Ws(WsError.t)
  | Auth(AuthError.t)
  | Validation(string)
  | Decode(string)
  | Program(ProgramError.t)
  | InvalidTransaction(string)
  | Scaling(Scaling.Error.t)
  | MissingMarketContext(string)
  | Signing(string)
  | UserCancelled
  | ApiRejected(ApiRejected.t)
  | TransactionFailed({signature: string, error: string})
  | TransactionExpired({signature: string})
  | ConfirmationTimeout({signature: string})
  | SubmissionUnknown({signature: string, lastValidBlockHeight: bigint, reason: string})
  | SubmissionRejected({signature: string, code: int, reason: string})
  | InsufficientSolForTransactionFees({availableLamports: bigint, requiredLamports: bigint})
  | Other(string)

// Human-readable message per error case.
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

let isUnauthorized: t => bool
let isPrivyVerificationUnavailable: t => bool
