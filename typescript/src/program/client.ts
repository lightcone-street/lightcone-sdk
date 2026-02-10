import {
  Connection,
  PublicKey,
  Transaction,
  Keypair,
} from "@solana/web3.js";
import { PROGRAM_ID } from "./constants";
import {
  Exchange,
  Market,
  Position,
  OrderStatus,
  Orderbook,
  BuildResult,
  InitializeParams,
  CreateMarketParams,
  AddDepositMintParams,
  MintCompleteSetParams,
  MergeCompleteSetParams,
  SettleMarketParams,
  RedeemWinningsParams,
  WithdrawFromPositionParams,
  ActivateMarketParams,
  MatchOrdersMultiParams,
  SetAuthorityParams,
  CreateOrderbookParams,
  SignedOrder,
  BidOrderParams,
  AskOrderParams,
  InitializeAccounts,
  CreateMarketAccounts,
  AddDepositMintAccounts,
  MintCompleteSetAccounts,
  MergeCompleteSetAccounts,
  CancelOrderAccounts,
  IncrementNonceAccounts,
  SettleMarketAccounts,
  RedeemWinningsAccounts,
  ActivateMarketAccounts,
  MatchOrdersMultiAccounts,
} from "./types";
import * as pda from "./pda";
import {
  deserializeExchange,
  deserializeMarket,
  deserializePosition,
  deserializeOrderStatus,
  deserializeUserNonce,
  deserializeOrderbook,
} from "./accounts";
import {
  buildInitializeIx,
  buildCreateMarketIx,
  buildAddDepositMintIx,
  buildMintCompleteSetIx,
  buildMergeCompleteSetIx,
  buildCancelOrderIx,
  buildIncrementNonceIx,
  buildSettleMarketIx,
  buildRedeemWinningsIx,
  buildSetPausedIx,
  buildSetOperatorIx,
  buildWithdrawFromPositionIx,
  buildActivateMarketIx,
  buildMatchOrdersMultiIx,
  buildSetAuthorityIx,
  buildCreateOrderbookIx,
} from "./instructions";
import {
  hashOrder,
  signOrder,
  createBidOrder,
  createAskOrder,
  signOrderFull,
} from "./orders";
import { deriveConditionId } from "./utils";

/**
 * LightconePinocchioClient - SDK for interacting with the Lightcone Pinocchio program
 */
export class LightconePinocchioClient {
  readonly connection: Connection;
  readonly programId: PublicKey;
  readonly pda = pda;

  constructor(connection: Connection, programId: PublicKey = PROGRAM_ID) {
    this.connection = connection;
    this.programId = programId;
  }

  // ============================================================================
  // ACCOUNT FETCHERS
  // ============================================================================

  async getExchange(): Promise<Exchange> {
    const [exchangePda] = pda.getExchangePda(this.programId);
    const accountInfo = await this.connection.getAccountInfo(exchangePda);
    if (!accountInfo) {
      throw new Error("Exchange account not found. Protocol not initialized.");
    }
    return deserializeExchange(accountInfo.data as Buffer);
  }

  async getMarket(marketId: bigint): Promise<Market> {
    const [marketPda] = pda.getMarketPda(marketId, this.programId);
    const accountInfo = await this.connection.getAccountInfo(marketPda);
    if (!accountInfo) {
      throw new Error(`Market ${marketId} not found`);
    }
    return deserializeMarket(accountInfo.data as Buffer);
  }

  async getMarketByPubkey(market: PublicKey): Promise<Market> {
    const accountInfo = await this.connection.getAccountInfo(market);
    if (!accountInfo) {
      throw new Error(`Market not found at ${market.toBase58()}`);
    }
    return deserializeMarket(accountInfo.data as Buffer);
  }

  async getPosition(
    owner: PublicKey,
    market: PublicKey
  ): Promise<Position | null> {
    const [positionPda] = pda.getPositionPda(owner, market, this.programId);
    const accountInfo = await this.connection.getAccountInfo(positionPda);
    if (!accountInfo) {
      return null;
    }
    return deserializePosition(accountInfo.data as Buffer);
  }

