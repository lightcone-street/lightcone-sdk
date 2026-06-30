// Unified SDK error type — the ReScript counterpart of the Rust SDK's `SdkError`.
// Named `SdkError` (not `Error`) so it doesn't shadow the stdlib `Error` module.
//
// The result-returning core API uses `SdkError.t`. The gentype-exported throwing
// facade converts it to a real JS `Error` (carrying `.message` + `.sdkError`) via
// `unwrap` — see TypeScriptApi.res.

// ── Machine-readable rejection codes (custom spice codec) ─────────────────────
module RejectionCode = {
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

  let toWire = (code: t): string =>
    switch code {
    | InsufficientBalance => "INSUFFICIENT_BALANCE"
    | Expired => "EXPIRED"
    | NonceMismatch => "NONCE_MISMATCH"
    | SelfTrade => "SELF_TRADE"
    | MarketInactive => "MARKET_INACTIVE"
    | BelowMinOrderSize => "BELOW_MIN_ORDER_SIZE"
    | InvalidNonce => "INVALID_NONCE"
    | BroadcastFailure => "BROADCAST_FAILURE"
    | OrderNotFound => "ORDER_NOT_FOUND"
    | NotOrderMaker => "NOT_ORDER_MAKER"
    | OrderAlreadyFilled => "ORDER_ALREADY_FILLED"
    | OrderAlreadyCancelled => "ORDER_ALREADY_CANCELLED"
    | DuplicateOrder => "DUPLICATE_ORDER"
    | PostOnlyWouldCross => "POST_ONLY_WOULD_CROSS"
    | FokNoFill => "FOK_NO_FILL"
    | IocNoFill => "IOC_NO_FILL"
    | WouldCrossUnavailableLiquidity => "WOULD_CROSS_UNAVAILABLE_LIQUIDITY"
    | WouldCrossBook => "WOULD_CROSS_BOOK"
    | MarketNotFound => "MARKET_NOT_FOUND"
    | OrderbookNotFound => "ORDERBOOK_NOT_FOUND"
    | TokenPairMismatch => "TOKEN_PAIR_MISMATCH"
    | InsufficientMarketFeeBuffer => "INSUFFICIENT_MARKET_FEE_BUFFER"
    | SignatureExpired => "SIGNATURE_EXPIRED"
    | Unknown(code) => code
    }

  // Backend sends SCREAMING_SNAKE_CASE; accept any case, fall back to Unknown.
  let fromWire = (raw: string): t =>
    switch String.toUpperCase(raw) {
    | "INSUFFICIENT_BALANCE" => InsufficientBalance
    | "EXPIRED" => Expired
    | "NONCE_MISMATCH" => NonceMismatch
    | "SELF_TRADE" => SelfTrade
    | "MARKET_INACTIVE" => MarketInactive
    | "BELOW_MIN_ORDER_SIZE" => BelowMinOrderSize
    | "INVALID_NONCE" => InvalidNonce
    | "BROADCAST_FAILURE" => BroadcastFailure
    | "ORDER_NOT_FOUND" => OrderNotFound
    | "NOT_ORDER_MAKER" => NotOrderMaker
    | "ORDER_ALREADY_FILLED" => OrderAlreadyFilled
    | "ORDER_ALREADY_CANCELLED" => OrderAlreadyCancelled
    | "DUPLICATE_ORDER" => DuplicateOrder
    | "POST_ONLY_WOULD_CROSS" => PostOnlyWouldCross
    | "FOK_NO_FILL" => FokNoFill
    | "IOC_NO_FILL" => IocNoFill
    | "WOULD_CROSS_UNAVAILABLE_LIQUIDITY" => WouldCrossUnavailableLiquidity
    | "WOULD_CROSS_BOOK" => WouldCrossBook
    | "MARKET_NOT_FOUND" => MarketNotFound
    | "ORDERBOOK_NOT_FOUND" => OrderbookNotFound
    | "TOKEN_PAIR_MISMATCH" => TokenPairMismatch
    | "INSUFFICIENT_MARKET_FEE_BUFFER" => InsufficientMarketFeeBuffer
    | "SIGNATURE_EXPIRED" => SignatureExpired
    | _ => Unknown(raw)
    }

