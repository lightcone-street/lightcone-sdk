import { Keypair, PublicKey } from "@solana/web3.js";
import bs58 from "bs58";
import type { ClientContext } from "../context";
import { requireSigningStrategy } from "../context";
import { SdkError } from "../error";
import {
  ScalingError,
  scalePriceSize,
  validateRawAmounts,
  validateSignedFields,
  type OrderbookRules,
} from "../shared/scaling";
import { isUserCancellation } from "../shared/signing";
import { Orderbooks, type OrderBookPair } from "../domain/orderbook";
import type {
  DepositSource,
  SubmitOrderRequest,
  TimeInForce,
} from "../shared";
import { Orders, type SubmitOrderResponse } from "../domain/order/client";
import { ProgramSdkError } from "./error";
import {
  applySignature,
  generateSalt,
  hashOrderHex,
  signOrder,
  signOrderFull,
  toSubmitRequest,
} from "./orders";
import { OrderSide, type SignedOrder } from "./types";

interface OrderFields {
  /**
   * The salt is the order's only identity. When the caller does not set one,
   * the first build draws a random salt and every later `payload()`,
   * `sign()`, `finalize()`, or `submit()` on this envelope reuses it, so the
   * hash a wallet signs is the hash that is submitted.
   */
  salt?: bigint;
  maker?: PublicKey;
  market?: PublicKey;
  baseMint?: PublicKey;
  quoteMint?: PublicKey;
  side?: OrderSide;
  amountIn?: bigint;
  amountOut?: bigint;
  /**
   * Whether `amountIn`/`amountOut` were derived from price and size, so the
   * next build derives them again from the current price, size, and side.
   */
  amountsDerived?: boolean;
  expiration: bigint;
  priceRaw?: string;
  sizeRaw?: string;
  depositSource?: DepositSource;
}

function defaultFields(): OrderFields {
  return {
    expiration: 0n,
  };
}

function toUnsignedOrder(fields: OrderFields): Omit<SignedOrder, "signature"> {
  if (!fields.maker) throw ProgramSdkError.missingField("maker");
  if (!fields.market) throw ProgramSdkError.missingField("market");
  if (!fields.baseMint) throw ProgramSdkError.missingField("base_mint");
  if (!fields.quoteMint) throw ProgramSdkError.missingField("quote_mint");
  if (fields.side === undefined) throw ProgramSdkError.missingField("side");
  if (fields.amountIn === undefined) throw ProgramSdkError.missingField("amount_in");
  if (fields.amountOut === undefined) throw ProgramSdkError.missingField("amount_out");
  if (fields.amountIn === 0n) throw ProgramSdkError.missingField("amount_in must be greater than 0");
  if (fields.amountOut === 0n) throw ProgramSdkError.missingField("amount_out must be greater than 0");

  validateSignedFields(fields.amountIn, fields.amountOut);
  const salt = fields.salt ?? generateSalt();
  fields.salt = salt;
  return {
    salt,
    maker: fields.maker,
    market: fields.market,
    baseMint: fields.baseMint,
    quoteMint: fields.quoteMint,
    side: fields.side,
    amountIn: fields.amountIn,
    amountOut: fields.amountOut,
    expiration: fields.expiration,
  };
}

export interface OrderEnvelope {
  salt(value: bigint): this;
  maker(value: PublicKey): this;
  market(value: PublicKey): this;
  baseMint(value: PublicKey): this;
  quoteMint(value: PublicKey): this;
  bid(): this;
  ask(): this;
  side(value: OrderSide): this;
  amountIn(value: bigint): this;
  amountOut(value: bigint): this;
  expiration(value: bigint): this;
  price(value: string): this;
  size(value: string): this;
  depositSource(value: DepositSource): this;
  payload(): Omit<SignedOrder, "signature">;
  sign(keypair: Keypair, orderbook: OrderBookPair, rules: OrderbookRules): SubmitOrderRequest;
  finalize(signatureBase58: string, orderbook: OrderBookPair, rules: OrderbookRules): SubmitOrderRequest;
}

class BaseEnvelope {
  protected readonly fields: OrderFields;

  constructor(fields?: OrderFields) {
    this.fields = fields ? { ...fields } : defaultFields();
  }

