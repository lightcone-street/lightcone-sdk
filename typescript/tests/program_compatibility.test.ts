import { it } from "node:test";
import assert from "node:assert/strict";
import { Keypair, PublicKey } from "@solana/web3.js";
import {
  INITIALIZE_AUTHORITY, MAX_MAKERS, OrderSide, ProgramSdkError,
  buildCreateMarketIx, buildSetOracleIx, buildSetPausedIx,
  buildInitPositionTokensIx,
  buildMatchOrdersMultiIx, buildDepositAndSwapIx, type SignedOrder,
  deserializeOrderbook, deserializeGlobalDepositToken,
  getOrderbookBaseDepositMint, getOrderbookQuoteDepositMint,
} from "../src/program";

const wallet = (seed: number): PublicKey => Keypair.fromSeed(Buffer.alloc(32, seed)).publicKey;

it("exports the program initializer and eleven-maker limit", () => {
  assert.equal(MAX_MAKERS, 11);
  assert.equal(INITIALIZE_AUTHORITY.toBase58(), "3vYRAzr5X41hrmKMnDCoQJJmPH89S4LLwmFpk8UtwCqr");
});

for(const deposit of [false, true]) {
  it(`preserves exact fills at the eleven-maker boundary (deposit=${deposit})`, () => {
    const market = wallet(3), baseMint = wallet(4), quoteMint = wallet(5);
    const orders: SignedOrder[] = Array.from({ length: 13 }, (_, i) => ({
      nonce: i, salt: BigInt(i), maker: wallet(i + 10), market, baseMint, quoteMint,
      side: i === 0 ? OrderSide.BID : OrderSide.ASK,
      amountIn: 2n ** 63n + 11n, amountOut: 2n ** 53n + 7n,
      expiration: 0n, signature: Buffer.alloc(64, i),
    }));
    const common = { operator: wallet(1), market, baseMint, quoteMint, feeReceiver: wallet(2), baseDepositMint: wallet(70), quoteDepositMint: wallet(71), takerOrder: orders[0]! };
    const makerAmount = 2n ** 53n + 7n, takerAmount = 2n ** 63n + 11n;
    const build = (count: number) => deposit ? buildDepositAndSwapIx({
      ...common, takerIsFullFill: true, takerIsDeposit: false, takerDepositMint: baseMint, numOutcomes: 2,
      makers: orders.slice(1, count + 1).map(order => ({ order, makerFillAmount: makerAmount, takerFillAmount: takerAmount, isFullFill: true, isDeposit: false, depositMint: baseMint })),
    }) : buildMatchOrdersMultiIx({
      ...common, makerOrders: orders.slice(1, count + 1), makerFillAmounts: Array(count).fill(makerAmount), takerFillAmounts: Array(count).fill(takerAmount), fullFillBitmask: 0x87ff,
    });
    const data = build(11).data;
    assert.equal(data[0], deposit ? 20 : 13);
    assert.equal(data[102], 11);
    assert.equal(data.readUInt16LE(103), 0x87ff);
    if(deposit) assert.equal(data.readUInt16LE(105), 0);
    for(let i = 0;i < 11;i++) {
      const offset = (deposit ? 107 : 105) + i * 117;
      assert.deepEqual(data.subarray(offset + 37, offset + 101), orders[i + 1]!.signature);
      assert.equal(data.readBigUInt64LE(offset + 101), makerAmount);
      assert.equal(data.readBigUInt64LE(offset + 109), takerAmount);
    }
    assert.throws(() => build(12), (e: unknown) => e instanceof ProgramSdkError && e.variant === "TooManyMakers" && e.message.includes("12"));
  });
}

it("rejects zero and PDA oracles for creation and rotation", () => {
  for(const oracle of [PublicKey.default, PublicKey.findProgramAddressSync([Buffer.from("oracle")], wallet(9))[0]]) {
    const invalid = (e: unknown) => e instanceof ProgramSdkError && e.variant === "InvalidOracle";
    assert.throws(() => buildCreateMarketIx({ manager: wallet(1), numOutcomes: 2, oracle, questionId: Buffer.alloc(32), makerFeeBps: 0, takerFeeBps: 0 }, 0n), invalid);
    assert.throws(() => buildSetOracleIx({ authority: wallet(1), market: wallet(2), newOracle: oracle }), invalid);
  }
});

it("rejects zero and PDA beneficiaries while accepting PDA governance authorities", () => {
  const [user] = PublicKey.findProgramAddressSync([Buffer.from("user")], wallet(9));
  for (const beneficiary of [PublicKey.default, user]) {
    const params = { payer: wallet(1), user: beneficiary, market: wallet(2), depositMints: [wallet(3)] };
    const invalid = (e: unknown) => e instanceof ProgramSdkError && e.variant === "InvalidPubkey" && e.message.includes(beneficiary.toBase58());
    assert.throws(() => buildInitPositionTokensIx(params, 2), invalid);
  }
  const ix = buildSetPausedIx(user, true);
  assert.ok(ix.keys[0]!.pubkey.equals(user));
  assert.ok(ix.keys[0]!.isSigner);
  assert.deepEqual(buildSetOracleIx({ authority: user, market: wallet(2), newOracle: wallet(1) }).data.subarray(1), wallet(1).toBuffer());
});

