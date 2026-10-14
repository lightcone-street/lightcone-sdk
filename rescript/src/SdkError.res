// Unified SDK error type.
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
    | TradingRulesUnavailable
    | OrderFieldOutOfRange
    | PriceNotExactlyRepresentable
    | PriceOutOfRange
    | InvalidPriceDecimals
    | InvalidPriceSignificantFigures
    | InvalidSizeDecimals
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
    | TradingRulesUnavailable => "TRADING_RULES_UNAVAILABLE"
    | OrderFieldOutOfRange => "ORDER_FIELD_OUT_OF_RANGE"
    | PriceNotExactlyRepresentable => "PRICE_NOT_EXACTLY_REPRESENTABLE"
    | PriceOutOfRange => "PRICE_OUT_OF_RANGE"
    | InvalidPriceDecimals => "INVALID_PRICE_DECIMALS"
    | InvalidPriceSignificantFigures => "INVALID_PRICE_SIGNIFICANT_FIGURES"
    | InvalidSizeDecimals => "INVALID_SIZE_DECIMALS"
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
    | "TRADING_RULES_UNAVAILABLE" => TradingRulesUnavailable
    | "ORDER_FIELD_OUT_OF_RANGE" => OrderFieldOutOfRange
    | "PRICE_NOT_EXACTLY_REPRESENTABLE" => PriceNotExactlyRepresentable
    | "PRICE_OUT_OF_RANGE" => PriceOutOfRange
    | "INVALID_PRICE_DECIMALS" => InvalidPriceDecimals
    | "INVALID_PRICE_SIGNIFICANT_FIGURES" => InvalidPriceSignificantFigures
    | "INVALID_SIZE_DECIMALS" => InvalidSizeDecimals
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
// The module's `t` is the details payload carried by the `ApiRejected(…)` variant of
// `SdkError.t` below (module and constructor deliberately share the name).
module LinkedIdentityType = {
  @genType @spice
  type t =
    | @spice.as("email") Email
    | @spice.as("google") Google
    | @spice.as("x") X
    | @spice.as("wallet") Wallet
}

module ApiRejected = {
  type t = {
    reason: string,
    rejectionCode?: RejectionCode.t,
    errorCode?: string,
    errorLogId?: string,
    // Correlation id set client-side from x-request-id; never in the wire body.
    requestId?: string,
    httpStatus?: int,
    retryAfterMs?: float,
    existingMethod?: LinkedIdentityType.t,
  }

  let decode = (json: JSON.t): t => {
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
      rejectionCode: ?(str("rejection_code")->Option.map(RejectionCode.fromWire)),
      errorCode: ?str("error_code"),
      errorLogId: ?str("error_log_id"),
      existingMethod: ?(
        str("existing_method")->Option.flatMap(value =>
          LinkedIdentityType.t_decode(JSON.String(value))->Result.mapOr(None, value => Some(value))
        )
      ),
    }
  }

  let toMessage = (details: t): string => {
    let parts = [`Reason: ${details.reason}`]
    details.rejectionCode->Option.forEach(code =>
      parts->Array.push(`Rejection Code: ${RejectionCode.label(code)}`)
    )
    details.errorCode->Option.forEach(code => parts->Array.push(`Error Code: ${code}`))
    details.errorLogId->Option.forEach(id => parts->Array.push(`Error Log ID: ${id}`))
    details.requestId->Option.forEach(id => parts->Array.push(`Request ID: ${id}`))
    parts->Array.join("\n")
  }
}

// ── Layered error variants ────────────────────────────────────────────────────
module HttpError = {
  type t =
    | ServerError({status: int, body: string})
    | RateLimited({retryAfterMs: option<float>})
    | Unauthorized
    | NotFound(string)
    | BadRequest(string)
    | Timeout
    | Network(string)
    | MaxRetriesExceeded({attempts: int, lastError: option<string>})

