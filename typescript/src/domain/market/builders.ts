import { PublicKey, Transaction, type TransactionInstruction } from "@solana/web3.js";
import type { ClientContext } from "../../context";
import { signAndSubmitTx } from "../../context";
import { SdkError } from "../../error";
import {
  buildMintCompleteSetIx,
  buildMergeCompleteSetIx,
} from "../../program/instructions";

function require<T>(value: T | undefined, field: string): T {
  if (value === undefined) {
    throw SdkError.validation(`${field} is required`);
  }
  return value;
}

// ─── MintCompleteSetBuilder ─────────────────────────────────────────────────

export class MintCompleteSetBuilder {
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
    const user = require(this.userValue, "user");
    const market = require(this.marketValue, "market");
    const depositMint = require(this.mintValue, "mint");
    const amount = require(this.amountValue, "amount");
    const numOutcomes = require(this.numOutcomesValue, "num_outcomes");

    return buildMintCompleteSetIx(
      { user, market, depositMint, amount },
      numOutcomes,
      this.client.programId,
    );
  }

  buildTx(): Transaction {
    const user = require(this.userValue, "user");
    const ix = this.buildIx();
    return new Transaction({ feePayer: user }).add(ix);
  }

  async signAndSubmit(): Promise<string> {
    const tx = this.buildTx();
    return signAndSubmitTx(this.client, tx);
  }
}

// ─── MergeCompleteSetBuilder ────────────────────────────────────────────────

export class MergeCompleteSetBuilder {
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
    const user = require(this.userValue, "user");
    const market = require(this.marketValue, "market");
    const depositMint = require(this.mintValue, "mint");
    const amount = require(this.amountValue, "amount");
    const numOutcomes = require(this.numOutcomesValue, "num_outcomes");

    return buildMergeCompleteSetIx(
      { user, market, depositMint, amount },
      numOutcomes,
      this.client.programId,
    );
  }

  buildTx(): Transaction {
    const user = require(this.userValue, "user");
    const ix = this.buildIx();
    return new Transaction({ feePayer: user }).add(ix);
  }

  async signAndSubmit(): Promise<string> {
    const tx = this.buildTx();
    return signAndSubmitTx(this.client, tx);
  }
}