  async getOrderStatus(orderHash: Buffer): Promise<OrderStatus | null> {
    const [orderStatusPda] = pda.getOrderStatusPda(orderHash, this.programId);
    const accountInfo = await this.connection.getAccountInfo(orderStatusPda);
    if (!accountInfo) {
      return null;
    }
    return deserializeOrderStatus(accountInfo.data as Buffer);
  }

  async getUserNonce(user: PublicKey): Promise<bigint> {
    const [userNoncePda] = pda.getUserNoncePda(user, this.programId);
    const accountInfo = await this.connection.getAccountInfo(userNoncePda);
    if (!accountInfo) {
      return 0n;
    }
    const nonce = deserializeUserNonce(accountInfo.data as Buffer);
    return nonce.nonce;
  }

  async getNextNonce(user: PublicKey): Promise<number> {
    const nonce = await this.getUserNonce(user);
    if (nonce > 0xFFFFFFFFn) {
      throw new Error(`Nonce exceeds u32 range: ${nonce}`);
    }
    return Number(nonce);
  }

  async getNextMarketId(): Promise<bigint> {
    const exchange = await this.getExchange();
    return exchange.marketCount;
  }

  async getOrderbook(
    mintA: PublicKey,
    mintB: PublicKey
  ): Promise<Orderbook | null> {
    const [orderbookPda] = pda.getOrderbookPda(mintA, mintB, this.programId);
    const accountInfo = await this.connection.getAccountInfo(orderbookPda);
    if (!accountInfo) {
      return null;
    }
    return deserializeOrderbook(accountInfo.data as Buffer);
  }

  // ============================================================================
  // TRANSACTION BUILDERS
  // ============================================================================

  private async createBuildResult<T>(
    feePayer: PublicKey,
    accounts: T,
    ...instructions: Parameters<Transaction["add"]>
  ): Promise<BuildResult<T>> {
    const { blockhash, lastValidBlockHeight } =
      await this.connection.getLatestBlockhash();

    const transaction = new Transaction({
      feePayer,
      blockhash,
      lastValidBlockHeight,
    });

    for (const ix of instructions) {
      transaction.add(ix);
    }

    return {
      transaction,
      accounts,
      serialize: () =>
        transaction
          .serialize({ requireAllSignatures: false, verifySignatures: false })
          .toString("base64"),
    };
  }

  async initialize(
    params: InitializeParams
  ): Promise<BuildResult<InitializeAccounts>> {
    const [exchange] = pda.getExchangePda(this.programId);
    const ix = buildInitializeIx(params, this.programId);
    return this.createBuildResult(params.authority, { exchange }, ix);
  }

  async createMarket(
    params: CreateMarketParams
  ): Promise<BuildResult<CreateMarketAccounts>> {
    const marketId = await this.getNextMarketId();
    const [exchange] = pda.getExchangePda(this.programId);
    const [market] = pda.getMarketPda(marketId, this.programId);
    const ix = buildCreateMarketIx(params, marketId, this.programId);
    return this.createBuildResult(params.authority, { exchange, market }, ix);
  }

  async addDepositMint(
    params: AddDepositMintParams,
    numOutcomes: number
  ): Promise<BuildResult<AddDepositMintAccounts>> {
    const [market] = pda.getMarketPda(params.marketId, this.programId);
    const [vault] = pda.getVaultPda(params.depositMint, market, this.programId);
    const [mintAuthority] = pda.getMintAuthorityPda(market, this.programId);
    const conditionalMints = pda
      .getAllConditionalMintPdas(
        market,
        params.depositMint,
        numOutcomes,
        this.programId
      )
      .map(([mint]) => mint);

    const ix = buildAddDepositMintIx(params, market, numOutcomes, this.programId);

    return this.createBuildResult(
      params.authority,
      { market, vault, mintAuthority, conditionalMints },
      ix
    );
  }

