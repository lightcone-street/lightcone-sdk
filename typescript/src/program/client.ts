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
  FullOrder,
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
} from "./instructions";
import {
  hashOrder,
  signOrder,
  createBidOrder,
  createAskOrder,
  signOrderFull,
} from "./orders";
import { buildMatchOrdersTransaction } from "./ed25519";
import { deriveConditionId } from "./utils";

/**
 * LightconePinocchioClient - SDK for interacting with the Lightcone Pinocchio program
 *
 * This client provides methods to:
 * - Fetch account data (Exchange, Market, Position, etc.)
 * - Build unsigned transactions for all instructions
 * - Create and sign orders
 * - Derive PDAs
 */
export class LightconePinocchioClient {
  /** Solana connection */
  readonly connection: Connection;
  /** Program ID */
  readonly programId: PublicKey;
  /** PDA derivation functions */
  readonly pda = pda;

  /**
   * Create a new LightconePinocchioClient
   * @param connection - Solana connection
   * @param programId - Program ID (defaults to mainnet program)
   */
  constructor(connection: Connection, programId: PublicKey = PROGRAM_ID) {
    this.connection = connection;
    this.programId = programId;
  }

  // ============================================================================
  // ACCOUNT FETCHERS
  // ============================================================================

  /**
   * Fetch the Exchange account (singleton)
   */
  async getExchange(): Promise<Exchange> {
    const [exchangePda] = pda.getExchangePda(this.programId);
    const accountInfo = await this.connection.getAccountInfo(exchangePda);
    if (!accountInfo) {
      throw new Error("Exchange account not found. Protocol not initialized.");
    }
    return deserializeExchange(accountInfo.data as Buffer);
  }

  /**
   * Fetch a Market by ID
   */
  async getMarket(marketId: bigint): Promise<Market> {
    const [marketPda] = pda.getMarketPda(marketId, this.programId);
    const accountInfo = await this.connection.getAccountInfo(marketPda);
    if (!accountInfo) {
      throw new Error(`Market ${marketId} not found`);
    }
    return deserializeMarket(accountInfo.data as Buffer);
  }

  /**
   * Fetch a Market by its pubkey
   */
  async getMarketByPubkey(market: PublicKey): Promise<Market> {
    const accountInfo = await this.connection.getAccountInfo(market);
    if (!accountInfo) {
      throw new Error(`Market not found at ${market.toBase58()}`);
    }
    return deserializeMarket(accountInfo.data as Buffer);
  }

  /**
   * Fetch a Position for a user in a market
   * Returns null if the position doesn't exist
   */
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

  /**
   * Fetch an OrderStatus by order hash
   * Returns null if the order status doesn't exist
   */
  async getOrderStatus(orderHash: Buffer): Promise<OrderStatus | null> {
    const [orderStatusPda] = pda.getOrderStatusPda(orderHash, this.programId);
    const accountInfo = await this.connection.getAccountInfo(orderStatusPda);
    if (!accountInfo) {
      return null;
    }
    return deserializeOrderStatus(accountInfo.data as Buffer);
  }

  /**
   * Fetch a user's nonce
   * Returns 0n if the nonce account doesn't exist
   */
  async getUserNonce(user: PublicKey): Promise<bigint> {
    const [userNoncePda] = pda.getUserNoncePda(user, this.programId);
    const accountInfo = await this.connection.getAccountInfo(userNoncePda);
    if (!accountInfo) {
      return 0n;
    }
    const nonce = deserializeUserNonce(accountInfo.data as Buffer);
    return nonce.nonce;
  }

  /**
   * Get the next nonce for a user (current nonce value for new orders)
   */
  async getNextNonce(user: PublicKey): Promise<bigint> {
    return this.getUserNonce(user);
  }

  /**
   * Get the next market ID that will be assigned
   */
  async getNextMarketId(): Promise<bigint> {
    const exchange = await this.getExchange();
    return exchange.marketCount;
  }

  // ============================================================================
  // TRANSACTION BUILDERS
  // ============================================================================

  /**
   * Helper to create a BuildResult from an instruction
   */
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

  /**
   * Build Initialize instruction
   * Creates the exchange account (singleton)
   */
  async initialize(
    params: InitializeParams
  ): Promise<BuildResult<InitializeAccounts>> {
    const [exchange] = pda.getExchangePda(this.programId);
    const ix = buildInitializeIx(params, this.programId);

    return this.createBuildResult(params.authority, { exchange }, ix);
  }

  /**
   * Build CreateMarket instruction
   */
  async createMarket(
    params: CreateMarketParams
  ): Promise<BuildResult<CreateMarketAccounts>> {
    const marketId = await this.getNextMarketId();
    const [exchange] = pda.getExchangePda(this.programId);
    const [market] = pda.getMarketPda(marketId, this.programId);
    const ix = buildCreateMarketIx(params, marketId, this.programId);

    return this.createBuildResult(params.authority, { exchange, market }, ix);
  }

  /**
   * Build AddDepositMint instruction
   */
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

  /**
   * Build MintCompleteSet instruction
   */
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

  /**
   * Build MergeCompleteSet instruction
   */
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

