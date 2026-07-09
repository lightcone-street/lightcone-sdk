/* TypeScript file generated from Shared.resi by genType. */

/* eslint-disable */
/* tslint:disable */

export type orderBookId = string;

export type pubkeyStr = string;

export type Side_t = "bid" | "ask";

export type Denominator_t = "Base" | "Quote";

export type TimeInForce_t = "GTC" | "IOC" | "FOK" | "ALO";

export type TriggerType_t = "TP" | "SL";

export type OrderStatus_t = 
    "OPEN"
  | "MATCHING"
  | "CANCELLED"
  | "FILLED"
  | "PENDING";

export type TriggerStatus_t = 
    "created"
  | "triggered"
  | "failed"
  | "expired"
  | "invalidated";

export type OrderUpdateType_t = "PLACEMENT" | "UPDATE" | "CANCELLATION";

export type TriggerUpdateType_t = 
    "CREATED"
  | "TRIGGERED"
  | "FAILED"
  | "EXPIRED"
  | "INVALIDATED";

export type TriggerResultStatus_t = "filled" | "accepted" | "rejected";

export type DepositSource_t = "global" | "market";

export type Resolution_t = "1m" | "5m" | "15m" | "1h" | "4h" | "1d";
