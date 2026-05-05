import { describe, it } from "node:test";
import assert from "node:assert/strict";
import { PublicKey } from "@solana/web3.js";
import {
  ACCOUNT_SIZE,
  ALT_PROGRAM_ID,
  DISCRIMINATOR,
  INSTRUCTION,
  OrderSide,
  buildAddDepositMintIx,
  buildCancelOrderIx,
  buildCreateMarketIx,
  buildCreateOrderbookIx,
  buildDepositAndSwapIx,
  buildDepositToGlobalIx,
  buildDepositToGlobalIxWithAlt,
  buildExtendPositionTokensIx,
  buildMatchOrdersMultiIx,
  buildSetManagerIx,
  buildWithdrawFromGlobalIx,
  deriveConditionId,
  deserializeExchange,
  deserializeOrderStatus,
  getAltPda,
  getConditionTombstonePda,
  getExchangePda,
  getGlobalDepositTokenPda,
  getOrderStatusPda,
  getOrderbookPda,
  getUserNoncePda,
  hashOrder,
  type MakerFill,
  type SignedOrder,
} from "../src/program";

function pubkey(fill: number): PublicKey {
  return new PublicKey(Buffer.alloc(32, fill));
}

function order(seed: number, market = pubkey(30), baseMint = pubkey(40), quoteMint = pubkey(50)): SignedOrder {
  return {
    nonce: seed,
    salt: BigInt(seed + 1),
    maker: pubkey(seed + 10),
    market,
    baseMint,
    quoteMint,
    side: OrderSide.BID,
    amountIn: 100n + BigInt(seed),
    amountOut: 50n + BigInt(seed),
    expiration: 0n,
    signature: Buffer.alloc(64, seed),
  };
}

