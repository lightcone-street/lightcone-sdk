import {
  PublicKey,
  SystemProgram,
  Transaction,
  type TransactionInstruction,
} from "@solana/web3.js";
import {
  createAssociatedTokenAccountIdempotentInstruction,
  createCloseAccountInstruction,
  createSyncNativeInstruction,
  getAssociatedTokenAddressSync,
  NATIVE_MINT,
  TOKEN_PROGRAM_ID,
} from "@solana/spl-token";
import Decimal from "decimal.js";
import { isAuthenticated } from "../../auth";
import type { ClientContext } from "../../context";
import {
  requireConnection,
  requireSigningStrategy,
  signAndSubmitTxConfirmedUsingStrategy,
} from "../../context";
import { SdkError } from "../../error";
import { RetryPolicy } from "../../http";
import { exactScaledInteger } from "../../shared";
import type { SigningStrategy } from "../../shared/signing";
import {
  buildRedeemWinningsIx,
  buildWithdrawConditionalFromPositionIx,
  buildInitPositionTokensIx,
  buildExtendPositionTokensIx,
  buildDepositToGlobalIx,
  buildDepositToGlobalIxWithAlt,
  buildGlobalToMarketDepositIx,
  buildWithdrawFromGlobalIx,
  buildClosePositionAltIx,
  buildClosePositionTokenAccountsIx,
} from "../../program/instructions";
import { getPositionPda } from "../../program/pda";
import { deserializePosition as deserializeProgramPosition } from "../../program/accounts";
import type {
  Position as ProgramPosition,
  RedeemWinningsParams,
  WithdrawConditionalFromPositionParams,
  WithdrawFromPositionParams,
  InitPositionTokensParams,
  ExtendPositionTokensParams,
  DepositToGlobalParams,
  DepositToGlobalAltContext,
  GlobalToMarketDepositParams,
  WithdrawFromGlobalParams,
  ClosePositionAltParams,
  ClosePositionTokenAccountsParams,
} from "../../program/types";
import type { DepositTokenBalancesSnapshot } from "./index";
import type { WalletDepositBalancesState } from "./state";
import type { MarketPositionsResponse, PositionsResponse } from "./wire";
import {
  DepositBuilder,
  MergeBuilder,
  WithdrawBuilder,
  RedeemWinningsBuilder,
  WithdrawFromPositionBuilder,
  InitPositionTokensBuilder,
  ExtendPositionTokensBuilder,
  DepositToGlobalBuilder,
  WithdrawFromGlobalBuilder,
  GlobalToMarketDepositBuilder,
} from "./builders";

export class Positions {
  constructor(private readonly client: ClientContext) {}

  // ── PDA helpers ──────────────────────────────────────────────────────

  pda(owner: PublicKey, market: PublicKey): PublicKey {
    return getPositionPda(owner, market, this.client.programId)[0];
  }

  // ── HTTP methods ─────────────────────────────────────────────────────

  async get(userPubkey: string): Promise<PositionsResponse> {
    const url = `${this.client.http.baseUrl()}/api/users/${encodeURIComponent(userPubkey)}/positions`;
    return this.client.http.get<PositionsResponse>(url, RetryPolicy.Idempotent);
  }

  async getForMarket(userPubkey: string, marketPubkey: string): Promise<MarketPositionsResponse> {
    const url = `${this.client.http.baseUrl()}/api/users/${encodeURIComponent(userPubkey)}/markets/${encodeURIComponent(marketPubkey)}/positions`;
    return this.client.http.get<MarketPositionsResponse>(url, RetryPolicy.Idempotent);
  }

  /**
   * Get all conditional-token positions for the authenticated user across
   * every market. The wallet is resolved server-side from the auth cookie,
   * so no parameter is required. Same response shape as `get()`.
   *
   * `GET /api/users/positions`
   */
  async positions(): Promise<PositionsResponse> {
    const url = `${this.client.http.baseUrl()}/api/users/positions`;
    return this.client.http.get<PositionsResponse>(url, RetryPolicy.Idempotent);
  }

