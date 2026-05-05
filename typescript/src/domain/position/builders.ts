import { PublicKey, Transaction, type TransactionInstruction } from "@solana/web3.js";
import type { ClientContext } from "../../context";
import { resolveDepositSource, signAndSubmitTx } from "../../context";
import { SdkError } from "../../error";
import {
  buildDepositIx,
  buildDepositToGlobalIx,
  buildDepositToGlobalIxWithAlt,
  buildExtendPositionTokensIx,
  buildGlobalToMarketDepositIx,
  buildMergeIx,
  buildRedeemWinningsIx,
  buildWithdrawFromGlobalIx,
  buildWithdrawFromPositionIx,
  buildInitPositionTokensIx,
} from "../../program/instructions";
import { DepositSource } from "../../shared";
import type { DepositToGlobalAltContext } from "../../program/types";
import type { Market } from "../market";

// ─── Helpers ────────────────────────────────────────────────────────────────

function requireField<T>(value: T | undefined, field: string): T {
  if (value === undefined) {
    throw SdkError.validation(`${field} is required`);
  }
  return value;
}

// ─── DepositBuilder ─────────────────────────────────────────────────────────

export class DepositBuilder {
  private readonly client: ClientContext;
  private userValue?: PublicKey;
  private mintValue?: PublicKey;
  private amountValue?: bigint;
  private marketValue?: Market;
  private depositSourceValue?: DepositSource;

  constructor(client: ClientContext, depositSource: DepositSource) {
    this.client = client;
    this.depositSourceValue = depositSource;
  }

  user(user: PublicKey): this {
    this.userValue = user;
    return this;
  }

  mint(mint: PublicKey): this {
    this.mintValue = mint;
    return this;
  }

  amount(amount: bigint): this {
    this.amountValue = amount;
    return this;
  }

  market(market: Market): this {
    this.marketValue = market;
    return this;
  }

  depositSource(source: DepositSource): this {
    this.depositSourceValue = source;
    return this;
  }

  withMarketDepositSource(market: Market): this {
    this.depositSourceValue = DepositSource.Market;
    this.marketValue = market;
    return this;
  }

  withGlobalDepositSource(): this {
    this.depositSourceValue = DepositSource.Global;
    return this;
  }

  buildIx(): TransactionInstruction {
    const user = requireField(this.userValue, "user");
    const mint = requireField(this.mintValue, "mint");
    const amount = requireField(this.amountValue, "amount");
    const source = resolveDepositSource(this.client, this.depositSourceValue);

    switch (source) {
      case DepositSource.Global:
        return buildDepositToGlobalIx({ user, mint, amount }, this.client.programId);
      case DepositSource.Market: {
        const market = this.marketValue;
        if (!market) {
          throw SdkError.missingMarketContext("market is required for Market deposit source");
        }
        const marketPubkey = new PublicKey(market.pubkey);
        const numOutcomes = market.outcomes.length;
        return buildDepositIx(
          { user, market: marketPubkey, depositMint: mint, amount },
          numOutcomes,
          this.client.programId,
        );
      }
    }
  }

  buildTx(): Transaction {
    const user = requireField(this.userValue, "user");
    const ix = this.buildIx();
    return new Transaction({ feePayer: user }).add(ix);
  }

  async signAndSubmit(): Promise<string> {
    const tx = this.buildTx();
    return signAndSubmitTx(this.client, tx);
  }
}

// ─── WithdrawBuilder ────────────────────────────────────────────────────────

export class WithdrawBuilder {
  private readonly client: ClientContext;
  private userValue?: PublicKey;
  private mintValue?: PublicKey;
  private amountValue?: bigint;
  private marketValue?: Market;
  private depositSourceValue?: DepositSource;
  private outcomeIndexValue?: number;
  private isToken2022Value = false;

  constructor(client: ClientContext, depositSource: DepositSource) {
    this.client = client;
    this.depositSourceValue = depositSource;
  }