  let toMessage = (error: t): string =>
    switch error {
    | ServerError({status, body}) => `Server error ${Int.toString(status)}: ${body}`
    | RateLimited({retryAfterMs}) =>
      switch retryAfterMs {
      | Some(ms) => `Rate limited (retry after ${Float.toString(ms)}ms)`
      | None => "Rate limited"
      }
    | Unauthorized => "Unauthorized"
    | NotFound(message) => `Not found: ${message}`
    | BadRequest(message) => `Bad request: ${message}`
    | Timeout => "Timeout"
    | Network(message) => `Request failed: ${message}`
    | MaxRetriesExceeded({attempts, lastError}) =>
      `Max retries exceeded after ${Int.toString(attempts)} attempts: ${lastError->Option.getOr(
          "",
        )}`
    }
}

module WsError = {
  type t =
    | NotConnected
    | ConnectionFailed(string)
    | SendFailed(string)
    | DeserializationError(string)
    | ProtocolError(string)
    | Closed({code: option<int>, reason: string})

  let toMessage = (error: t): string =>
    switch error {
    | NotConnected => "Not connected"
    | ConnectionFailed(message) => `Connection failed: ${message}`
    | SendFailed(message) => `Send failed: ${message}`
    | DeserializationError(message) => `Deserialization error: ${message}`
    | ProtocolError(message) => `Protocol error: ${message}`
    | Closed({code, reason}) =>
      `Connection closed: code=${code
        ->Option.map(c => Int.toString(c))
        ->Option.getOr("?")} reason=${reason}`
    }
}

module AuthError = {
  type t =
    | NotAuthenticated
    | LoginFailed(string)
    | SignatureVerificationFailed
    | TokenExpired

  let toMessage = (error: t): string =>
    switch error {
    | NotAuthenticated => "Not authenticated"
    | LoginFailed(message) => `Login failed: ${message}`
    | SignatureVerificationFailed => "Signature verification failed"
    | TokenExpired => "Token expired"
    }
}

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

// ── Human-readable messages ────────────────────────────────────────────────────
let toMessage = (error: t): string =>
  switch error {
  | Http(inner) => `HTTP error: ${HttpError.toMessage(inner)}`
  | Ws(inner) => `WebSocket error: ${WsError.toMessage(inner)}`
  | Auth(inner) => `Auth error: ${AuthError.toMessage(inner)}`
  | Validation(message) => `Validation error: ${message}`
  | Decode(message) => `Serialization error: ${message}`
  | InvalidTransaction(message) => `Invalid v1 transaction: ${message}`
  | Scaling(error) => Scaling.toMessage(error)
  | Program(message) => `Program error: ${ProgramError.toMessage(message)}`
  | MissingMarketContext(message) =>
    `Missing required market context for Market deposit source: ${message}`
  | Signing(message) => `Signing error: ${message}`
  | UserCancelled => "User cancelled signing"
  | ApiRejected(details) => ApiRejected.toMessage(details)
  | TransactionFailed({signature, error}) => `Transaction ${signature} failed: ${error}`
  | TransactionExpired({signature}) =>
    `Transaction ${signature} expired; reconcile before rebuilding`
  | ConfirmationTimeout({signature}) => `Confirmation unknown for ${signature}`
  | SubmissionUnknown({signature, reason}) => `Submission unknown for ${signature}: ${reason}`
  | SubmissionRejected({signature, reason}) => `Submission rejected for ${signature}: ${reason}`
  | InsufficientSolForTransactionFees({availableLamports, requiredLamports}) =>
    `Insufficient SOL for fees: ${BigInt.toString(availableLamports)} available, ${BigInt.toString(
        requiredLamports,
      )} required`
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
        | JSON.Null if key == "linked_identities" => Some((key, value))
        | JSON.Null => None
        | _ =>
          Some((
            key,
            if key == "balances" || key == "decimals" {
              value
            } else {
              stripNulls(value)
            },
          ))
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
let parseApiResponse = (bodyDecode: JSON.t => result<'a, Spice.decodeError>, json: JSON.t): result<
  'a,
  t,
> =>
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
      | Some(details) => Error(ApiRejected(ApiRejected.decode(details)))
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

let isUnauthorized = error =>
  switch error {
  | Http(Unauthorized) => true
  | ApiRejected({httpStatus: ?Some(401)}) => true
  | _ => false
  }
let isPrivyVerificationUnavailable = error =>
  switch error {
  | ApiRejected({
      httpStatus: ?Some(503),
      errorCode: ?Some("PRIVY_VERIFICATION_UNAVAILABLE"),
    }) => true
  | _ => false
  }