  /**
   * Same as {@link positions}, but uses the supplied `cookieHeader` for this
   * call instead of the SDK's process-wide cookie store.
   *
   * Intended for server-side cookie forwarding (SSR / server functions)
   * where the per-request browser cookie can't propagate to the shared
   * client. In a browser context this is equivalent to {@link positions}
   * because the runtime is already attaching the cookie via
   * `credentials: "include"`.
   */
  async positionsWithCookies(cookieHeader: string): Promise<PositionsResponse> {
    const url = `${this.client.http.baseUrl()}/api/users/positions`;
    return this.client.http.getWithCookies<PositionsResponse>(
      url,
      RetryPolicy.Idempotent,
      cookieHeader,
    );
  }

  /**
   * Get the authenticated user's positions in a specific market. The wallet
   * is resolved server-side from the auth cookie.
   *
   * `GET /api/users/markets/{market_pubkey}/positions`
   */
  async positionsForMarket(marketPubkey: string): Promise<MarketPositionsResponse> {
    const url = `${this.client.http.baseUrl()}/api/users/markets/${encodeURIComponent(marketPubkey)}/positions`;
    return this.client.http.get<MarketPositionsResponse>(url, RetryPolicy.Idempotent);
  }

  /**
   * Same as {@link positionsForMarket}, but uses the supplied `cookieHeader`
   * for this call instead of the SDK's process-wide cookie store. For
   * server-side cookie forwarding (SSR / server functions).
   */
  async positionsForMarketWithCookies(
    marketPubkey: string,
    cookieHeader: string,
  ): Promise<MarketPositionsResponse> {
    const url = `${this.client.http.baseUrl()}/api/users/markets/${encodeURIComponent(marketPubkey)}/positions`;
    return this.client.http.getWithCookies<MarketPositionsResponse>(
      url,
      RetryPolicy.Idempotent,
      cookieHeader,
    );
  }

  /**
   * Fetch a complete authenticated SPL and native-SOL balance snapshot.
   *
   * `minContextSlot` lower-bounds the cross-component snapshot. Native SOL is
   * required canonical nine-decimal text and remains outside the SPL map. The
   * generic HTTP layer trusts that shape at runtime; WebSocket frames are decoded
   * strictly, while malformed REST exact values fail later when state scales them.
   */
  async depositTokenBalances(
    minContextSlot?: number
  ): Promise<DepositTokenBalancesSnapshot> {
    const query =
      minContextSlot === undefined
        ? ""
        : `?min_context_slot=${encodeURIComponent(minContextSlot)}`;
    const url = `${this.client.http.baseUrl()}/api/users/deposit-token-balances${query}`;
    return this.client.http.get<DepositTokenBalancesSnapshot>(
      url,
      RetryPolicy.Idempotent,
    );
  }

  /**
   * Same as {@link depositTokenBalances}, but uses the supplied `cookieHeader`
   * for this call instead of the SDK's process-wide cookie store.
   *
   * Intended for server-side cookie forwarding (SSR / server functions)
   * where the per-request browser cookie can't propagate to the shared
   * client. The complete response has the same separate, exact native-SOL
   * contract as {@link depositTokenBalances}. In a browser this is equivalent to
   * {@link depositTokenBalances} because the runtime is already attaching
   * the cookie via `credentials: "include"`.
   */
  async depositTokenBalancesWithCookies(
    minContextSlot: number | undefined,
    cookieHeader: string,
  ): Promise<DepositTokenBalancesSnapshot> {
    const query =
      minContextSlot === undefined
        ? ""
        : `?min_context_slot=${encodeURIComponent(minContextSlot)}`;
    const url = `${this.client.http.baseUrl()}/api/users/deposit-token-balances${query}`;
    return this.client.http.getWithCookies<DepositTokenBalancesSnapshot>(
      url,
      RetryPolicy.Idempotent,
      cookieHeader,
    );
  }

