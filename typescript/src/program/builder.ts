import { PublicKey, Keypair } from "@solana/web3.js";
import { SignedOrder, OrderSide } from "./types";
import { signOrderFull, signatureHex } from "./orders";
import { scalePriceSize, OrderbookDecimals } from "../shared/scaling";

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
 *   .makerAmount(1000000n)
 *   .takerAmount(500000n)
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
  private _makerAmount: bigint = 0n;
  private _takerAmount: bigint = 0n;
  private _expiration: bigint = 0n;

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

  /** Set maker amount (what maker gives) */
  makerAmount(value: bigint): this {
    this._makerAmount = value;
    return this;
  }

  /** Set taker amount (what maker receives) */
  takerAmount(value: bigint): this {
    this._takerAmount = value;
    return this;
  }

  /** Set expiration timestamp (0 = no expiration) */
  expiration(value: bigint): this {
    this._expiration = value;
    return this;
  }

  /**
   * Set price and size, auto-computing maker_amount and taker_amount using decimal scaling.
   *
   * @param price - Price as a decimal string (e.g., "0.75")
   * @param size - Size as a decimal string (e.g., "100")
   * @param decimals - Orderbook decimal configuration
   */
  price(priceStr: string, sizeStr: string, decimals: OrderbookDecimals): this {
    const { makerAmount, takerAmount } = scalePriceSize(
      priceStr,
      sizeStr,
      this._side,
      decimals
    );
    this._makerAmount = makerAmount;
    this._takerAmount = takerAmount;
    return this;
  }

  /**
   * Apply decimal scaling to convert price/size to maker_amount/taker_amount.
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
      makerAmount: this._makerAmount,
      takerAmount: this._takerAmount,
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
        makerAmount: unsigned.makerAmount,
        takerAmount: unsigned.takerAmount,
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
  ): {
    maker: string;
    nonce: string;
    market_pubkey: string;
    base_token: string;
    quote_token: string;
    side: number;
    maker_amount: string;
    taker_amount: string;
    expiration: number;
    signature: string;
    orderbook_id: string;
  } {
    const signed = this.buildAndSign(keypair);
    return {
      maker: signed.maker.toBase58(),
      nonce: signed.nonce.toString(),
      market_pubkey: signed.market.toBase58(),
      base_token: signed.baseMint.toBase58(),
      quote_token: signed.quoteMint.toBase58(),
      side: signed.side,
      maker_amount: signed.makerAmount.toString(),
      taker_amount: signed.takerAmount.toString(),
      expiration: Number(signed.expiration),
      signature: signatureHex(signed),
      orderbook_id: orderbookId,
    };
  }
}
