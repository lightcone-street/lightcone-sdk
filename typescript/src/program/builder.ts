import { PublicKey, Keypair } from "@solana/web3.js";
import { SignedOrder, OrderSide } from "./types";
import { signOrderFull, toSubmitRequest as orderToSubmitRequest } from "./orders";
import { scalePriceSize, OrderbookDecimals } from "../shared/scaling";
import type { DepositSource, SubmitOrderRequest } from "../shared";

/**
 * Fluent builder for creating orders.
 * Matches the Rust SDK's OrderBuilder pattern.
 *
 * @example
 * ```typescript
 * const order = new OrderBuilder()
 *   .nonce(1)
 *   .maker(makerPubkey)
 *   .market(marketPubkey)
 *   .baseMint(baseMintPubkey)
 *   .quoteMint(quoteMintPubkey)
 *   .bid()
 *   .amountIn(1000000n)
 *   .amountOut(500000n)
 *   .expiration(0n)
 *   .build();
 * ```
 */
export class OrderBuilder {
  private _nonce: number = 0;
  private _maker: PublicKey | null = null;
  private _market: PublicKey | null = null;
  private _baseMint: PublicKey | null = null;
  private _quoteMint: PublicKey | null = null;
  private _side: OrderSide = OrderSide.BID;
  private _amountIn: bigint = 0n;
  private _amountOut: bigint = 0n;
  private _expiration: bigint = 0n;
  private _depositSource?: DepositSource;

  /** Set the order nonce (u32) */
  nonce(value: number): this {
    this._nonce = value;
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

  /** Set amount in (what maker gives) */
  amountIn(value: bigint): this {
    this._amountIn = value;
    return this;
  }

  /** Set amount out (what maker receives) */
  amountOut(value: bigint): this {
    this._amountOut = value;
    return this;
  }

  /** @deprecated Use amountIn() */
  makerAmount(value: bigint): this {
    return this.amountIn(value);
  }

  /** @deprecated Use amountOut() */
  takerAmount(value: bigint): this {
    return this.amountOut(value);
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

  /**
   * Set price and size, auto-computing amountIn and amountOut using decimal scaling.
   *
   * @param price - Price as a decimal string (e.g., "0.75")
   * @param size - Size as a decimal string (e.g., "100")
   * @param decimals - Orderbook decimal configuration
   */
  price(priceStr: string, sizeStr: string, decimals: OrderbookDecimals): this {
    const { amountIn, amountOut } = scalePriceSize(
      priceStr,
      sizeStr,
      this._side,
      decimals
    );
    this._amountIn = amountIn;
    this._amountOut = amountOut;
    return this;
  }

  /**
   * Apply decimal scaling to convert price/size to amountIn/amountOut.
   * Equivalent to calling price() but with separate method name for clarity.
   */
  applyScaling(
    priceStr: string,
    sizeStr: string,
    decimals: OrderbookDecimals
  ): this {
    return this.price(priceStr, sizeStr, decimals);
  }

  /**
   * Build the unsigned SignedOrder (signature will be all zeros).
   * All required fields must be set.
   */
  build(): SignedOrder {
    if (!this._maker) throw new Error("OrderBuilder: maker is required");
    if (!this._market) throw new Error("OrderBuilder: market is required");
    if (!this._baseMint) throw new Error("OrderBuilder: baseMint is required");
    if (!this._quoteMint) throw new Error("OrderBuilder: quoteMint is required");

    return {
      nonce: this._nonce,
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
  buildAndSign(keypair: Keypair): SignedOrder {
    const unsigned = this.build();
    return signOrderFull(
      {
        nonce: unsigned.nonce,
        maker: unsigned.maker,
        market: unsigned.market,
        baseMint: unsigned.baseMint,
        quoteMint: unsigned.quoteMint,
        side: unsigned.side,
        amountIn: unsigned.amountIn,
        amountOut: unsigned.amountOut,
        expiration: unsigned.expiration,
      },
      keypair
    );
  }

  /**
   * Build, sign, and convert to a SubmitOrderRequest.
   */
  toSubmitRequest(
    keypair: Keypair,
    orderbookId: string
  ): SubmitOrderRequest {
    const signed = this.buildAndSign(keypair);
    return orderToSubmitRequest(signed, orderbookId, {
      depositSource: this._depositSource,
    });
  }
}