  /**
   * Build CancelOrder instruction
   */
  async cancelOrder(
    maker: PublicKey,
    order: FullOrder
  ): Promise<BuildResult<CancelOrderAccounts>> {
    const orderHash = hashOrder(order);
    const [orderStatus] = pda.getOrderStatusPda(orderHash, this.programId);
    const ix = buildCancelOrderIx(maker, order, this.programId);

    return this.createBuildResult(maker, { orderStatus }, ix);
  }

  /**
   * Build IncrementNonce instruction
   */
  async incrementNonce(
    user: PublicKey
  ): Promise<BuildResult<IncrementNonceAccounts>> {
    const [userNonce] = pda.getUserNoncePda(user, this.programId);
    const ix = buildIncrementNonceIx(user, this.programId);

    return this.createBuildResult(user, { userNonce }, ix);
  }

  /**
   * Build SettleMarket instruction
   */
  async settleMarket(
    params: SettleMarketParams
  ): Promise<BuildResult<SettleMarketAccounts>> {
    const [exchange] = pda.getExchangePda(this.programId);
    const [market] = pda.getMarketPda(params.marketId, this.programId);
    const ix = buildSettleMarketIx(params, this.programId);

    return this.createBuildResult(params.oracle, { exchange, market }, ix);
  }

  /**
   * Build RedeemWinnings instruction
   */
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

  /**
   * Build SetPaused instruction
   */
  async setPaused(
    authority: PublicKey,
    paused: boolean
  ): Promise<BuildResult<{ exchange: PublicKey }>> {
    const [exchange] = pda.getExchangePda(this.programId);
    const ix = buildSetPausedIx(authority, paused, this.programId);

    return this.createBuildResult(authority, { exchange }, ix);
  }

  /**
   * Build SetOperator instruction
   */
  async setOperator(
    authority: PublicKey,
    newOperator: PublicKey
  ): Promise<BuildResult<{ exchange: PublicKey }>> {
    const [exchange] = pda.getExchangePda(this.programId);
    const ix = buildSetOperatorIx(authority, newOperator, this.programId);

    return this.createBuildResult(authority, { exchange }, ix);
  }

  /**
   * Build WithdrawFromPosition instruction
   */
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

  /**
   * Build ActivateMarket instruction
   */
  async activateMarket(
    params: ActivateMarketParams
  ): Promise<BuildResult<ActivateMarketAccounts>> {
    const [exchange] = pda.getExchangePda(this.programId);
    const [market] = pda.getMarketPda(params.marketId, this.programId);
    const ix = buildActivateMarketIx(params, this.programId);

    return this.createBuildResult(params.authority, { exchange, market }, ix);
  }

  /**
   * Build MatchOrdersMulti instruction (without Ed25519 verification)
   * Note: For a complete transaction, use matchOrdersMultiWithVerify instead
   */
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

  /**
   * Build complete MatchOrdersMulti transaction with Ed25519 signature verification
   * This is the recommended way to create match orders transactions
   */
  async matchOrdersMultiWithVerify(
    params: MatchOrdersMultiParams
  ): Promise<BuildResult<MatchOrdersMultiAccounts>> {
    const transaction = buildMatchOrdersTransaction(params, this.programId);

    const { blockhash, lastValidBlockHeight } =
      await this.connection.getLatestBlockhash();
    transaction.recentBlockhash = blockhash;
    transaction.lastValidBlockHeight = lastValidBlockHeight;
    transaction.feePayer = params.operator;

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

    return {
      transaction,
      accounts: {
        takerOrderStatus,
        takerPosition,
        makerOrderStatuses,
        makerPositions,
      },
      serialize: () =>
        transaction
          .serialize({ requireAllSignatures: false, verifySignatures: false })
          .toString("base64"),
    };
  }

  // ============================================================================
  // ORDER HELPERS
  // ============================================================================

  /**
   * Create a BID order (unsigned)
   */
  createBidOrder(params: BidOrderParams): Omit<FullOrder, "signature"> {
    return createBidOrder(params);
  }

  /**
   * Create an ASK order (unsigned)
   */
  createAskOrder(params: AskOrderParams): Omit<FullOrder, "signature"> {
    return createAskOrder(params);
  }

  /**
   * Hash an order
   */
  hashOrder(order: FullOrder): Buffer {
    return hashOrder(order);
  }

  /**
   * Sign an order with a Keypair
   */
  signOrder(order: FullOrder, signer: Keypair): Buffer {
    return signOrder(order, signer);
  }

  /**
   * Create and sign a full order
   */
  signFullOrder(
    order: Omit<FullOrder, "signature">,
    signer: Keypair
  ): FullOrder {
    return signOrderFull(order, signer);
  }

  // ============================================================================
  // UTILITY METHODS
  // ============================================================================

  /**
   * Derive condition ID from oracle, question ID, and number of outcomes
   */
  deriveConditionId(
    oracle: PublicKey,
    questionId: Buffer,
    numOutcomes: number
  ): Buffer {
    return deriveConditionId(oracle, questionId, numOutcomes);
  }

  /**
   * Get all conditional mint addresses for a market
   */
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