function orderbookAccountBytes(baseIndex = 0): Buffer {
  const data = Buffer.alloc(176);
  Buffer.from("2b221971c3454807", "hex").copy(data, 0);
  [8, 40, 72, 104, 136].forEach((offset, index) => {
    data.fill(index + 1, offset, offset + 32);
  });
  data[168] = baseIndex;
  data[169] = 5;
  data[170] = 247;
  return data;
}

function globalDepositTokenAccountBytes(active = 0): Buffer {
  const data = Buffer.alloc(47);
  Buffer.from("25bea1e87b922a57", "hex").copy(data, 0);
  data.fill(6, 8, 40);
  data[40] = 251;
  data.writeUInt16LE(0x1234, 41);
  data[43] = active;
  return data;
}

const programError = (variant: ProgramSdkError["variant"]) => (error: unknown): boolean =>
  error instanceof ProgramSdkError && error.variant === variant;

for (const baseIndex of [0, 1]) {
  it(`decodes every 176-byte Orderbook field and collateral orientation (baseIndex=${baseIndex})`, () => {
    const book = deserializeOrderbook(orderbookAccountBytes(baseIndex));
    assert.equal(book.discriminator.toString("hex"), "2b221971c3454807");
    const fields = ["market", "mintA", "mintB", "depositMintA", "depositMintB"] as const;
    fields.forEach((field, index) => {
      assert.ok(book[field].equals(new PublicKey(Buffer.alloc(32, index + 1))), field);
    });
    assert.equal(book.baseIndex, baseIndex);
    assert.equal(book.outcomeIndex, 5);
    assert.equal(book.bump, 247);
    assert.ok(getOrderbookBaseDepositMint(book).equals(new PublicKey(Buffer.alloc(32, baseIndex === 0 ? 4 : 5))));
    assert.ok(getOrderbookQuoteDepositMint(book).equals(new PublicKey(Buffer.alloc(32, baseIndex === 0 ? 5 : 4))));
  });
}

for (const length of [144, 175, 177]) {
  it(`rejects a ${length}-byte Orderbook account`, () => {
    const data = Buffer.alloc(length);
    orderbookAccountBytes().copy(data);
    assert.throws(() => deserializeOrderbook(data), programError("InvalidDataLength"));
  });
}

it("rejects an Orderbook account with the wrong discriminator", () => {
  const data = orderbookAccountBytes();
  data[0] ^= 0xff;
  assert.throws(() => deserializeOrderbook(data), programError("InvalidDiscriminator"));
});

for (const baseIndex of [2, 255]) {
  it(`rejects Orderbook base index ${baseIndex}`, () => {
    assert.throws(() => deserializeOrderbook(orderbookAccountBytes(baseIndex)), programError("InvalidOrderbook"));
  });
}

for (const outcomeIndex of [6, 255]) {
  it(`rejects Orderbook outcome index ${outcomeIndex}`, () => {
    const data = orderbookAccountBytes();
    data[169] = outcomeIndex;
    assert.throws(() => deserializeOrderbook(data), (error: unknown) => {
      assert.ok(error instanceof ProgramSdkError);
      assert.equal(error.variant, "InvalidOutcomeIndex");
      assert.equal(error.message, `Invalid outcome index: ${outcomeIndex}. Must be between 0 and 5.`);
      return true;
    });
  });
}

for (const active of [0, 1]) {
  it(`decodes every 47-byte GlobalDepositToken field with active=${active}`, () => {
    const token = deserializeGlobalDepositToken(globalDepositTokenAccountBytes(active));
    assert.equal(token.discriminator.toString("hex"), "25bea1e87b922a57");
    assert.ok(token.mint.equals(new PublicKey(Buffer.alloc(32, 6))));
    assert.equal(token.bump, 251);
    assert.equal(token.index, 0x1234);
    assert.equal(token.active, active === 1);
  });
}

for (const active of [2, 255]) {
  it(`rejects the nonboolean GlobalDepositToken activity byte ${active}`, () => {
    assert.throws(() => deserializeGlobalDepositToken(globalDepositTokenAccountBytes(active)), programError("Serialization"));
  });
}

for (const length of [46, 48]) {
  it(`rejects a ${length}-byte GlobalDepositToken account`, () => {
    const data = Buffer.alloc(length);
    globalDepositTokenAccountBytes().copy(data);
    assert.throws(() => deserializeGlobalDepositToken(data), programError("InvalidDataLength"));
  });
}

it("rejects a GlobalDepositToken account with the wrong discriminator", () => {
  const data = globalDepositTokenAccountBytes();
  data[0] ^= 0xff;
  assert.throws(() => deserializeGlobalDepositToken(data), programError("InvalidDiscriminator"));
});