  // Human-readable label (Title Case), e.g. "Insufficient Balance".
  let label = (code: t): string =>
    switch code {
    | Unknown(raw) => raw
    | _ =>
      toWire(code)
      ->String.split("_")
      ->Array.map(word => {
        let lower = String.toLowerCase(word)
        switch String.length(lower) {
        | 0 => ""
        | length =>
          String.toUpperCase(String.slice(lower, ~start=0, ~end=1)) ++
            String.slice(lower, ~start=1, ~end=length)
        }
      })
      ->Array.join(" ")
    }

  let codec: Spice.codec<t> = (
    code => JSON.String(toWire(code)),
    json =>
      switch json {
      | JSON.String(raw) => Ok(fromWire(raw))
      | _ => Spice.error("Expected a rejection-code string", json)
      },
  )
}

// ── Structured rejection details (hand-decoded; the wrapper is internally-tagged) ──
type apiRejectedDetails = {
  reason: string,
  rejectionCode?: RejectionCode.t,
  errorCode?: string,
  errorLogId?: string,
  // Correlation id set client-side from x-request-id; never in the wire body.
  requestId?: string,
}

let decodeRejectedDetails = (json: JSON.t): apiRejectedDetails => {
  let str = key =>
    switch json {
    | JSON.Object(dict) =>
      switch Dict.get(dict, key) {
      | Some(JSON.String(value)) => Some(value)
      | _ => None
      }
    | _ => None
    }
  {
    reason: str("reason")->Option.getOr(""),
    rejectionCode: ?str("rejection_code")->Option.map(RejectionCode.fromWire),
    errorCode: ?str("error_code"),
    errorLogId: ?str("error_log_id"),
  }
}

// ── Layered error variants ────────────────────────────────────────────────────
type httpError =
  | ServerError({status: int, body: string})
  | RateLimited({retryAfterMs: option<int>})
  | Unauthorized
  | NotFound(string)
  | BadRequest(string)
  | Timeout
  | Network(string)
  | MaxRetriesExceeded({attempts: int, lastError: option<string>})

type wsError =
  | NotConnected
  | ConnectionFailed(string)
  | SendFailed(string)
  | DeserializationError(string)
  | ProtocolError(string)
  | Closed({code: option<int>, reason: string})

type authError =
  | NotAuthenticated
  | LoginFailed(string)
  | SignatureVerificationFailed
  | TokenExpired

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

// ── Human-readable messages (mirror the Rust `Display` impls) ─────────────────
let httpErrorToMessage = (error: httpError): string =>
  switch error {
  | ServerError({status, body}) => `Server error ${Int.toString(status)}: ${body}`
  | RateLimited({retryAfterMs}) =>
    switch retryAfterMs {
    | Some(ms) => `Rate limited (retry after ${Int.toString(ms)}ms)`
    | None => "Rate limited"
    }
  | Unauthorized => "Unauthorized"
  | NotFound(message) => `Not found: ${message}`
  | BadRequest(message) => `Bad request: ${message}`
  | Timeout => "Timeout"
  | Network(message) => `Request failed: ${message}`
  | MaxRetriesExceeded({attempts, lastError}) =>
    `Max retries exceeded after ${Int.toString(attempts)} attempts: ${lastError->Option.getOr("")}`
  }

let wsErrorToMessage = (error: wsError): string =>
  switch error {
  | NotConnected => "Not connected"
  | ConnectionFailed(message) => `Connection failed: ${message}`
  | SendFailed(message) => `Send failed: ${message}`
  | DeserializationError(message) => `Deserialization error: ${message}`
  | ProtocolError(message) => `Protocol error: ${message}`
  | Closed({code, reason}) =>
    `Connection closed: code=${code->Option.map(c => Int.toString(c))->Option.getOr("?")} reason=${reason}`
  }