  async mintCompleteSet(
    params: MintCompleteSetParams,
    numOutcomes: number
  ): Promise<BuildResult<MintCompleteSetAccounts>> {
    const [vault] = pda.getVaultPda(
      params.depositMint,
      params.market,
      this.programId
    );
    const [position] = pda.getPositionPda(
      params.user,
      params.market,
      this.programId
    );
    const conditionalMints = pda
      .getAllConditionalMintPdas(
        params.market,
        params.depositMint,
        numOutcomes,
        this.programId
      )
      .map(([mint]) => mint);

    const ix = buildMintCompleteSetIx(params, numOutcomes, this.programId);

    return this.createBuildResult(
      params.user,
      { position, vault, conditionalMints },
      ix
    );
  }

  async mergeCompleteSet(
    params: MergeCompleteSetParams,
    numOutcomes: number
  ): Promise<BuildResult<MergeCompleteSetAccounts>> {
    const [vault] = pda.getVaultPda(
      params.depositMint,
      params.market,
      this.programId
    );
    const [position] = pda.getPositionPda(
      params.user,
      params.market,
      this.programId
    );
    const conditionalMints = pda
      .getAllConditionalMintPdas(
        params.market,
        params.depositMint,
        numOutcomes,
        this.programId
      )
      .map(([mint]) => mint);

    const ix = buildMergeCompleteSetIx(params, numOutcomes, this.programId);

    return this.createBuildResult(
      params.user,
      { position, vault, conditionalMints },
      ix
    );
  }

  async cancelOrder(
    maker: PublicKey,
    order: SignedOrder
  ): Promise<BuildResult<CancelOrderAccounts>> {
    const orderHash = hashOrder(order);
    const [orderStatus] = pda.getOrderStatusPda(orderHash, this.programId);
    const ix = buildCancelOrderIx(maker, order, this.programId);
    return this.createBuildResult(maker, { orderStatus }, ix);
  }

  async incrementNonce(
    user: PublicKey
  ): Promise<BuildResult<IncrementNonceAccounts>> {
    const [userNonce] = pda.getUserNoncePda(user, this.programId);
    const ix = buildIncrementNonceIx(user, this.programId);
    return this.createBuildResult(user, { userNonce }, ix);
  }

  async settleMarket(
    params: SettleMarketParams
  ): Promise<BuildResult<SettleMarketAccounts>> {
    const [exchange] = pda.getExchangePda(this.programId);
    const [market] = pda.getMarketPda(params.marketId, this.programId);
    const ix = buildSettleMarketIx(params, this.programId);
    return this.createBuildResult(params.oracle, { exchange, market }, ix);
  }

  async redeemWinnings(
    params: RedeemWinningsParams,
    winningOutcome: number
  ): Promise<BuildResult<RedeemWinningsAccounts>> {
    const [vault] = pda.getVaultPda(
      params.depositMint,
      params.market,
      this.programId
    );
    const [position] = pda.getPositionPda(
      params.user,
      params.market,
      this.programId
    );
    const [winningMint] = pda.getConditionalMintPda(
      params.market,
      params.depositMint,
      winningOutcome,
      this.programId
    );

    const ix = buildRedeemWinningsIx(params, winningOutcome, this.programId);

    return this.createBuildResult(
      params.user,
      { position, vault, winningMint },
      ix
    );
  }

  async setPaused(
    authority: PublicKey,
    paused: boolean
  ): Promise<BuildResult<{ exchange: PublicKey }>> {
    const [exchange] = pda.getExchangePda(this.programId);
    const ix = buildSetPausedIx(authority, paused, this.programId);
    return this.createBuildResult(authority, { exchange }, ix);
  }

  async setOperator(
    authority: PublicKey,
    newOperator: PublicKey
  ): Promise<BuildResult<{ exchange: PublicKey }>> {
    const [exchange] = pda.getExchangePda(this.programId);
    const ix = buildSetOperatorIx(authority, newOperator, this.programId);
    return this.createBuildResult(authority, { exchange }, ix);
  }