  user(user: PublicKey): this {
    this.userValue = user;
    return this;
  }

  mint(mint: PublicKey): this {
    this.mintValue = mint;
    return this;
  }

  amount(amount: bigint): this {
    this.amountValue = amount;
    return this;
  }

  market(market: Market): this {
    this.marketValue = market;
    return this;
  }

  depositSource(source: DepositSource): this {
    this.depositSourceValue = source;
    return this;
  }

  outcomeIndex(index: number): this {
    this.outcomeIndexValue = index;
    return this;
  }

  token2022(isToken2022: boolean): this {
    this.isToken2022Value = isToken2022;
    return this;
  }

  withMarketDepositSource(market: Market): this {
    this.depositSourceValue = DepositSource.Market;
    this.marketValue = market;
    return this;
  }

  withGlobalDepositSource(): this {
    this.depositSourceValue = DepositSource.Global;
    return this;
  }

  buildIx(): TransactionInstruction {
    const user = requireField(this.userValue, "user");
    const mint = requireField(this.mintValue, "mint");
    const amount = requireField(this.amountValue, "amount");
    const source = resolveDepositSource(this.client, this.depositSourceValue);

    switch (source) {
      case DepositSource.Global:
        return buildWithdrawFromGlobalIx({ user, mint, amount }, this.client.programId);
      case DepositSource.Market: {
        const market = this.marketValue;
        if (!market) {
          throw SdkError.missingMarketContext("market is required for Market withdrawal");
        }
        const marketPubkey = new PublicKey(market.pubkey);
        const outcomeIndex = requireField(this.outcomeIndexValue, "outcome_index");
        return buildWithdrawFromPositionIx(
          { user, market: marketPubkey, mint, amount, outcomeIndex },
          this.isToken2022Value,
          this.client.programId,
        );
      }
    }
  }

  buildTx(): Transaction {
    const user = requireField(this.userValue, "user");
    const ix = this.buildIx();
    return new Transaction({ feePayer: user }).add(ix);
  }

  async signAndSubmit(): Promise<string> {
    const tx = this.buildTx();
    return signAndSubmitTx(this.client, tx);
  }
}

// ─── MergeBuilder ──────────────────────────────────────────────────────────

export class MergeBuilder {
  private readonly client: ClientContext;
  private userValue?: PublicKey;
  private mintValue?: PublicKey;
  private amountValue?: bigint;
  private marketValue?: Market;

  constructor(client: ClientContext) {
    this.client = client;
  }

  user(user: PublicKey): this {
    this.userValue = user;
    return this;
  }

  mint(mint: PublicKey): this {
    this.mintValue = mint;
    return this;
  }

  amount(amount: bigint): this {
    this.amountValue = amount;
    return this;
  }

  market(market: Market): this {
    this.marketValue = market;
    return this;
  }

  buildIx(): TransactionInstruction {
    const user = requireField(this.userValue, "user");
    const mint = requireField(this.mintValue, "mint");
    const amount = requireField(this.amountValue, "amount");
    const market = this.marketValue;
    if (!market) {
      throw SdkError.missingMarketContext("market is required for merge");
    }
    const marketPubkey = new PublicKey(market.pubkey);
    const numOutcomes = market.outcomes.length;

    return buildMergeIx(
      { user, market: marketPubkey, depositMint: mint, amount },
      numOutcomes,
      this.client.programId,
    );
  }

  buildTx(): Transaction {
    const user = requireField(this.userValue, "user");
    const ix = this.buildIx();
    return new Transaction({ feePayer: user }).add(ix);
  }

  async signAndSubmit(): Promise<string> {
    const tx = this.buildTx();
    return signAndSubmitTx(this.client, tx);
  }
}

// ─── RedeemWinningsBuilder ──────────────────────────────────────────────────

