import { Keypair, PublicKey } from "@solana/web3.js";
import bs58 from "bs58";
import type { ClientContext } from "../context";
import { requireSigningStrategy } from "../context";
import { SdkError } from "../error";
import { RetryPolicy } from "../http";
import {
  privyOrderFromLimitEnvelope,
  privyOrderFromTriggerEnvelope,
} from "../privy";
import {
  scalePriceSize,
  validateRawAmounts,
  validateSignedFields,
  validateTriggerPrice,
  type OrderbookRules,
} from "../shared/scaling";
import { isUserCancellation } from "../shared/signing";
import { Orderbooks, type OrderBookPair } from "../domain/orderbook";
import type {
  DepositSource,
  SubmitOrderRequest,
  TimeInForce,
  TriggerType,
} from "../shared";
import type { SubmitOrderResponse } from "../domain/order/client";
import type { TriggerOrderResponse } from "../domain/order/client";
import { ProgramSdkError } from "./error";
import {
  generateSalt,
  hashOrderHex,
  signOrder,
  signOrderFull,
  toSubmitRequest,
  type SubmitRequestOptions,
} from "./orders";
import { OrderSide, type SignedOrder } from "./types";

interface OrderFields {
  nonce?: number;
  salt?: bigint;
  maker?: PublicKey;
  market?: PublicKey;
  baseMint?: PublicKey;
  quoteMint?: PublicKey;
  side?: OrderSide;
  amountIn?: bigint;
  amountOut?: bigint;
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
  if (fields.nonce === undefined) throw ProgramSdkError.missingField("nonce");
  if (!fields.maker) throw ProgramSdkError.missingField("maker");
  if (!fields.market) throw ProgramSdkError.missingField("market");
  if (!fields.baseMint) throw ProgramSdkError.missingField("base_mint");
  if (!fields.quoteMint) throw ProgramSdkError.missingField("quote_mint");
  if (fields.side === undefined) throw ProgramSdkError.missingField("side");
  if (fields.amountIn === undefined) throw ProgramSdkError.missingField("amount_in");
  if (fields.amountOut === undefined) throw ProgramSdkError.missingField("amount_out");

  const salt = fields.salt ?? generateSalt();
  fields.salt = salt;
  validateSignedFields(fields.amountIn, fields.amountOut, salt, fields.nonce);
  return {
    nonce: fields.nonce,
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
  nonce(value: number): this;
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

  nonce(value: number): this {
    this.fields.nonce = value;
    return this;
  }

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
    this.fields.amountIn = value;
    return this;
  }

  amountOut(value: bigint): this {
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

  getNonce(): number | undefined {
    return this.fields.nonce;
  }

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

  /** Construct or preflight the signed ratio using fetched immutable rules. */
  protected applyRules(rules: OrderbookRules): void {
    if (this.fields.side === undefined) throw ProgramSdkError.missingField("side");
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
  }

  protected finalizeWithHexSignature(
    signatureHex: string,
    orderbookId: string,
    options: SubmitRequestOptions = {}
  ): SubmitOrderRequest {
    const unsigned = toUnsignedOrder(this.fields);
    return toSubmitRequest(
      {
        ...unsigned,
        signature: Buffer.from(signatureHex, "hex"),
      },
      orderbookId,
      {
        ...options,
        depositSource: this.fields.depositSource,
      }
    );
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
    this.applyRules(rules);
    const signed = signOrderFull(this.payload(), keypair, rules);
    return toSubmitRequest(signed, orderbook.orderbookId, {
      timeInForce: this.timeInForceValue,
      depositSource: this.getDepositSource(),
    });
  }

  finalize(signatureBase58: string, orderbook: OrderBookPair, rules: OrderbookRules): SubmitOrderRequest {
    this.autoFillFromOrderbook(orderbook);
    this.applyRules(rules);
    const signatureHex = Buffer.from(bs58.decode(signatureBase58)).toString("hex");
    return this.finalizeWithHexSignature(signatureHex, orderbook.orderbookId, {
      timeInForce: this.timeInForceValue,
    });
  }

  async submit(
    client: ClientContext,
    orderbook: OrderBookPair
  ): Promise<SubmitOrderResponse> {
    const rules = await new Orderbooks(client).decimals(orderbook.orderbookId);
    const strategy = requireSigningStrategy(client);
    this.autoFillFromOrderbook(orderbook);
    this.applyRules(rules);

    // Nonce cache: cache if explicitly set, auto-populate from cache if not
    if (this.fields.nonce !== undefined) {
      client.setOrderNonce?.(this.fields.nonce);
    } else {
      this.fields.nonce = client.orderNonce?.() ?? 0;
    }

    switch (strategy.type) {
      case "native": {
        const request = this.sign(strategy.keypair, orderbook, rules);
        const url = `${client.http.baseUrl()}/api/orders/submit`;
        return client.http.post<SubmitOrderResponse, SubmitOrderRequest>(
          url,
          request,
          RetryPolicy.None
        );
      }
      case "walletAdapter": {
        const unsigned = this.payload();
        const hash = hashOrderHex({ ...unsigned, signature: Buffer.alloc(64) });
        const sigBytes = await strategy.signer
          .signMessage(new TextEncoder().encode(hash))
          .catch((err: unknown) => {
            const msg = err instanceof Error ? err.message : String(err);
            if (isUserCancellation(msg)) throw SdkError.userCancelled();
            throw SdkError.signing(msg);
          });
        const request = this.finalize(bs58.encode(sigBytes), orderbook, rules);
        const url = `${client.http.baseUrl()}/api/orders/submit`;
        return client.http.post<SubmitOrderResponse, SubmitOrderRequest>(
          url,
          request,
          RetryPolicy.None
        );
      }
      case "privy": {
        const envelope = privyOrderFromLimitEnvelope(this, orderbook.orderbookId);
        const url = `${client.http.baseUrl()}/api/privy/sign_and_send_order`;
        return client.http.post(url, { wallet_id: strategy.walletId, order: envelope }, RetryPolicy.None);
      }
    }
  }
}

export class TriggerOrderEnvelope extends BaseEnvelope implements OrderEnvelope {
  private timeInForceValue?: TimeInForce;
  private triggerPriceValue?: string;
  private triggerTypeValue?: TriggerType;

