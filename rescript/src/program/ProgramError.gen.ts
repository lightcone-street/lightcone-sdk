/* TypeScript file generated from ProgramError.resi by genType. */

/* eslint-disable */
/* tslint:disable */

import type {Scaling_Error_t as LightconeSdk_Scaling_Error_t} from './LightconeSdk.gen.ts';

export type t =
    "InvalidPayoutNumerators"
  | "PayoutVectorExceedsU32"
  | "InvalidScalarRange"
  | "DuplicateScalarOutcomes"
  | "SignatureVerificationFailed"
  | "InvalidSignature"
  | "Overflow"
  | "InvalidMintOrder"
  | "OrderbookExists"
  | "InvalidMarket"
  | "MarketSettled"
  | "InvalidProgramId"
  | "InvalidOrderbook"
  | "FullFillRequired"
  | "DivisionByZero"
  | "Reserved50"
  | "InsufficientGlobalDeposit"
  | "InvalidDepositMintOrder"
  | "ZeroAmount"
  | "InvalidAta"
  | "OrderNotFullyFilled"
  | "PayoutTooSmall"
  | "TokenAccountNotEmpty"
  | "InvalidManager"
  | "InvalidFeeRange"
  | "InvalidFeeSum"
  | "InvalidFeeReceiver"
  | "InvalidOracle"
  | "NoPendingRoleTransfer"
  | "PendingRoleMismatch"
  | "InvalidEventAuthority"
  | "EventBatchOverflow"
  | "InvalidEventBatch"
  | "InvalidEventContract"
  | "UnsupportedEventSchema"
  | "PublicInstructionMustBeTopLevel"
  | "InactiveDepositToken"
  | "DepositMintMismatch"
  | "InvalidConditionalMint"
  | "UnsignedOrder"
  | { TAG: "InvalidDiscriminator"; readonly expected: string; readonly actual: string }
  | { TAG: "AccountNotFound"; _0: string }
  | { TAG: "InvalidDataLength"; readonly expected: number; readonly actual: number }
  | { TAG: "InvalidOutcomeCount"; _0: number }
  | { TAG: "InvalidOutcomeIndex"; readonly index: number; readonly max: number }
  | { TAG: "TooManyMakers"; _0: number }
  | { TAG: "InvalidSide"; _0: number }
  | { TAG: "InvalidMarketStatus"; _0: number }
  | { TAG: "InvalidPendingRoleKind"; _0: number }
  | { TAG: "MissingField"; _0: string }
  | { TAG: "TooManyDepositMints"; _0: number }
  | { TAG: "InvalidPubkey"; _0: string }
  | { TAG: "InvalidTransaction"; _0: string }
  | { TAG: "Serialization"; _0: string }
  | { TAG: "Scaling"; _0: LightconeSdk_Scaling_Error_t }
  | { TAG: "Other"; _0: string };