  /**
   * Wrap exact SOL into the authenticated wallet's canonical Tokenkeg WSOL ATA.
   *
   * The amount must be positive, exactly representable at nine decimals, fit a
   * Solana `u64`, and not exceed cached native SOL. Live credentials must match
   * initialized state, and the configured signing strategy must control that
   * wallet. The method builds create/transfer/sync instructions, confirms them,
   * returns the transaction signature, and never mutates state. Fee and rent
   * reserves remain chain-authoritative rather than guessed locally. A confirmation
   * error does not prove rollback; refresh authoritative state before retrying.
   */
  async wrapSol(
    amount: string | Decimal,
    state: WalletDepositBalancesState
  ): Promise<string> {
    const { wallet, strategy } = this.conversionWallet(state);
    const lamports = solLamports(amount);
    if (lamports <= 0n) {
      throw SdkError.validation("wrap amount must be greater than zero");
    }
    // Do not guess a fee or ATA-rent reserve from stale client state; an
    // equal-balance wrap is valid preflight and the chain remains authoritative.
    if (lamports > state.nativeSolLamports()) {
      throw SdkError.validation(
        "wrap amount exceeds cached native SOL balance"
      );
    }

    const account = getAssociatedTokenAddressSync(
      NATIVE_MINT,
      wallet,
      false,
      TOKEN_PROGRAM_ID
    );
    const transaction = new Transaction({ feePayer: wallet }).add(
      createAssociatedTokenAccountIdempotentInstruction(
        wallet,
        account,
        wallet,
        NATIVE_MINT,
        TOKEN_PROGRAM_ID
      ),
      SystemProgram.transfer({
        fromPubkey: wallet,
        toPubkey: account,
        lamports,
      }),
      createSyncNativeInstruction(account, TOKEN_PROGRAM_ID)
    );
    return signAndSubmitTxConfirmedUsingStrategy(
      this.client,
      transaction,
      strategy
    );
  }

  /**
   * Fully unwrap the authenticated wallet's canonical Tokenkeg WSOL ATA.
   *
   * Live matching credentials, a signing strategy controlling that wallet, and
   * positive cached canonical WSOL are required. CloseAccount credits all token
   * lamports plus rent to the wallet; partial unwrap is unsupported. The method
   * returns the confirmed transaction signature and leaves cached state unchanged.
   * A confirmation error does not prove the account stayed open; refresh
   * authoritative state before retrying.
   */
  async unwrapWsol(state: WalletDepositBalancesState): Promise<string> {
    const { wallet, strategy } = this.conversionWallet(state);
    if (!state.hasPositiveWsol()) {
      throw SdkError.validation(
        "canonical WSOL balance must be greater than zero"
      );
    }
    const account = getAssociatedTokenAddressSync(
      NATIVE_MINT,
      wallet,
      false,
      TOKEN_PROGRAM_ID
    );
    const transaction = new Transaction({ feePayer: wallet }).add(
      createCloseAccountInstruction(
        account,
        wallet,
        wallet,
        [],
        TOKEN_PROGRAM_ID
      )
    );
    return signAndSubmitTxConfirmedUsingStrategy(
      this.client,
      transaction,
      strategy
    );
  }

  private conversionWallet(state: WalletDepositBalancesState): {
    wallet: PublicKey;
    strategy: SigningStrategy;
  } {
    // Cached identity is a signing trust boundary: validate expiry, complete
    // state initialization, and wallet equality before constructing a transaction.
    const credentials = this.client.authCredentials;
    if (!credentials) {
      throw SdkError.validation("authenticated credentials are required");
    }
    if (!isAuthenticated(credentials)) {
      throw SdkError.validation("authenticated credentials have expired");
    }
    if (
      state.walletAddress === undefined ||
      state.contextSlot === undefined ||
      state.nativeSolBalance === undefined
    ) {
      throw SdkError.validation("wallet balance state is not initialized");
    }
    if (state.walletAddress !== credentials.wallet_address) {
      throw SdkError.validation(
        "authenticated wallet does not match wallet balance state"
      );
    }
    let wallet: PublicKey;
    try {
      wallet = new PublicKey(credentials.wallet_address);
    } catch (error) {
      throw SdkError.validation(
        `authenticated wallet is invalid: ${error instanceof Error ? error.message : String(error)}`
      );
    }
    const strategy = requireSigningStrategy(this.client);
    const signingAddress =
      strategy.type === "native"
        ? strategy.keypair.publicKey.toBase58()
        : strategy.type === "walletAdapter"
          ? strategy.signer.walletAddress
          : strategy.walletAddress;
    if (signingAddress === undefined) {
      throw SdkError.validation(
        "signing strategy wallet identity is required"
      );
    }
    let signingWallet: PublicKey;
    try {
      signingWallet = new PublicKey(signingAddress);
    } catch (error) {
      throw SdkError.validation(
        `signing strategy wallet is invalid: ${error instanceof Error ? error.message : String(error)}`
      );
    }
    if (!signingWallet.equals(wallet)) {
      throw SdkError.validation(
        "signing strategy does not control authenticated wallet"
      );
    }
    return { wallet, strategy };
  }

