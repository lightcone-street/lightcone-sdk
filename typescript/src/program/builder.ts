import { PublicKey, Keypair } from "@solana/web3.js";
import { ProgramSdkError } from "./error";
import { SignedOrder, OrderSide } from "./types";
import { generateSalt, signOrderFull, toSubmitRequest as orderToSubmitRequest } from "./orders";
import {
  scalePriceSize,
  validateRawAmounts,
  validateSignedFields,
  type OrderbookRules,
} from "../shared/scaling";
import type { DepositSource, SubmitOrderRequest } from "../shared";

/**
 * Fluent builder for creating orders.
 * Matches the Rust SDK's OrderBuilder pattern.
 *
 * @example
 * ```typescript
 * const order = new OrderBuilder()
 *   .maker(makerPubkey)
 *   .market(marketPubkey)
 *   .baseMint(baseMintPubkey)
 *   .quoteMint(quoteMintPubkey)
 *   .bid()
 *   .amountIn(1000000n)
 *   .amountOut(500000n)
 *   .expiration(0n)
 *   .build(rules); // fetched from client.orderbooks().decimals(orderbookId)
 * ```
 */
export class OrderBuilder {
  private _salt: bigint | null = null;
  private _maker: PublicKey | null = null;
  private _market: PublicKey | null = null;
  private _baseMint: PublicKey | null = null;
  private _quoteMint: PublicKey | null = null;
  private _side: OrderSide = OrderSide.BID;
  private _amountIn: bigint = 0n;
  private _amountOut: bigint = 0n;
  private _expiration: bigint = 0n;
  private _depositSource?: DepositSource;
  private _rules?: OrderbookRules;
  /** Price and size to scale at build time with the final side and rules. */
  private _priceSize: { price: string; size: string } | null = null;

  /**
   * Set the order's identity salt (any u64). When omitted, the first
   * `build()` draws one and every later build reuses it, so `build()` and
   * `buildAndSign()` describe the same order.
   */
  salt(value: bigint): this {
    this._salt = value;
    return this;
  }

  /** Set the maker (signer) */
  maker(value: PublicKey): this {
    this._maker = value;
    return this;
  }

  /** Set the market */
  market(value: PublicKey): this {
    this._market = value;
    return this;
  }

  /** Set the base mint (conditional token) */
  baseMint(value: PublicKey): this {
    this._baseMint = value;
    return this;
  }

  /** Set the quote mint (payment token) */
  quoteMint(value: PublicKey): this {
    this._quoteMint = value;
    return this;
  }

  /** Set side to BID */
  bid(): this {
    this._side = OrderSide.BID;
    return this;
  }

  /** Set side to ASK */
  ask(): this {
    this._side = OrderSide.ASK;
    return this;
  }

  /** Set the order side */
  side(value: OrderSide): this {
    this._side = value;
    return this;
  }

  /** Set amount in (what maker gives). Replaces any price and size. */
  amountIn(value: bigint): this {
    this._priceSize = null;
    this._amountIn = value;
    return this;
  }

  /** Set amount out (what maker receives). Replaces any price and size. */
  amountOut(value: bigint): this {
    this._priceSize = null;
    this._amountOut = value;
    return this;
  }

  /** Set expiration timestamp (0 = no expiration) */
  expiration(value: bigint): this {
    this._expiration = value;
    return this;
  }

  /** Set deposit source for order submission */
  depositSource(value: DepositSource): this {
    this._depositSource = value;
    return this;
  }

  rules(value: OrderbookRules): this {
    this._rules = value;
    return this;
  }

  /**
   * Set price and size. Every build constructs amountIn and amountOut exactly
   * from them under the trading rules and the side set at build time, so
   * `.price(...).ask()` and `.ask().price(...)` describe the same order.
   * Invalid values are rejected here and at build, never rounded.
   *
   * @param price - Price as a decimal string (e.g., "0.75")
   * @param size - Size as a decimal string (e.g., "100")
   * @param rules - Rules returned by the orderbook decimals endpoint
   */
  price(priceStr: string, sizeStr: string, rules: OrderbookRules): this {
    scalePriceSize(priceStr, sizeStr, this._side, rules);
    this._rules = rules;
    this._priceSize = { price: priceStr, size: sizeStr };
    return this;
  }

  /**
   * Construct exact raw amounts from price/size. Equivalent to `price()`.
   */
  applyScaling(
    priceStr: string,
    sizeStr: string,
    rules: OrderbookRules
  ): this {
    return this.price(priceStr, sizeStr, rules);
  }

  /**
   * Build the unsigned SignedOrder (signature will be all zeros).
   * All required fields must be set.
   */
  build(rules?: OrderbookRules): SignedOrder {
    if (!this._maker) throw ProgramSdkError.missingField("maker");
    if (!this._market) throw ProgramSdkError.missingField("market");
    if (!this._baseMint) throw ProgramSdkError.missingField("baseMint");
    if (!this._quoteMint) throw ProgramSdkError.missingField("quoteMint");

    const resolvedRules = rules ?? this._rules;
    if (!resolvedRules) throw ProgramSdkError.missingField("trading_rules");
    if (this._priceSize) {
      const { amountIn, amountOut } = scalePriceSize(
        this._priceSize.price,
        this._priceSize.size,
        this._side,
        resolvedRules
      );
      this._amountIn = amountIn;
      this._amountOut = amountOut;
    }
    validateSignedFields(this._amountIn, this._amountOut);
    validateRawAmounts(this._amountIn, this._amountOut, this._side, resolvedRules);
    const salt = this._salt ?? generateSalt();
    this._salt = salt;
    return {
      salt,
      maker: this._maker,
      market: this._market,
      baseMint: this._baseMint,
      quoteMint: this._quoteMint,
      side: this._side,
      amountIn: this._amountIn,
      amountOut: this._amountOut,
      expiration: this._expiration,
      signature: Buffer.alloc(64),
    };
  }

  /**
   * Build and sign the order with a Keypair.
   */
  buildAndSign(keypair: Keypair, rules?: OrderbookRules): SignedOrder {
    const resolvedRules = rules ?? this._rules;
    if (!resolvedRules) throw ProgramSdkError.missingField("trading_rules");
    const unsigned = this.build(resolvedRules);
    return signOrderFull(
      {
        salt: unsigned.salt,
        maker: unsigned.maker,
        market: unsigned.market,
        baseMint: unsigned.baseMint,
        quoteMint: unsigned.quoteMint,
        side: unsigned.side,
        amountIn: unsigned.amountIn,
        amountOut: unsigned.amountOut,
        expiration: unsigned.expiration,
      },
      keypair,
      resolvedRules
    );
  }

  /**
   * Build, sign, and convert to a SubmitOrderRequest.
   */
  toSubmitRequest(
    keypair: Keypair,
    orderbookId: string,
    rules?: OrderbookRules
  ): SubmitOrderRequest {
    const signed = this.buildAndSign(keypair, rules);
    return orderToSubmitRequest(signed, orderbookId, {
      depositSource: this._depositSource,
    });
  }
}
