/* TypeScript file generated from SdkError.resi by genType. */

/* eslint-disable */
/* tslint:disable */

import type {ProgramError_t as LightconeSdk_ProgramError_t} from './LightconeSdk.gen.ts';

import type {Scaling_Error_t as LightconeSdk_Scaling_Error_t} from './LightconeSdk.gen.ts';

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
  | "TradingRulesUnavailable"
  | "OrderFieldOutOfRange"
  | "PriceNotExactlyRepresentable"
  | "PriceOutOfRange"
  | "InvalidPriceDecimals"
  | "InvalidPriceSignificantFigures"
  | "InvalidSizeDecimals"
  | { TAG: "Unknown"; _0: string };

export type LinkedIdentityType_t = "Email" | "Google" | "X" | "Wallet";

export type ApiRejected_t = {
  readonly reason: string;
  readonly rejectionCode?: RejectionCode_t;
  readonly errorCode?: string;
  readonly errorLogId?: string;
  readonly requestId?: string;
  readonly httpStatus?: number;
  readonly retryAfterMs?: number;
  readonly existingMethod?: LinkedIdentityType_t
};

export type HttpError_t =
    "Unauthorized"
  | "Timeout"
  | { TAG: "ServerError"; readonly status: number; readonly body: string }
  | { TAG: "RateLimited"; readonly retryAfterMs: (undefined | number) }
  | { TAG: "NotFound"; _0: string }
  | { TAG: "BadRequest"; _0: string }
  | { TAG: "Network"; _0: string }
  | { TAG: "MaxRetriesExceeded"; readonly attempts: number; readonly lastError: (undefined | string) };

export type WsError_t =
    "NotConnected"
  | { TAG: "ConnectionFailed"; _0: string }
  | { TAG: "SendFailed"; _0: string }
  | { TAG: "DeserializationError"; _0: string }
  | { TAG: "ProtocolError"; _0: string }
  | { TAG: "Closed"; readonly code: (undefined | number); readonly reason: string };

export type AuthError_t =
    "NotAuthenticated"
  | "SignatureVerificationFailed"
  | "TokenExpired"
  | { TAG: "LoginFailed"; _0: string };

export type t =
    "UserCancelled"
  | { TAG: "Http"; _0: HttpError_t }
  | { TAG: "Ws"; _0: WsError_t }
  | { TAG: "Auth"; _0: AuthError_t }
  | { TAG: "Validation"; _0: string }
  | { TAG: "Decode"; _0: string }
  | { TAG: "Program"; _0: LightconeSdk_ProgramError_t }
  | { TAG: "InvalidTransaction"; _0: string }
  | { TAG: "Scaling"; _0: LightconeSdk_Scaling_Error_t }
  | { TAG: "MissingMarketContext"; _0: string }
  | { TAG: "Signing"; _0: string }
  | { TAG: "ApiRejected"; _0: ApiRejected_t }
  | { TAG: "TransactionFailed"; readonly signature: string; readonly error: string }
  | { TAG: "TransactionExpired"; readonly signature: string }
  | { TAG: "ConfirmationTimeout"; readonly signature: string }
  | { TAG: "SubmissionUnknown"; readonly signature: string; readonly lastValidBlockHeight: bigint; readonly reason: string }
  | { TAG: "SubmissionRejected"; readonly signature: string; readonly code: number; readonly reason: string }
  | { TAG: "InsufficientSolForTransactionFees"; readonly availableLamports: bigint; readonly requiredLamports: bigint }
  | { TAG: "Other"; _0: string };