export class RedeemWinningsBuilder {
  private readonly client: ClientContext;
  private userValue?: PublicKey;
  private marketValue?: PublicKey;
  private mintValue?: PublicKey;
  private amountValue?: bigint;
  private outcomeIndexValue?: number;

  constructor(client: ClientContext) {
    this.client = client;
  }

  user(user: PublicKey): this {
    this.userValue = user;
    return this;
  }

  market(market: PublicKey): this {
    this.marketValue = market;
    return this;
  }

  mint(mint: PublicKey): this {
    this.mintValue = mint;
    return this;
  }

  amount(amount: bigint): this {
    this.amountValue = amount;
    return this;
  }

  outcomeIndex(index: number): this {
    this.outcomeIndexValue = index;
    return this;
  }

  buildIx(): TransactionInstruction {
    const user = requireField(this.userValue, "user");
    const market = requireField(this.marketValue, "market");
    const depositMint = requireField(this.mintValue, "mint");
    const amount = requireField(this.amountValue, "amount");
    const outcomeIndex = requireField(this.outcomeIndexValue, "outcome_index");

    return buildRedeemWinningsIx(
      { user, market, depositMint, amount },
      outcomeIndex,
      this.client.programId,
    );
  }

  buildTx(): Transaction {
    const user = requireField(this.userValue, "user");
    const ix = this.buildIx();
    return new Transaction({ feePayer: user }).add(ix);
  }

  async signAndSubmit(): Promise<string> {
    const tx = this.buildTx();
    return signAndSubmitTx(this.client, tx);
  }
}

// ─── WithdrawFromPositionBuilder ────────────────────────────────────────────

export class WithdrawFromPositionBuilder {
  private readonly client: ClientContext;
  private userValue?: PublicKey;
  private marketValue?: PublicKey;
  private mintValue?: PublicKey;
  private amountValue?: bigint;
  private outcomeIndexValue?: number;
  private isToken2022Value = false;

  constructor(client: ClientContext) {
    this.client = client;
  }

  user(user: PublicKey): this {
    this.userValue = user;
    return this;
  }

  market(market: PublicKey): this {
    this.marketValue = market;
    return this;
  }

  mint(mint: PublicKey): this {
    this.mintValue = mint;
    return this;
  }

  amount(amount: bigint): this {
    this.amountValue = amount;
    return this;
  }

  outcomeIndex(index: number): this {
    this.outcomeIndexValue = index;
    return this;
  }

  token2022(isToken2022: boolean): this {
    this.isToken2022Value = isToken2022;
    return this;
  }

  buildIx(): TransactionInstruction {
    const user = requireField(this.userValue, "user");
    const market = requireField(this.marketValue, "market");
    const mint = requireField(this.mintValue, "mint");
    const amount = requireField(this.amountValue, "amount");
    const outcomeIndex = requireField(this.outcomeIndexValue, "outcome_index");

    return buildWithdrawFromPositionIx(
      { user, market, mint, amount, outcomeIndex },
      this.isToken2022Value,
      this.client.programId,
    );
  }

  buildTx(): Transaction {
    const user = requireField(this.userValue, "user");
    const ix = this.buildIx();
    return new Transaction({ feePayer: user }).add(ix);
  }

  async signAndSubmit(): Promise<string> {
    const tx = this.buildTx();
    return signAndSubmitTx(this.client, tx);
  }
}

// ─── InitPositionTokensBuilder ──────────────────────────────────────────────

export class InitPositionTokensBuilder {
  private readonly client: ClientContext;
  private payerValue?: PublicKey;
  private userValue?: PublicKey;
  private marketValue?: PublicKey;
  private depositMintsValue?: PublicKey[];
  private recentSlotValue?: bigint;
  private numOutcomesValue?: number;

  constructor(client: ClientContext) {
    this.client = client;
  }

  payer(payer: PublicKey): this {
    this.payerValue = payer;
    return this;
  }

  user(user: PublicKey): this {
    this.userValue = user;
    return this;
  }

