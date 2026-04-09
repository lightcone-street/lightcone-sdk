const REJECTION_CODE_LABELS = {
  INSUFFICIENT_BALANCE: "Insufficient Balance",
  EXPIRED: "Expired",
  NONCE_MISMATCH: "Nonce Mismatch",
  SELF_TRADE: "Self Trade",
  MARKET_INACTIVE: "Market Inactive",
  BELOW_MIN_ORDER_SIZE: "Below Min Order Size",
  INVALID_NONCE: "Invalid Nonce",
  BROADCAST_FAILURE: "Broadcast Failure",
  ORDER_NOT_FOUND: "Order Not Found",
  NOT_ORDER_MAKER: "Not Order Maker",
  ORDER_ALREADY_FILLED: "Order Already Filled",
  ORDER_ALREADY_CANCELLED: "Order Already Cancelled",
} as const;

type KnownRejectionCode = keyof typeof REJECTION_CODE_LABELS;

function normalizeRejectionCode(raw: string): string {
  return raw.toUpperCase();
}

export class RejectionCode {
  readonly raw: string;

  private constructor(raw: string) {
    this.raw = raw;
  }

  static from(raw: string): RejectionCode {
    return new RejectionCode(raw);
  }

  label(): string {
    const normalized = normalizeRejectionCode(this.raw);
    if (normalized in REJECTION_CODE_LABELS) {
      return REJECTION_CODE_LABELS[normalized as KnownRejectionCode];
    }
    return this.raw;
  }

  wireName(): string {
    const normalized = normalizeRejectionCode(this.raw);
    if (normalized in REJECTION_CODE_LABELS) {
      return normalized;
    }
    return this.raw;
  }

  toString(): string {
    return this.label();
  }
}
