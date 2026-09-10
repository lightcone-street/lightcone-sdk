import { it } from "node:test";
import assert from "node:assert/strict";
import { Keypair, PublicKey } from "@solana/web3.js";
import {
  INITIALIZE_AUTHORITY, MAX_MAKERS, OrderSide, ProgramSdkError,
  buildCreateMarketIx, buildSetOracleIx, buildSetPausedIx,
  buildInitPositionTokensIx,
  buildMatchOrdersMultiIx, buildDepositAndSwapIx, type SignedOrder,
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