  market(market: PublicKey): this {
    this.marketValue = market;
    return this;
  }

  depositMints(mints: PublicKey[]): this {
    this.depositMintsValue = mints;
    return this;
  }

  recentSlot(slot: bigint): this {
    this.recentSlotValue = slot;
    return this;
  }

  numOutcomes(n: number): this {
    this.numOutcomesValue = n;
    return this;
  }

  buildIx(): TransactionInstruction {
    const payer = requireField(this.payerValue, "payer");
    const user = requireField(this.userValue, "user");
    const market = requireField(this.marketValue, "market");
    const depositMints = requireField(this.depositMintsValue, "deposit_mints");
    const recentSlot = requireField(this.recentSlotValue, "recent_slot");
    const numOutcomes = requireField(this.numOutcomesValue, "num_outcomes");

    return buildInitPositionTokensIx(
      { payer, user, market, depositMints, recentSlot },
      numOutcomes,
      this.client.programId,
    );
  }

  buildTx(): Transaction {
    const payer = requireField(this.payerValue, "payer");
    const ix = this.buildIx();
    return new Transaction({ feePayer: payer }).add(ix);
  }

  async signAndSubmit(): Promise<string> {
    const tx = this.buildTx();
    return signAndSubmitTx(this.client, tx);
  }
}

// ─── ExtendPositionTokensBuilder ────────────────────────────────────────────

export class ExtendPositionTokensBuilder {
  private readonly client: ClientContext;
  private operatorValue?: PublicKey;
  private userValue?: PublicKey;
  private marketValue?: PublicKey;
  private lookupTableValue?: PublicKey;
  private depositMintsValue?: PublicKey[];
  private numOutcomesValue?: number;

  constructor(client: ClientContext) {
    this.client = client;
  }

  operator(operator: PublicKey): this {
    this.operatorValue = operator;
    return this;
  }

  user(user: PublicKey): this {
    this.userValue = user;
    return this;
  }

  market(market: PublicKey): this {
    this.marketValue = market;
    return this;
  }

  lookupTable(lookupTable: PublicKey): this {
    this.lookupTableValue = lookupTable;
    return this;
  }

  depositMints(mints: PublicKey[]): this {
    this.depositMintsValue = mints;
    return this;
  }

  numOutcomes(n: number): this {
    this.numOutcomesValue = n;
    return this;
  }

  buildIx(): TransactionInstruction {
    const operator = requireField(this.operatorValue, "operator");
    const user = requireField(this.userValue, "user");
    const market = requireField(this.marketValue, "market");
    const lookupTable = requireField(this.lookupTableValue, "lookup_table");
    const depositMints = requireField(this.depositMintsValue, "deposit_mints");
    const numOutcomes = requireField(this.numOutcomesValue, "num_outcomes");

    return buildExtendPositionTokensIx(
      { operator, user, market, lookupTable, depositMints },
      numOutcomes,
      this.client.programId,
    );
  }

  buildTx(): Transaction {
    const operator = requireField(this.operatorValue, "operator");
    const ix = this.buildIx();
    return new Transaction({ feePayer: operator }).add(ix);
  }

  async signAndSubmit(): Promise<string> {
    const tx = this.buildTx();
    return signAndSubmitTx(this.client, tx);
  }
}

// ─── DepositToGlobalBuilder ─────────────────────────────────────────────────

export class DepositToGlobalBuilder {
  private readonly client: ClientContext;
  private userValue?: PublicKey;
  private mintValue?: PublicKey;
  private amountValue?: bigint;
  private altContextValue?: DepositToGlobalAltContext;

  constructor(client: ClientContext) {
    this.client = client;
  }

  user(user: PublicKey): this {
    this.userValue = user;
    return this;
  }

  mint(mint: PublicKey): this {
    this.mintValue = mint;
    return this;
  }

  amount(amount: bigint): this {
    this.amountValue = amount;
    return this;
  }