  static new(): TriggerOrderEnvelope {
    return new TriggerOrderEnvelope();
  }

  timeInForce(value: TimeInForce): this {
    this.timeInForceValue = value;
    return this;
  }

  triggerPrice(value: string): this {
    this.triggerPriceValue = value;
    return this;
  }

  triggerType(value: TriggerType): this {
    this.triggerTypeValue = value;
    return this;
  }

  takeProfit(price: string): this {
    this.triggerPriceValue = price;
    this.triggerTypeValue = "TP" as TriggerType;
    return this;
  }

  stopLoss(price: string): this {
    this.triggerPriceValue = price;
    this.triggerTypeValue = "SL" as TriggerType;
    return this;
  }

  gtc(): this {
    this.timeInForceValue = "GTC" as TimeInForce;
    return this;
  }

  ioc(): this {
    this.timeInForceValue = "IOC" as TimeInForce;
    return this;
  }

  fok(): this {
    this.timeInForceValue = "FOK" as TimeInForce;
    return this;
  }

  alo(): this {
    this.timeInForceValue = "ALO" as TimeInForce;
    return this;
  }

  getTimeInForce(): TimeInForce | undefined {
    return this.timeInForceValue;
  }

  getTriggerPrice(): string | undefined {
    return this.triggerPriceValue;
  }

  getTriggerType(): TriggerType | undefined {
    return this.triggerTypeValue;
  }

  sign(keypair: Keypair, orderbook: OrderBookPair, rules: OrderbookRules): SubmitOrderRequest {
    const trigger = this.requireTriggerFields();
    validateTriggerPrice(trigger.price, rules.priceDecimals);
    this.autoFillFromOrderbook(orderbook);
    this.applyRules(rules);
    const signed = signOrderFull(this.payload(), keypair, rules);
    return toSubmitRequest(signed, orderbook.orderbookId, {
      timeInForce: this.timeInForceValue,
      triggerPrice: Number(trigger.price),
      triggerType: trigger.type,
      depositSource: this.getDepositSource(),
    });
  }

  finalize(signatureBase58: string, orderbook: OrderBookPair, rules: OrderbookRules): SubmitOrderRequest {
    const trigger = this.requireTriggerFields();
    validateTriggerPrice(trigger.price, rules.priceDecimals);
    this.autoFillFromOrderbook(orderbook);
    this.applyRules(rules);
    const signatureHex = Buffer.from(bs58.decode(signatureBase58)).toString("hex");
    return this.finalizeWithHexSignature(signatureHex, orderbook.orderbookId, {
      timeInForce: this.timeInForceValue,
      triggerPrice: Number(trigger.price),
      triggerType: trigger.type,
    });
  }

  async submit(
    client: ClientContext,
    orderbook: OrderBookPair
  ): Promise<TriggerOrderResponse> {
    const rules = await new Orderbooks(client).decimals(orderbook.orderbookId);
    const strategy = requireSigningStrategy(client);
    const trigger = this.requireTriggerFields();
    validateTriggerPrice(trigger.price, rules.priceDecimals);
    this.autoFillFromOrderbook(orderbook);
    this.applyRules(rules);

    // Nonce cache: cache if explicitly set, auto-populate from cache if not
    if (this.fields.nonce !== undefined) {
      client.setOrderNonce?.(this.fields.nonce);
    } else {
      this.fields.nonce = client.orderNonce?.() ?? 0;
    }

    switch (strategy.type) {
      case "native": {
        const request = this.sign(strategy.keypair, orderbook, rules);
        const url = `${client.http.baseUrl()}/api/orders/submit`;
        return client.http.post<TriggerOrderResponse, SubmitOrderRequest>(
          url,
          request,
          RetryPolicy.None
        );
      }
      case "walletAdapter": {
        const unsigned = this.payload();
        const hash = hashOrderHex({ ...unsigned, signature: Buffer.alloc(64) });
        const sigBytes = await strategy.signer
          .signMessage(new TextEncoder().encode(hash))
          .catch((err: unknown) => {
            const msg = err instanceof Error ? err.message : String(err);
            if (isUserCancellation(msg)) throw SdkError.userCancelled();
            throw SdkError.signing(msg);
          });
        const request = this.finalize(bs58.encode(sigBytes), orderbook, rules);
        const url = `${client.http.baseUrl()}/api/orders/submit`;
        return client.http.post<TriggerOrderResponse, SubmitOrderRequest>(
          url,
          request,
          RetryPolicy.None
        );
      }
      case "privy": {
        const envelope = privyOrderFromTriggerEnvelope(this, orderbook.orderbookId);
        const url = `${client.http.baseUrl()}/api/privy/sign_and_send_order`;
        return client.http.post(url, { wallet_id: strategy.walletId, order: envelope }, RetryPolicy.None);
      }
    }
  }

  private requireTriggerFields(): { price: string; type: TriggerType } {
    if (this.triggerPriceValue === undefined) {
      throw ProgramSdkError.missingField("trigger_price");
    }
    if (!this.triggerTypeValue) {
      throw ProgramSdkError.missingField("trigger_type");
    }
    return { price: this.triggerPriceValue, type: this.triggerTypeValue };
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