  // ── On-chain transaction builders ────────────────────────────────────

  redeemWinningsIx(
    params: RedeemWinningsParams,
    outcomeIndex: number
  ): TransactionInstruction {
    return buildRedeemWinningsIx(params, outcomeIndex, this.client.programId);
  }

  withdrawConditionalFromPositionIx(
    params: WithdrawConditionalFromPositionParams
  ): TransactionInstruction {
    return buildWithdrawConditionalFromPositionIx(params, this.client.programId);
  }

  withdrawFromPositionIx(
    params: WithdrawFromPositionParams
  ): TransactionInstruction {
    return this.withdrawConditionalFromPositionIx(params);
  }

  initPositionTokensIx(
    params: InitPositionTokensParams,
    numOutcomes: number
  ): TransactionInstruction {
    return buildInitPositionTokensIx(params, numOutcomes, this.client.programId);
  }

  extendPositionTokensIx(
    params: ExtendPositionTokensParams,
    numOutcomes: number
  ): TransactionInstruction {
    return buildExtendPositionTokensIx(params, numOutcomes, this.client.programId);
  }

  depositToGlobalIx(params: DepositToGlobalParams): TransactionInstruction {
    return buildDepositToGlobalIx(params, this.client.programId);
  }

  depositToGlobalIxWithAlt(
    params: DepositToGlobalParams,
    altContext: DepositToGlobalAltContext
  ): TransactionInstruction {
    return buildDepositToGlobalIxWithAlt(params, altContext, this.client.programId);
  }

  globalToMarketDepositIx(
    params: GlobalToMarketDepositParams,
    numOutcomes: number
  ): TransactionInstruction {
    return buildGlobalToMarketDepositIx(params, numOutcomes, this.client.programId);
  }

  withdrawFromGlobalIx(params: WithdrawFromGlobalParams): TransactionInstruction {
    return buildWithdrawFromGlobalIx(params, this.client.programId);
  }

  closePositionAltIx(params: ClosePositionAltParams): TransactionInstruction {
    return buildClosePositionAltIx(params, this.client.programId);
  }

  closePositionTokenAccountsIx(
    params: ClosePositionTokenAccountsParams,
    numOutcomes: number
  ): TransactionInstruction {
    return buildClosePositionTokenAccountsIx(
      params,
      numOutcomes,
      this.client.programId
    );
  }

  // ── Transaction builders (_tx convenience wrappers) ─────────────────

  redeemWinningsTx(
    params: RedeemWinningsParams,
    outcomeIndex: number
  ): Transaction {
    const ix = this.redeemWinningsIx(params, outcomeIndex);
    return new Transaction({ feePayer: params.user }).add(ix);
  }

  withdrawConditionalFromPositionTx(params: WithdrawConditionalFromPositionParams): Transaction {
    const ix = this.withdrawConditionalFromPositionIx(params);
    return new Transaction({ feePayer: params.user }).add(ix);
  }

  withdrawFromPositionTx(params: WithdrawFromPositionParams): Transaction {
    return this.withdrawConditionalFromPositionTx(params);
  }

  initPositionTokensTx(
    params: InitPositionTokensParams,
    numOutcomes: number
  ): Transaction {
    const ix = this.initPositionTokensIx(params, numOutcomes);
    return new Transaction({ feePayer: params.payer }).add(ix);
  }