  async withdrawFromPosition(
    params: WithdrawFromPositionParams,
    isToken2022: boolean
  ): Promise<BuildResult<{ position: PublicKey }>> {
    const [position] = pda.getPositionPda(
      params.user,
      params.market,
      this.programId
    );
    const ix = buildWithdrawFromPositionIx(params, isToken2022, this.programId);
    return this.createBuildResult(params.user, { position }, ix);
  }

  async activateMarket(
    params: ActivateMarketParams
  ): Promise<BuildResult<ActivateMarketAccounts>> {
    const [exchange] = pda.getExchangePda(this.programId);
    const [market] = pda.getMarketPda(params.marketId, this.programId);
    const ix = buildActivateMarketIx(params, this.programId);
    return this.createBuildResult(params.authority, { exchange, market }, ix);
  }

  async matchOrdersMulti(
    params: MatchOrdersMultiParams
  ): Promise<BuildResult<MatchOrdersMultiAccounts>> {
    const takerOrderHash = hashOrder(params.takerOrder);
    const [takerOrderStatus] = pda.getOrderStatusPda(
      takerOrderHash,
      this.programId
    );
    const [takerPosition] = pda.getPositionPda(
      params.takerOrder.maker,
      params.market,
      this.programId
    );

    const makerOrderStatuses: PublicKey[] = [];
    const makerPositions: PublicKey[] = [];
    for (const makerOrder of params.makerOrders) {
      const makerOrderHash = hashOrder(makerOrder);
      const [makerOrderStatus] = pda.getOrderStatusPda(
        makerOrderHash,
        this.programId
      );
      const [makerPosition] = pda.getPositionPda(
        makerOrder.maker,
        params.market,
        this.programId
      );
      makerOrderStatuses.push(makerOrderStatus);
      makerPositions.push(makerPosition);
    }

    const ix = buildMatchOrdersMultiIx(params, this.programId);

    return this.createBuildResult(
      params.operator,
      { takerOrderStatus, takerPosition, makerOrderStatuses, makerPositions },
      ix
    );
  }

  async setAuthority(
    params: SetAuthorityParams
  ): Promise<BuildResult<{ exchange: PublicKey }>> {
    const [exchange] = pda.getExchangePda(this.programId);
    const ix = buildSetAuthorityIx(params, this.programId);
    return this.createBuildResult(
      params.currentAuthority,
      { exchange },
      ix
    );
  }

  async createOrderbook(
    params: CreateOrderbookParams
  ): Promise<BuildResult<{ orderbook: PublicKey }>> {
    const [orderbook] = pda.getOrderbookPda(
      params.mintA,
      params.mintB,
      this.programId
    );
    const ix = buildCreateOrderbookIx(params, this.programId);
    return this.createBuildResult(params.payer, { orderbook }, ix);
  }

  // ============================================================================
  // ORDER HELPERS
  // ============================================================================

  createBidOrder(params: BidOrderParams): Omit<SignedOrder, "signature"> {
    return createBidOrder(params);
  }

  createAskOrder(params: AskOrderParams): Omit<SignedOrder, "signature"> {
    return createAskOrder(params);
  }

  hashOrder(order: SignedOrder): Buffer {
    return hashOrder(order);
  }

  signOrder(order: SignedOrder, signer: Keypair): Buffer {
    return signOrder(order, signer);
  }

  signFullOrder(
    order: Omit<SignedOrder, "signature">,
    signer: Keypair
  ): SignedOrder {
    return signOrderFull(order, signer);
  }

  // ============================================================================
  // UTILITY METHODS
  // ============================================================================

  deriveConditionId(
    oracle: PublicKey,
    questionId: Buffer,
    numOutcomes: number
  ): Buffer {
    return deriveConditionId(oracle, questionId, numOutcomes);
  }

  getConditionalMints(
    market: PublicKey,
    depositMint: PublicKey,
    numOutcomes: number
  ): PublicKey[] {
    return pda
      .getAllConditionalMintPdas(
        market,
        depositMint,
        numOutcomes,
        this.programId
      )
      .map(([mint]) => mint);
  }
}