  /** Set the order's identity salt (any u64). Drawn once when omitted. */
  salt(value: bigint): this {
    this.fields.salt = value;
    return this;
  }

  maker(value: PublicKey): this {
    this.fields.maker = value;
    return this;
  }

  market(value: PublicKey): this {
    this.fields.market = value;
    return this;
  }

  baseMint(value: PublicKey): this {
    this.fields.baseMint = value;
    return this;
  }

  quoteMint(value: PublicKey): this {
    this.fields.quoteMint = value;
    return this;
  }

  bid(): this {
    this.fields.side = OrderSide.BID;
    return this;
  }

  ask(): this {
    this.fields.side = OrderSide.ASK;
    return this;
  }

  side(value: OrderSide): this {
    this.fields.side = value;
    return this;
  }

  amountIn(value: bigint): this {
    this.clearDerivedAmounts();
    this.fields.amountIn = value;
    return this;
  }

  amountOut(value: bigint): this {
    this.clearDerivedAmounts();
    this.fields.amountOut = value;
    return this;
  }

  expiration(value: bigint): this {
    this.fields.expiration = value;
    return this;
  }

  price(value: string): this {
    this.fields.priceRaw = value;
    return this;
  }

  size(value: string): this {
    this.fields.sizeRaw = value;
    return this;
  }

  depositSource(value: DepositSource): this {
    this.fields.depositSource = value;
    return this;
  }

  payload(): Omit<SignedOrder, "signature"> {
    return toUnsignedOrder(this.fields);
  }

  /** The salt, once set by the caller or fixed by the first build. */
  getSalt(): bigint | undefined {
    return this.fields.salt;
  }

  getMaker(): PublicKey | undefined {
    return this.fields.maker;
  }

  getMarket(): PublicKey | undefined {
    return this.fields.market;
  }

  getBaseMint(): PublicKey | undefined {
    return this.fields.baseMint;
  }

  getQuoteMint(): PublicKey | undefined {
    return this.fields.quoteMint;
  }

  getSide(): OrderSide | undefined {
    return this.fields.side;
  }

  getAmountIn(): bigint | undefined {
    return this.fields.amountIn;
  }

  getAmountOut(): bigint | undefined {
    return this.fields.amountOut;
  }

  getExpiration(): bigint {
    return this.fields.expiration;
  }

  getDepositSource(): DepositSource | undefined {
    return this.fields.depositSource;
  }

  /**
   * Auto-fill market, base_mint, quote_mint, and salt from the orderbook
   * if not explicitly set by the caller.
   */
  protected autoFillFromOrderbook(orderbook: OrderBookPair): void {
    if (!this.fields.market) {
      this.fields.market = new PublicKey(orderbook.marketPubkey);
    }
    if (this.fields.salt === undefined) {
      this.fields.salt = generateSalt();
    }
    if (!this.fields.baseMint) {
      this.fields.baseMint = new PublicKey(orderbook.base.pubkey);
    }
    if (!this.fields.quoteMint) {
      this.fields.quoteMint = new PublicKey(orderbook.quote.pubkey);
    }
  }

  /** Forget amounts derived from price and size by an earlier build. */
  private clearDerivedAmounts(): void {
    if (this.fields.amountsDerived) {
      this.fields.amountIn = undefined;
      this.fields.amountOut = undefined;
      this.fields.amountsDerived = false;
    }
  }

  /**
   * Construct or preflight the signed ratio using fetched immutable rules.
   * Amounts derived from price and size are derived again on every build, so
   * a later price, size, or side change is never ignored.
   */
  protected applyRules(rules: OrderbookRules, orderbookId: string): void {
    if (rules.orderbookId !== orderbookId) {
      throw new ScalingError(
        `trading rules for orderbook '${rules.orderbookId}' cannot be used for '${orderbookId}'`
      );
    }
    if (this.fields.side === undefined) throw ProgramSdkError.missingField("side");
    this.clearDerivedAmounts();
    if (this.fields.amountIn !== undefined && this.fields.amountOut !== undefined) {
      validateRawAmounts(this.fields.amountIn, this.fields.amountOut, this.fields.side, rules);
      return;
    }
    if (this.fields.amountIn !== undefined || this.fields.amountOut !== undefined) {
      throw ProgramSdkError.missingField("amount_in and amount_out must be supplied together");
    }
    if (!this.fields.priceRaw) throw ProgramSdkError.missingField("price");
    if (!this.fields.sizeRaw) throw ProgramSdkError.missingField("size");
    const scaled = scalePriceSize(this.fields.priceRaw, this.fields.sizeRaw, this.fields.side, rules);
    this.fields.amountIn = scaled.amountIn;
    this.fields.amountOut = scaled.amountOut;
    this.fields.amountsDerived = true;
  }

}