  extendPositionTokensTx(
    params: ExtendPositionTokensParams,
    numOutcomes: number
  ): Transaction {
    const ix = this.extendPositionTokensIx(params, numOutcomes);
    return new Transaction({ feePayer: params.operator }).add(ix);
  }

  depositToGlobalTx(params: DepositToGlobalParams): Transaction {
    const ix = this.depositToGlobalIx(params);
    return new Transaction({ feePayer: params.user }).add(ix);
  }

  depositToGlobalTxWithAlt(
    params: DepositToGlobalParams,
    altContext: DepositToGlobalAltContext
  ): Transaction {
    const ix = this.depositToGlobalIxWithAlt(params, altContext);
    return new Transaction({ feePayer: params.user }).add(ix);
  }

  globalToMarketDepositTx(
    params: GlobalToMarketDepositParams,
    numOutcomes: number
  ): Transaction {
    const ix = this.globalToMarketDepositIx(params, numOutcomes);
    return new Transaction({ feePayer: params.user }).add(ix);
  }

  withdrawFromGlobalTx(params: WithdrawFromGlobalParams): Transaction {
    const ix = this.withdrawFromGlobalIx(params);
    return new Transaction({ feePayer: params.user }).add(ix);
  }

  closePositionAltTx(params: ClosePositionAltParams): Transaction {
    const ix = this.closePositionAltIx(params);
    return new Transaction({ feePayer: params.operator }).add(ix);
  }

  closePositionTokenAccountsTx(
    params: ClosePositionTokenAccountsParams,
    numOutcomes: number
  ): Transaction {
    const ix = this.closePositionTokenAccountsIx(params, numOutcomes);
    return new Transaction({ feePayer: params.operator }).add(ix);
  }

  // ── Builder factories ──────────────────────────────────────────────

  deposit(): DepositBuilder {
    return new DepositBuilder(this.client, this.client.depositSource);
  }

  merge(): MergeBuilder {
    return new MergeBuilder(this.client);
  }

  withdraw(): WithdrawBuilder {
    return new WithdrawBuilder(this.client, this.client.depositSource);
  }

  redeemWinnings(): RedeemWinningsBuilder {
    return new RedeemWinningsBuilder(this.client);
  }

  withdrawFromPosition(): WithdrawFromPositionBuilder {
    return new WithdrawFromPositionBuilder(this.client);
  }

  withdrawConditionalFromPosition(): WithdrawFromPositionBuilder {
    return new WithdrawFromPositionBuilder(this.client);
  }

  initPositionTokens(): InitPositionTokensBuilder {
    return new InitPositionTokensBuilder(this.client);
  }

  extendPositionTokens(): ExtendPositionTokensBuilder {
    return new ExtendPositionTokensBuilder(this.client);
  }

  depositToGlobal(): DepositToGlobalBuilder {
    return new DepositToGlobalBuilder(this.client);
  }

  withdrawFromGlobal(): WithdrawFromGlobalBuilder {
    return new WithdrawFromGlobalBuilder(this.client);
  }

  globalToMarketDeposit(): GlobalToMarketDepositBuilder {
    return new GlobalToMarketDepositBuilder(this.client);
  }

  // ── On-chain account fetchers (require Connection) ──────────────────

  async getOnchain(owner: PublicKey, market: PublicKey): Promise<ProgramPosition | null> {
    const connection = requireConnection(this.client);
    const positionPda = this.pda(owner, market);
    const accountInfo = await connection.getAccountInfo(positionPda);
    if (!accountInfo) {
      return null;
    }
    return deserializeProgramPosition(accountInfo.data as Buffer);
  }
}

function solLamports(amount: string | Decimal): bigint {
  // Exact scaling rejects rounding, negative values, excess precision, and
  // values beyond the unsigned amount accepted by Solana instructions.
  let value: bigint;
  try {
    value = exactScaledInteger(amount, 9);
  } catch (error) {
    throw SdkError.validation(
      `invalid SOL amount: ${error instanceof Error ? error.message : String(error)}`
    );
  }
  if (value > 0xffff_ffff_ffff_ffffn) {
    throw SdkError.validation("SOL amount exceeds the transaction u64 range");
  }
  return value;
}