  createAlt(recentSlot: bigint): this {
    this.altContextValue = { kind: "create", recentSlot };
    return this;
  }

  extendAlt(lookupTable: PublicKey): this {
    this.altContextValue = { kind: "extend", lookupTable };
    return this;
  }

  buildIx(): TransactionInstruction {
    const user = requireField(this.userValue, "user");
    const mint = requireField(this.mintValue, "mint");
    const amount = requireField(this.amountValue, "amount");

    const params = { user, mint, amount };
    return this.altContextValue
      ? buildDepositToGlobalIxWithAlt(params, this.altContextValue, this.client.programId)
      : buildDepositToGlobalIx(params, this.client.programId);
  }

  buildTx(): Transaction {
    const user = requireField(this.userValue, "user");
    const ix = this.buildIx();
    return new Transaction({ feePayer: user }).add(ix);
  }

  async signAndSubmit(): Promise<string> {
    const tx = this.buildTx();
    return signAndSubmitTx(this.client, tx);
  }
}

// ─── WithdrawFromGlobalBuilder ──────────────────────────────────────────────

export class WithdrawFromGlobalBuilder {
  private readonly client: ClientContext;
  private userValue?: PublicKey;
  private mintValue?: PublicKey;
  private amountValue?: bigint;

  constructor(client: ClientContext) {
    this.client = client;
  }

  user(user: PublicKey): this {
    this.userValue = user;
    return this;
  }

  mint(mint: PublicKey): this {
    this.mintValue = mint;
    return this;
  }

  amount(amount: bigint): this {
    this.amountValue = amount;
    return this;
  }

  buildIx(): TransactionInstruction {
    const user = requireField(this.userValue, "user");
    const mint = requireField(this.mintValue, "mint");
    const amount = requireField(this.amountValue, "amount");

    return buildWithdrawFromGlobalIx({ user, mint, amount }, this.client.programId);
  }

  buildTx(): Transaction {
    const user = requireField(this.userValue, "user");
    const ix = this.buildIx();
    return new Transaction({ feePayer: user }).add(ix);
  }

  async signAndSubmit(): Promise<string> {
    const tx = this.buildTx();
    return signAndSubmitTx(this.client, tx);
  }
}

// ─── GlobalToMarketDepositBuilder ───────────────────────────────────────────

export class GlobalToMarketDepositBuilder {
  private readonly client: ClientContext;
  private userValue?: PublicKey;
  private marketValue?: PublicKey;
  private mintValue?: PublicKey;
  private amountValue?: bigint;
  private numOutcomesValue?: number;

  constructor(client: ClientContext) {
    this.client = client;
  }

  user(user: PublicKey): this {
    this.userValue = user;
    return this;
  }

  market(market: PublicKey): this {
    this.marketValue = market;
    return this;
  }

  mint(mint: PublicKey): this {
    this.mintValue = mint;
    return this;
  }

  amount(amount: bigint): this {
    this.amountValue = amount;
    return this;
  }

  numOutcomes(n: number): this {
    this.numOutcomesValue = n;
    return this;
  }

  buildIx(): TransactionInstruction {
    const user = requireField(this.userValue, "user");
    const market = requireField(this.marketValue, "market");
    const depositMint = requireField(this.mintValue, "mint");
    const amount = requireField(this.amountValue, "amount");
    const numOutcomes = requireField(this.numOutcomesValue, "num_outcomes");

    return buildGlobalToMarketDepositIx(
      { user, market, depositMint, amount },
      numOutcomes,
      this.client.programId,
    );
  }

  buildTx(): Transaction {
    const user = requireField(this.userValue, "user");
    const ix = this.buildIx();
    return new Transaction({ feePayer: user }).add(ix);
  }

  async signAndSubmit(): Promise<string> {
    const tx = this.buildTx();
    return signAndSubmitTx(this.client, tx);
  }
}