let authErrorToMessage = (error: authError): string =>
  switch error {
  | NotAuthenticated => "Not authenticated"
  | LoginFailed(message) => `Login failed: ${message}`
  | SignatureVerificationFailed => "Signature verification failed"
  | TokenExpired => "Token expired"
  }

let rejectedToMessage = (details: apiRejectedDetails): string => {
  let parts = [`Reason: ${details.reason}`]
  details.rejectionCode->Option.forEach(code => parts->Array.push(`Rejection Code: ${RejectionCode.label(code)}`))
  details.errorCode->Option.forEach(code => parts->Array.push(`Error Code: ${code}`))
  details.errorLogId->Option.forEach(id => parts->Array.push(`Error Log ID: ${id}`))
  details.requestId->Option.forEach(id => parts->Array.push(`Request ID: ${id}`))
  parts->Array.join("\n")
}

let toMessage = (error: t): string =>
  switch error {
  | Http(inner) => `HTTP error: ${httpErrorToMessage(inner)}`
  | Ws(inner) => `WebSocket error: ${wsErrorToMessage(inner)}`
  | Auth(inner) => `Auth error: ${authErrorToMessage(inner)}`
  | Validation(message) => `Validation error: ${message}`
  | Decode(message) => `Serialization error: ${message}`
  | Program(message) => `Program error: ${message}`
  | MissingMarketContext(message) => `Missing required market context for Market deposit source: ${message}`
  | Signing(message) => `Signing error: ${message}`
  | UserCancelled => "User cancelled signing"
  | ApiRejected(details) => rejectedToMessage(details)
  | Other(message) => message
  }

// ── Dual-surface helpers ──────────────────────────────────────────────────────
// Deep-strip JSON `null` values: backend `Option<T>` fields without
// `skip_serializing_if` arrive as `null`, but spice's `field?: T` decoder only
// tolerates an ABSENT key (a present `null` makes it reject). Removing null keys
// makes every optional field handle both absent and null uniformly as `None`.
let rec stripNulls = (json: JSON.t): JSON.t =>
  switch json {
  | JSON.Object(dict) =>
    JSON.Object(
      dict
      ->Dict.toArray
      ->Array.filterMap(((key, value)) =>
        switch value {
        | JSON.Null => None
        | _ => Some((key, stripNulls(value)))
        }
      )
      ->Dict.fromArray,
    )
  | JSON.Array(items) => JSON.Array(items->Array.map(stripNulls))
  | other => other
  }

// Parse the internally-tagged `ApiResponse` envelope: {"status":"success","body":…}
// or {"status":"error","error_details":…}. `bodyDecode` is the spice `_decode` of
// the success body type.
let parseApiResponse = (
  bodyDecode: JSON.t => result<'a, Spice.decodeError>,
  json: JSON.t,
): result<'a, t> =>
  switch json {
  | JSON.Object(dict) =>
    switch Dict.get(dict, "status") {
    | Some(JSON.String("success")) =>
      switch Dict.get(dict, "body") {
      | Some(body) => bodyDecode(stripNulls(body))->Result.mapError(error => Decode(error.message))
      | None => Error(Decode("success response missing 'body'"))
      }
    | Some(JSON.String("error")) =>
      switch Dict.get(dict, "error_details") {
      | Some(details) => Error(ApiRejected(decodeRejectedDetails(details)))
      | None => Error(Other("error response missing 'error_details'"))
      }
    | _ => Error(Decode("response missing 'status'"))
    }
  | _ => Error(Decode("response is not an object"))
  }

// Throw `t` as a real JS Error so TypeScript consumers get idiomatic rejections.
let throwAsJsError: (string, t) => 'a = %raw(`function (message, sdkError) {
  const err = new Error(message);
  err.name = "LightconeError";
  err.sdkError = sdkError;
  throw err;
}`)

// The facade boundary: unwrap a result-promise into a value-promise that rejects.
let unwrap = async (promised: promise<result<'a, t>>): 'a =>
  switch await promised {
  | Ok(value) => value
  | Error(error) => throwAsJsError(toMessage(error), error)
  }