export class LimitOrderEnvelope extends BaseEnvelope implements OrderEnvelope {
  private timeInForceValue?: TimeInForce;

  static new(): LimitOrderEnvelope {
    return new LimitOrderEnvelope();
  }

  timeInForce(value: TimeInForce): this {
    this.timeInForceValue = value;
    return this;
  }

  getTimeInForce(): TimeInForce | undefined {
    return this.timeInForceValue;
  }

  sign(keypair: Keypair, orderbook: OrderBookPair, rules: OrderbookRules): SubmitOrderRequest {
    this.autoFillFromOrderbook(orderbook);
    this.applyRules(rules, orderbook.orderbookId);
    const signed = signOrderFull(this.payload(), keypair, rules);
    return toSubmitRequest(signed, orderbook.orderbookId, {
      timeInForce: this.timeInForceValue,
      depositSource: this.getDepositSource(),
    });
  }

  /** Validate an external base58 signature and attach it to this order. */
  finalize(signatureBase58: string, orderbook: OrderBookPair, rules: OrderbookRules): SubmitOrderRequest {
    this.autoFillFromOrderbook(orderbook);
    this.applyRules(rules, orderbook.orderbookId);
    const signed = applySignature(this.payload(), signatureBase58, rules);
    return toSubmitRequest(signed, orderbook.orderbookId, {
      timeInForce: this.timeInForceValue,
      depositSource: this.getDepositSource(),
    });
  }

  /**
   * Submit this order using the client's signing strategy.
   *
   * - **native**: signs locally with the keypair, submits via REST
   * - **walletAdapter**: signs the order hash via the external signer,
   *   submits via REST
   *
   * Fills orderbook-derived fields (market, mints, salt) and scales
   * price/size to raw amounts before signing. Submission is never retried:
   * if the outcome is unknown (`ENGINE_UNAVAILABLE`, a timeout, or a lost
   * connection), call `submit` again on this same envelope, which reuses its
   * salt and resubmits the identical order; `DUPLICATE_ORDER` or
   * `ALREADY_EXISTS` then proves the first submission was accepted. Changing
   * any field, including the salt, creates a different order.
   */
  async submit(
    client: ClientContext,
    orderbook: OrderBookPair
  ): Promise<SubmitOrderResponse> {
    const rules = await new Orderbooks(client).decimals(orderbook.orderbookId);
    const strategy = requireSigningStrategy(client);
    this.autoFillFromOrderbook(orderbook);
    this.applyRules(rules, orderbook.orderbookId);

    switch (strategy.type) {
      case "native": {
        const request = this.sign(strategy.keypair, orderbook, rules);
        return new Orders(client).submit(request);
      }
      case "walletAdapter": {
        const hash = hashOrderHex(this.payload());
        const sigBytes = await strategy.signer
          .signMessage(new TextEncoder().encode(hash))
          .catch((err: unknown) => {
            const msg = err instanceof Error ? err.message : String(err);
            if (isUserCancellation(msg)) throw SdkError.userCancelled();
            throw SdkError.signing(msg);
          });
        const request = this.finalize(bs58.encode(sigBytes), orderbook, rules);
        return new Orders(client).submit(request);
      }
      case "privy":
        throw SdkError.validation(
          "Privy order signing is not supported; configure a native signer or an ExternalSigner"
        );
    }
  }
}

export function signPayload(
  payload: Omit<SignedOrder, "signature">,
  keypair: Keypair,
  rules: OrderbookRules
): string {
  return signOrder(
    { ...payload, signature: Buffer.alloc(64) },
    keypair,
    rules
  ).toString("hex");
}
