import { describe, it } from "node:test";
import assert from "node:assert/strict";
import { PublicKey } from "@solana/web3.js";
import {
  ACCOUNT_SIZE,
  ALT_PROGRAM_ID,
  DISCRIMINATOR,
  INSTRUCTION,
  MarketStatus,
  OrderSide,
  ProgramSdkError,
  buildAddDepositMintIx,
  buildCancelOrderIx,
  buildCloseOrderStatusIx,
  buildCloseOrderbookAltIx,
  buildCloseOrderbookIx,
  buildClosePositionAltIx,
  buildClosePositionTokenAccountsIx,
  buildCreateMarketIx,
  buildCreateOrderbookIx,
  buildDepositIx,
  buildDepositAndSwapIx,
  buildDepositToGlobalIx,
  buildDepositToGlobalIxWithAlt,
  buildExtendPositionTokensIx,
  buildGlobalToMarketDepositIx,
  buildIncrementNonceIx,
  buildMatchOrdersMultiIx,
  buildRedeemWinningsIx,
  buildSetManagerIx,
  buildSettleMarketIx,
  buildWithdrawFromPositionIx,
  buildWithdrawFromGlobalIx,
  deriveConditionId,
  deserializeExchange,
  deserializeMarket,
  deserializeOrderStatus,
  getConditionalMintPda,
  getConditionalTokenAta,
  getAltPda,
  getConditionTombstonePda,
  getDepositTokenAta,
  getExchangePda,
  getGlobalDepositTokenPda,
  getMarketPda,
  getMintAuthorityPda,
  getOrderStatusPda,
  getOrderbookPda,
  getPositionPda,
  getUserNoncePda,
  getVaultPda,
  hashOrder,
  scalarToPayoutNumerators,
  winnerTakesAllPayoutNumerators,
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

  it("deserializes the 148-byte Market payout-vector layout", () => {
    const data = Buffer.alloc(ACCOUNT_SIZE.MARKET);
    const questionId = Buffer.alloc(32, 4);
    const conditionId = Buffer.alloc(32, 5);
    DISCRIMINATOR.MARKET.copy(data, 0);
    data.writeBigUInt64LE(42n, 8);
    data[16] = 3;
    data[17] = MarketStatus.Resolved;
    data[18] = 9;
    pubkey(6).toBuffer().copy(data, 24);
    questionId.copy(data, 56);
    conditionId.copy(data, 88);
    [1, 2, 3, 0, 0, 0].forEach((value, index) => {
      data.writeUInt32LE(value, 120 + index * 4);
    });
    data.writeUInt32LE(6, 144);

    const market = deserializeMarket(data);

    assert.equal(market.marketId, 42n);
    assert.equal(market.numOutcomes, 3);
    assert.equal(market.status, MarketStatus.Resolved);
    assert.equal(market.bump, 9);
    assert.equal(market.oracle.toBase58(), pubkey(6).toBase58());
    assert.deepEqual(market.questionId, questionId);
    assert.deepEqual(market.conditionId, conditionId);
    assert.deepEqual(market.payoutNumerators, [1, 2, 3, 0, 0, 0]);
    assert.equal(market.payoutDenominator, 6);
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

  it("builds mintCompleteSet without obsolete position collateral ATA", () => {
    const programId = pubkey(104);
    const user = pubkey(1);
    const market = pubkey(2);
    const depositMint = pubkey(3);
    const [mintAuthority] = getMintAuthorityPda(market, programId);

    const ix = buildDepositIx(
      { user, market, depositMint, amount: 100n },
      3,
      programId
    );

    assert.equal(ix.keys.length, 18);
    assert.equal(ix.keys[7]!.pubkey.toBase58(), mintAuthority.toBase58());
    assert.equal(ix.keys[7]!.isWritable, false);
    assert.equal(ix.data[0], INSTRUCTION.MINT_COMPLETE_SET);
  });

  it("builds incrementNonce with exchange pause-validation account", () => {
    const programId = pubkey(105);
    const user = pubkey(1);
    const [exchange] = getExchangePda(programId);

    const ix = buildIncrementNonceIx(user, programId);

    assert.equal(ix.keys.length, 4);
    assert.equal(ix.keys[3]!.pubkey.toBase58(), exchange.toBase58());
    assert.equal(ix.keys[3]!.isWritable, false);
    assert.equal(ix.data[0], INSTRUCTION.INCREMENT_NONCE);
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

  it("builds settleMarket with readonly oracle signer and u32 payout vector", () => {
    const programId = pubkey(101);
    const oracle = pubkey(1);
    const marketId = 7n;
    const [exchange] = getExchangePda(programId);
    const [market] = getMarketPda(marketId, programId);

    const ix = buildSettleMarketIx(
      { oracle, marketId, payoutNumerators: [7, 3] },
      programId
    );

    assert.equal(ix.keys.length, 3);
    assert.equal(ix.keys[0]!.pubkey.toBase58(), oracle.toBase58());
    assert.equal(ix.keys[0]!.isSigner, true);
    assert.equal(ix.keys[0]!.isWritable, false);
    assert.equal(ix.keys[1]!.pubkey.toBase58(), exchange.toBase58());
    assert.equal(ix.keys[2]!.pubkey.toBase58(), market.toBase58());
    assert.equal(ix.keys[2]!.isWritable, true);
    assert.equal(ix.data.length, 9);
    assert.equal(ix.data[0], INSTRUCTION.SETTLE_MARKET);
    assert.equal(ix.data.readUInt32LE(1), 7);
    assert.equal(ix.data.readUInt32LE(5), 3);
  });

  it("rejects invalid settle payout vectors before serialization", () => {
    const params = { oracle: pubkey(1), marketId: 1n };

    assert.throws(
      () => buildSettleMarketIx({ ...params, payoutNumerators: [0, 0] }, pubkey(102)),
      (error) =>
        error instanceof ProgramSdkError &&
        error.variant === "InvalidPayoutNumerators"
    );
    assert.throws(
      () => buildSettleMarketIx({ ...params, payoutNumerators: [1] }, pubkey(102)),
      (error) =>
        error instanceof ProgramSdkError &&
        error.variant === "InvalidOutcomeCount"
    );
    assert.throws(
      () => buildSettleMarketIx({ ...params, payoutNumerators: [0xffffffff, 1] }, pubkey(102)),
      (error) =>
        error instanceof ProgramSdkError &&
        error.variant === "Overflow"
    );
  });

  it("builds winner-takes-all and scalar payout vectors", () => {
    assert.deepEqual(winnerTakesAllPayoutNumerators(2, 4), [0, 0, 1, 0]);
    assert.deepEqual(
      scalarToPayoutNumerators({
        minValue: 0n,
        maxValue: 100n,
        resolvedValue: 25n,
        lowerOutcomeIndex: 0,
        upperOutcomeIndex: 1,
        numOutcomes: 2,
      }),
      [3, 1]
    );
    assert.deepEqual(
      scalarToPayoutNumerators({
        minValue: 0n,
        maxValue: 100n,
        resolvedValue: -5n,
        lowerOutcomeIndex: 0,
        upperOutcomeIndex: 1,
        numOutcomes: 2,
      }),
      [1, 0]
    );
    assert.deepEqual(
      scalarToPayoutNumerators({
        minValue: 0n,
        maxValue: 100n,
        resolvedValue: 120n,
        lowerOutcomeIndex: 0,
        upperOutcomeIndex: 1,
        numOutcomes: 2,
      }),
      [0, 1]
    );
    assert.deepEqual(
      scalarToPayoutNumerators({
        minValue: -10_000n,
        maxValue: 40_000n,
        resolvedValue: 15_250n,
        lowerOutcomeIndex: 0,
        upperOutcomeIndex: 1,
        numOutcomes: 2,
      }),
      [99, 101]
    );
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

  it("builds redeemWinnings with outcome index and exchange pause-validation account", () => {
    const programId = pubkey(103);
    const user = pubkey(1);
    const market = pubkey(2);
    const depositMint = pubkey(3);
    const outcomeIndex = 2;
    const [exchange] = getExchangePda(programId);
    const [vault] = getVaultPda(depositMint, market, programId);
    const [conditionalMint] = getConditionalMintPda(market, depositMint, outcomeIndex, programId);
    const [position] = getPositionPda(user, market, programId);
    const positionConditionalAta = getConditionalTokenAta(conditionalMint, position);
    const userDepositAta = getDepositTokenAta(depositMint, user);
    const [mintAuthority] = getMintAuthorityPda(market, programId);

    const ix = buildRedeemWinningsIx(
      { user, market, depositMint, amount: 123n },
      outcomeIndex,
      programId
    );

    assert.equal(ix.keys.length, 12);
    assert.equal(ix.keys[3]!.pubkey.toBase58(), vault.toBase58());
    assert.equal(ix.keys[4]!.pubkey.toBase58(), conditionalMint.toBase58());
    assert.equal(ix.keys[5]!.pubkey.toBase58(), position.toBase58());
    assert.equal(ix.keys[5]!.isWritable, false);
    assert.equal(ix.keys[6]!.pubkey.toBase58(), positionConditionalAta.toBase58());
    assert.equal(ix.keys[7]!.pubkey.toBase58(), userDepositAta.toBase58());
    assert.equal(ix.keys[8]!.pubkey.toBase58(), mintAuthority.toBase58());
    assert.equal(ix.keys[11]!.pubkey.toBase58(), exchange.toBase58());
    assert.equal(ix.data.length, 10);
    assert.equal(ix.data[0], INSTRUCTION.REDEEM_WINNINGS);
    assert.equal(ix.data.readBigUInt64LE(1), 123n);
    assert.equal(ix.data[9], outcomeIndex);
  });

  it("builds withdrawFromPosition with exchange pause-validation account", () => {
    const programId = pubkey(106);
    const [exchange] = getExchangePda(programId);

    const ix = buildWithdrawFromPositionIx(
      {
        user: pubkey(1),
        market: pubkey(2),
        mint: pubkey(3),
        amount: 100n,
        outcomeIndex: 1,
      },
      true,
      programId
    );

    assert.equal(ix.keys.length, 8);
    assert.equal(ix.keys[7]!.pubkey.toBase58(), exchange.toBase58());
    assert.equal(ix.keys[7]!.isWritable, false);
    assert.equal(ix.data[0], INSTRUCTION.WITHDRAW_FROM_POSITION);
  });

  it("builds globalToMarketDeposit without obsolete position collateral ATA", () => {
    const programId = pubkey(107);
    const user = pubkey(1);
    const market = pubkey(2);
    const depositMint = pubkey(3);
    const [mintAuthority] = getMintAuthorityPda(market, programId);

    const ix = buildGlobalToMarketDepositIx(
      { user, market, depositMint, amount: 100n },
      3,
      programId
    );

    assert.equal(ix.keys.length, 19);
    assert.equal(ix.keys[8]!.pubkey.toBase58(), mintAuthority.toBase58());
    assert.equal(ix.keys[8]!.isWritable, false);
    assert.equal(ix.data[0], INSTRUCTION.GLOBAL_TO_MARKET_DEPOSIT);
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

  it("builds closeOrderStatus with order hash payload", () => {
    const programId = pubkey(108);
    const operator = pubkey(1);
    const orderHash = Buffer.alloc(32, 9);
    const [orderStatus] = getOrderStatusPda(orderHash, programId);

    const ix = buildCloseOrderStatusIx({ operator, orderHash }, programId);

    assert.equal(ix.keys.length, 3);
    assert.equal(ix.keys[0]!.pubkey.toBase58(), operator.toBase58());
    assert.equal(ix.keys[2]!.pubkey.toBase58(), orderStatus.toBase58());
    assert.equal(ix.data.length, 33);
    assert.equal(ix.data[0], INSTRUCTION.CLOSE_ORDER_STATUS);
    assert.deepEqual(ix.data.subarray(1), orderHash);
  });

  it("builds closePositionTokenAccounts with grouped conditional ATAs", () => {
    const programId = pubkey(109);

    const ix = buildClosePositionTokenAccountsIx(
      {
        operator: pubkey(1),
        market: pubkey(2),
        position: pubkey(3),
        depositMints: [pubkey(4)],
      },
      3,
      programId
    );

    assert.equal(ix.keys.length, 12);
    assert.equal(ix.data.length, 1);
    assert.equal(ix.data[0], INSTRUCTION.CLOSE_POSITION_TOKEN_ACCOUNTS);
  });

  it("builds close ALT and orderbook cleanup instructions", () => {
    const programId = pubkey(110);
    const operator = pubkey(1);
    const market = pubkey(2);
    const lookupTable = pubkey(3);
    const position = pubkey(4);
    const orderbook = pubkey(5);

    const positionAltIx = buildClosePositionAltIx(
      { operator, position, market, lookupTable },
      programId
    );
    assert.equal(positionAltIx.keys.length, 6);
    assert.equal(positionAltIx.data[0], INSTRUCTION.CLOSE_POSITION_ALT);

    const orderbookAltIx = buildCloseOrderbookAltIx(
      { operator, orderbook, market, lookupTable },
      programId
    );
    assert.equal(orderbookAltIx.keys.length, 6);
    assert.equal(orderbookAltIx.data[0], INSTRUCTION.CLOSE_ORDERBOOK_ALT);

    const closeOrderbookIx = buildCloseOrderbookIx(
      { operator, orderbook, market, lookupTable },
      programId
    );
    assert.equal(closeOrderbookIx.keys.length, 5);
    assert.equal(closeOrderbookIx.data[0], INSTRUCTION.CLOSE_ORDERBOOK);
  });
});
