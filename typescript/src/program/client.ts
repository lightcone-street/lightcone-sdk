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
  GlobalDepositToken,
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
  WhitelistDepositTokenParams,
  DepositToGlobalParams,
  GlobalToMarketDepositParams,
  InitPositionTokensParams,
  ExtendPositionTokensParams,
  DepositAndSwapParams,
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
  deserializeGlobalDepositToken,
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
  buildWhitelistDepositTokenIx,
  buildDepositToGlobalIx,
  buildGlobalToMarketDepositIx,
  buildInitPositionTokensIx,
  buildExtendPositionTokensIx,
  buildDepositAndSwapIx,
} from "./instructions";
import {
  hashOrder,
  signOrder,
  createBidOrder,
  createAskOrder,
  createSignedBidOrder,
  createSignedAskOrder,
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

  static new(rpcUrl: string): LightconePinocchioClient {
    return new LightconePinocchioClient(new Connection(rpcUrl), PROGRAM_ID);
  }

  static withProgramId(rpcUrl: string, programId: PublicKey): LightconePinocchioClient {
    return new LightconePinocchioClient(new Connection(rpcUrl), programId);
  }

  static fromConnection(connection: Connection): LightconePinocchioClient {
    return new LightconePinocchioClient(connection, PROGRAM_ID);
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

  async getCurrentNonce(user: PublicKey): Promise<number> {
    const nonce = await this.getUserNonce(user);
    if (nonce > 0xFFFFFFFFn) {
      throw new Error(`Nonce exceeds u32 range: ${nonce}`);
    }
    return Number(nonce);
  }

  async getNextNonce(user: PublicKey): Promise<number> {
    return this.getCurrentNonce(user);
  }

  async getNextMarketId(): Promise<bigint> {
    const exchange = await this.getExchange();
    return exchange.marketCount;
  }

  async getOrderbook(
    mintA: PublicKey,
    mintB: PublicKey
  ): Promise<Orderbook> {
    const [orderbookPda] = pda.getOrderbookPda(mintA, mintB, this.programId);
    const accountInfo = await this.connection.getAccountInfo(orderbookPda);
    if (!accountInfo) {
      throw new Error(
        `Orderbook not found for ${mintA.toBase58()} / ${mintB.toBase58()}`
      );
    }
    return deserializeOrderbook(accountInfo.data as Buffer);
  }

  async getGlobalDepositToken(mint: PublicKey): Promise<GlobalDepositToken> {
    const [globalDepositTokenPda] = pda.getGlobalDepositTokenPda(mint, this.programId);
    const accountInfo = await this.connection.getAccountInfo(globalDepositTokenPda);
    if (!accountInfo) {
      throw new Error(`GlobalDepositToken not found for mint ${mint.toBase58()}`);
    }
    return deserializeGlobalDepositToken(accountInfo.data as Buffer);
  }

  async getLatestBlockhash() {
    return this.connection.getLatestBlockhash();
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
    market: PublicKey,
    numOutcomes: number
  ): Promise<BuildResult<AddDepositMintAccounts>> {
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
    return this.createBuildResult(params.authority, { orderbook }, ix);
  }

  async whitelistDepositToken(
    params: WhitelistDepositTokenParams
  ): Promise<BuildResult<{ globalDepositToken: PublicKey }>> {
    const [globalDepositToken] = pda.getGlobalDepositTokenPda(params.mint, this.programId);
    const ix = buildWhitelistDepositTokenIx(params, this.programId);
    return this.createBuildResult(params.authority, { globalDepositToken }, ix);
  }

  async depositToGlobal(
    params: DepositToGlobalParams
  ): Promise<BuildResult<{ userGlobalDeposit: PublicKey }>> {
    const [userGlobalDeposit] = pda.getUserGlobalDepositPda(params.user, params.mint, this.programId);
    const ix = buildDepositToGlobalIx(params, this.programId);
    return this.createBuildResult(params.user, { userGlobalDeposit }, ix);
  }

  async globalToMarketDeposit(
    params: GlobalToMarketDepositParams,
    numOutcomes: number
  ): Promise<BuildResult<{ position: PublicKey; userGlobalDeposit: PublicKey }>> {
    const [position] = pda.getPositionPda(params.user, params.market, this.programId);
    const [userGlobalDeposit] = pda.getUserGlobalDepositPda(
      params.user,
      params.depositMint,
      this.programId
    );
    const ix = buildGlobalToMarketDepositIx(params, numOutcomes, this.programId);
    return this.createBuildResult(params.user, { position, userGlobalDeposit }, ix);
  }

  async initPositionTokens(
    params: InitPositionTokensParams,
    numOutcomes: number
  ): Promise<BuildResult<{ position: PublicKey; lookupTable: PublicKey }>> {
    const [position] = pda.getPositionPda(params.user, params.market, this.programId);
    const [lookupTable] = pda.getPositionAltPda(position, params.recentSlot);
    const ix = buildInitPositionTokensIx(params, numOutcomes, this.programId);
    return this.createBuildResult(params.payer, { position, lookupTable }, ix);
  }

  async extendPositionTokens(
    params: ExtendPositionTokensParams,
    numOutcomes: number
  ): Promise<BuildResult<{ position: PublicKey; lookupTable: PublicKey }>> {
    const [position] = pda.getPositionPda(params.user, params.market, this.programId);
    const ix = buildExtendPositionTokensIx(params, numOutcomes, this.programId);
    return this.createBuildResult(params.payer, { position, lookupTable: params.lookupTable }, ix);
  }

  async depositAndSwap(
    params: DepositAndSwapParams
  ): Promise<BuildResult<{ takerPosition: PublicKey }>> {
    const [takerPosition] = pda.getPositionPda(
      params.takerOrder.maker,
      params.market,
      this.programId
    );
    const ix = buildDepositAndSwapIx(params, this.programId);
    return this.createBuildResult(params.operator, { takerPosition }, ix);
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

  createSignedBidOrder(params: BidOrderParams, signer: Keypair): SignedOrder {
    return createSignedBidOrder(params, signer);
  }

  createSignedAskOrder(params: AskOrderParams, signer: Keypair): SignedOrder {
    return createSignedAskOrder(params, signer);
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

  getExchangePda(): PublicKey {
    return pda.getExchangePda(this.programId)[0];
  }

  getMarketPda(marketId: bigint): PublicKey {
    return pda.getMarketPda(marketId, this.programId)[0];
  }

  getPositionPda(owner: PublicKey, market: PublicKey): PublicKey {
    return pda.getPositionPda(owner, market, this.programId)[0];
  }

  getOrderStatusPda(orderHash: Buffer): PublicKey {
    return pda.getOrderStatusPda(orderHash, this.programId)[0];
  }

  getUserNoncePda(user: PublicKey): PublicKey {
    return pda.getUserNoncePda(user, this.programId)[0];
  }

  getOrderbookPda(mintA: PublicKey, mintB: PublicKey): PublicKey {
    return pda.getOrderbookPda(mintA, mintB, this.programId)[0];
  }

  getGlobalDepositTokenPda(mint: PublicKey): PublicKey {
    return pda.getGlobalDepositTokenPda(mint, this.programId)[0];
  }

  getUserGlobalDepositPda(user: PublicKey, mint: PublicKey): PublicKey {
    return pda.getUserGlobalDepositPda(user, mint, this.programId)[0];
  }
}
