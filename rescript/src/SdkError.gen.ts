/* TypeScript file generated from SdkError.resi by genType. */

/* eslint-disable */
/* tslint:disable */

export type RejectionCode_t = 
    "InsufficientBalance"
  | "Expired"
  | "NonceMismatch"
  | "SelfTrade"
  | "MarketInactive"
  | "BelowMinOrderSize"
  | "InvalidNonce"
  | "BroadcastFailure"
  | "OrderNotFound"
  | "NotOrderMaker"
  | "OrderAlreadyFilled"
  | "OrderAlreadyCancelled"
  | "DuplicateOrder"
  | "PostOnlyWouldCross"
  | "FokNoFill"
  | "IocNoFill"
  | "WouldCrossUnavailableLiquidity"
  | "WouldCrossBook"
  | "MarketNotFound"
  | "OrderbookNotFound"
  | "TokenPairMismatch"
  | "InsufficientMarketFeeBuffer"
  | "SignatureExpired"
  | { TAG: "Unknown"; _0: string };

export type apiRejectedDetails = {
  readonly reason: string; 
  readonly rejectionCode?: RejectionCode_t; 
  readonly errorCode?: string; 
  readonly errorLogId?: string; 
  readonly requestId?: string
};

export type httpError = 
    "Unauthorized"
  | "Timeout"
  | { TAG: "ServerError"; readonly status: number; readonly body: string }
  | { TAG: "RateLimited"; readonly retryAfterMs: (undefined | number) }
  | { TAG: "NotFound"; _0: string }
  | { TAG: "BadRequest"; _0: string }
  | { TAG: "Network"; _0: string }
  | { TAG: "MaxRetriesExceeded"; readonly attempts: number; readonly lastError: (undefined | string) };

export type wsError = 
    "NotConnected"
  | { TAG: "ConnectionFailed"; _0: string }
  | { TAG: "SendFailed"; _0: string }
  | { TAG: "DeserializationError"; _0: string }
  | { TAG: "ProtocolError"; _0: string }
  | { TAG: "Closed"; readonly code: (undefined | number); readonly reason: string };

export type authError = 
    "NotAuthenticated"
  | "SignatureVerificationFailed"
  | "TokenExpired"
  | { TAG: "LoginFailed"; _0: string };

export type t = 
    "UserCancelled"
  | { TAG: "Http"; _0: httpError }
  | { TAG: "Ws"; _0: wsError }
  | { TAG: "Auth"; _0: authError }
  | { TAG: "Validation"; _0: string }
  | { TAG: "Decode"; _0: string }
  | { TAG: "Program"; _0: string }
  | { TAG: "MissingMarketContext"; _0: string }
  | { TAG: "Signing"; _0: string }
  | { TAG: "ApiRejected"; _0: apiRejectedDetails }
  | { TAG: "Other"; _0: string };