describe("program authority/account alignment", () => {
  it("deserializes the 120-byte Exchange layout with manager", () => {
    const data = Buffer.alloc(ACCOUNT_SIZE.EXCHANGE);
    DISCRIMINATOR.EXCHANGE.copy(data, 0);
    pubkey(1).toBuffer().copy(data, 8);
    pubkey(2).toBuffer().copy(data, 40);
    pubkey(3).toBuffer().copy(data, 72);
    data.writeBigUInt64LE(42n, 104);
    data[112] = 1;
    data[113] = 7;
    data.writeUInt16LE(5, 114);

    const exchange = deserializeExchange(data);

    assert.equal(exchange.authority.toBase58(), pubkey(1).toBase58());
    assert.equal(exchange.operator.toBase58(), pubkey(2).toBase58());
    assert.equal(exchange.manager.toBase58(), pubkey(3).toBase58());
    assert.equal(exchange.marketCount, 42n);
    assert.equal(exchange.paused, true);
    assert.equal(exchange.bump, 7);
    assert.equal(exchange.depositTokenCount, 5);
  });

  it("deserializes the 32-byte OrderStatus layout with baseRemaining", () => {
    const data = Buffer.alloc(ACCOUNT_SIZE.ORDER_STATUS);
    DISCRIMINATOR.ORDER_STATUS.copy(data, 0);
    data.writeBigUInt64LE(10n, 8);
    data.writeBigUInt64LE(20n, 16);
    data[24] = 1;

    const status = deserializeOrderStatus(data);

    assert.equal(status.remaining, 10n);
    assert.equal(status.baseRemaining, 20n);
    assert.equal(status.isCancelled, true);
  });

  it("derives orderbook PDAs with canonical mint ordering", () => {
    const programId = pubkey(90);
    const [forward] = getOrderbookPda(pubkey(11), pubkey(12), programId);
    const [reverse] = getOrderbookPda(pubkey(12), pubkey(11), programId);

    assert.equal(forward.toBase58(), reverse.toBase58());
  });

  it("builds createMarket with manager and condition tombstone", () => {
    const programId = pubkey(91);
    const params = {
      manager: pubkey(1),
      numOutcomes: 2,
      oracle: pubkey(2),
      questionId: Buffer.alloc(32, 3),
    };
    const conditionId = deriveConditionId(params.oracle, params.questionId, params.numOutcomes);
    const [conditionTombstone] = getConditionTombstonePda(conditionId, programId);

    const ix = buildCreateMarketIx(params, 0n, programId);

    assert.equal(ix.keys.length, 5);
    assert.equal(ix.keys[0]!.pubkey.toBase58(), params.manager.toBase58());
    assert.equal(ix.keys[0]!.isSigner, true);
    assert.equal(ix.keys[4]!.pubkey.toBase58(), conditionTombstone.toBase58());
    assert.equal(ix.keys[4]!.isWritable, true);
  });

  it("builds addDepositMint with manager, readonly market, and global deposit token", () => {
    const programId = pubkey(92);
    const market = pubkey(20);
    const depositMint = pubkey(21);
    const [globalDepositToken] = getGlobalDepositTokenPda(depositMint, programId);

    const ix = buildAddDepositMintIx(
      {
        manager: pubkey(1),
        depositMint,
        outcomeMetadata: [
          { name: "Yes", symbol: "YES", uri: "" },
          { name: "No", symbol: "NO", uri: "" },
        ],
      },
      market,
      2,
      programId
    );

    assert.equal(ix.keys.length, 12);
    assert.equal(ix.keys[0]!.pubkey.toBase58(), pubkey(1).toBase58());
    assert.equal(ix.keys[2]!.pubkey.toBase58(), market.toBase58());
    assert.equal(ix.keys[2]!.isWritable, false);
    assert.equal(ix.keys[9]!.pubkey.toBase58(), globalDepositToken.toBase58());
  });

  it("builds setManager as authority-only role rotation", () => {
    const programId = pubkey(93);
    const authority = pubkey(1);
    const newManager = pubkey(2);
    const [exchange] = getExchangePda(programId);

    const ix = buildSetManagerIx({ authority, newManager }, programId);

    assert.equal(ix.keys.length, 2);
    assert.equal(ix.keys[0]!.pubkey.toBase58(), authority.toBase58());
    assert.equal(ix.keys[0]!.isSigner, true);
    assert.equal(ix.keys[1]!.pubkey.toBase58(), exchange.toBase58());
    assert.equal(ix.data.length, 33);
    assert.equal(ix.data[0], INSTRUCTION.SET_MANAGER);
    assert.deepEqual(ix.data.subarray(1), newManager.toBuffer());
  });

  it("builds cancelOrder as operator-only without system program", () => {
    const programId = pubkey(94);
    const operator = pubkey(1);
    const market = pubkey(2);
    const signedOrder = order(1, market);
    const [exchange] = getExchangePda(programId);
    const [orderStatus] = getOrderStatusPda(hashOrder(signedOrder), programId);

    const ix = buildCancelOrderIx(operator, market, signedOrder, programId);

    assert.equal(ix.keys.length, 4);
    assert.equal(ix.keys[0]!.pubkey.toBase58(), operator.toBase58());
    assert.equal(ix.keys[1]!.pubkey.toBase58(), exchange.toBase58());
    assert.equal(ix.keys[2]!.pubkey.toBase58(), market.toBase58());
    assert.equal(ix.keys[3]!.pubkey.toBase58(), orderStatus.toBase58());
    assert.equal(ix.data.length, 266);
  });

  it("canonicalizes createOrderbook account order while preserving supplied base side", () => {
    const programId = pubkey(95);
    const suppliedMintA = pubkey(22);
    const suppliedMintB = pubkey(11);
    const [orderbook] = getOrderbookPda(suppliedMintA, suppliedMintB, programId);
    const [lookupTable] = getAltPda(orderbook, 123n);

    const ix = buildCreateOrderbookIx(
      {
        manager: pubkey(1),
        market: pubkey(2),
        mintA: suppliedMintA,
        mintB: suppliedMintB,
        mintADepositMint: pubkey(3),
        mintBDepositMint: pubkey(4),
        recentSlot: 123n,
        baseIndex: 0,
        mintAOutcomeIndex: 5,
        mintBOutcomeIndex: 1,
      },
      programId
    );

    assert.equal(ix.keys.length, 11);
    assert.equal(ix.keys[2]!.pubkey.toBase58(), suppliedMintB.toBase58());
    assert.equal(ix.keys[3]!.pubkey.toBase58(), suppliedMintA.toBase58());
    assert.equal(ix.keys[4]!.pubkey.toBase58(), orderbook.toBase58());
    assert.equal(ix.keys[5]!.pubkey.toBase58(), lookupTable.toBase58());
    assert.equal(ix.data.length, 12);
    assert.equal(ix.data[9], 1);
    assert.equal(ix.data[10], 1);
    assert.equal(ix.data[11], 5);
  });

  it("includes orderbook in matchOrdersMulti fixed accounts", () => {
    const programId = pubkey(96);
    const market = pubkey(2);
    const baseMint = pubkey(3);
    const quoteMint = pubkey(4);
    const [orderbook] = getOrderbookPda(baseMint, quoteMint, programId);

    const ix = buildMatchOrdersMultiIx(
      {
        operator: pubkey(1),
        market,
        baseMint,
        quoteMint,
        takerOrder: order(1, market, baseMint, quoteMint),
        makerOrders: [order(2, market, baseMint, quoteMint)],
        makerFillAmounts: [10n],
        takerFillAmounts: [8n],
        fullFillBitmask: 0,
      },
      programId
    );

    assert.equal(ix.keys[3]!.pubkey.toBase58(), orderbook.toBase58());
    assert.equal(ix.data.length, 221);
  });

  it("builds depositToGlobal with exchange and optional user deposit ALT accounts", () => {
    const programId = pubkey(97);
    const user = pubkey(1);
    const mint = pubkey(2);
    const [exchange] = getExchangePda(programId);
    const [userNonce] = getUserNoncePda(user, programId);
    const [lookupTable] = getAltPda(userNonce, 321n);

    const plain = buildDepositToGlobalIx({ user, mint, amount: 100n }, programId);
    assert.equal(plain.keys.length, 8);
    assert.equal(plain.keys[7]!.pubkey.toBase58(), exchange.toBase58());
    assert.equal(plain.data.length, 9);

    const withAlt = buildDepositToGlobalIxWithAlt(
      { user, mint, amount: 100n },
      { kind: "create", recentSlot: 321n },
      programId
    );
    assert.equal(withAlt.keys.length, 11);
    assert.equal(withAlt.keys[8]!.pubkey.toBase58(), userNonce.toBase58());
    assert.equal(withAlt.keys[9]!.pubkey.toBase58(), lookupTable.toBase58());
    assert.equal(withAlt.keys[10]!.pubkey.toBase58(), ALT_PROGRAM_ID.toBase58());
    assert.equal(withAlt.data.length, 17);
  });

  it("builds withdrawFromGlobal with exchange pause-validation account", () => {
    const programId = pubkey(98);
    const [exchange] = getExchangePda(programId);

    const ix = buildWithdrawFromGlobalIx(
      { user: pubkey(1), mint: pubkey(2), amount: 100n },
      programId
    );

    assert.equal(ix.keys.length, 7);
    assert.equal(ix.keys[6]!.pubkey.toBase58(), exchange.toBase58());
  });

  it("includes orderbook in depositAndSwap fixed accounts", () => {
    const programId = pubkey(99);
    const market = pubkey(2);
    const baseMint = pubkey(3);
    const quoteMint = pubkey(4);
    const [orderbook] = getOrderbookPda(baseMint, quoteMint, programId);
    const maker: MakerFill = {
      order: order(2, market, baseMint, quoteMint),
      makerFillAmount: 10n,
      takerFillAmount: 8n,
      isFullFill: true,
      isDeposit: false,
      depositMint: pubkey(5),
    };

    const ix = buildDepositAndSwapIx(
      {
        operator: pubkey(1),
        market,
        baseMint,
        quoteMint,
        takerOrder: order(1, market, baseMint, quoteMint),
        takerIsFullFill: true,
        takerIsDeposit: false,
        takerDepositMint: pubkey(5),
        numOutcomes: 2,
        makers: [maker],
      },
      programId
    );

    assert.equal(ix.keys[3]!.pubkey.toBase58(), orderbook.toBase58());
  });

  it("builds extendPositionTokens with operator signer", () => {
    const programId = pubkey(100);
    const operator = pubkey(1);

    const ix = buildExtendPositionTokensIx(
      {
        operator,
        user: pubkey(2),
        market: pubkey(3),
        lookupTable: pubkey(4),
        depositMints: [pubkey(5)],
      },
      2,
      programId
    );

    assert.equal(ix.keys[0]!.pubkey.toBase58(), operator.toBase58());
    assert.equal(ix.keys[0]!.isSigner, true);
    assert.equal(ix.data[0], INSTRUCTION.EXTEND_POSITION_TOKENS);
  });
});
