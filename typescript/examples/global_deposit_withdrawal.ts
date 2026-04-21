import { PublicKey, Transaction } from "@solana/web3.js";
import { getPositionAltPda, getPositionPda } from "../src/program";
import {
  confirmTransactionOrThrow,
  depositMint,
  formatError,
  login,
  market,
  rpcClient,
  getKeypair,
  runExample,
} from "./common";

async function main() {
  const client = rpcClient();
  const connection = client.rpc().inner();
  const keypair = getKeypair();
  await login(client, keypair);

  const m = await market(client);
  const marketPubkey = new PublicKey(m.pubkey);
  const dMint = depositMint(m);
  const amount = 1_000_000n;
  const depositAmount = amount * 2n; // deposit extra so global has funds after market transfer

  const [positionPda] = getPositionPda(keypair.publicKey, marketPubkey, client.programId);

  // Check if position already exists (init_position_tokens is one-time)
  const positionAccount = await connection.getAccountInfo(positionPda);
  const needsInit = positionAccount === null;

  if (needsInit) {
    // Init + extend in a single transaction.
    // Use "processed" commitment for getSlot to minimize staleness — the
    // on-chain CreateLookupTable instruction rejects slots that are too old.
    const maxAttempts = 5;
    for (let attempt = 1; attempt <= maxAttempts; attempt++) {
      try {
        const recentSlot = BigInt(await connection.getSlot("processed"));
        const [lookupTable] = getPositionAltPda(positionPda, recentSlot);

        const initIx = client.positions().initPositionTokens()
          .payer(keypair.publicKey)
          .user(keypair.publicKey)
          .market(marketPubkey)
          .depositMints([dMint])
          .recentSlot(recentSlot)
          .numOutcomes(m.outcomes.length)
          .buildIx();

        const extendIx = client.positions().extendPositionTokens()
          .payer(keypair.publicKey)
          .user(keypair.publicKey)
          .market(marketPubkey)
          .lookupTable(lookupTable)
          .depositMints([dMint])
          .numOutcomes(m.outcomes.length)
          .buildIx();

        const { blockhash, lastValidBlockHeight } = await client.rpc().getLatestBlockhash();
        const tx = new Transaction({ feePayer: keypair.publicKey, blockhash, lastValidBlockHeight })
          .add(initIx)
          .add(extendIx);
        tx.sign(keypair);
        const signature = await connection.sendRawTransaction(tx.serialize(), {
          skipPreflight: true,
        });
        await confirmTransactionOrThrow(connection, signature, {
          blockhash,
          lastValidBlockHeight,
        });
        console.log(`init_position_tokens: confirmed ${signature}`);
        break;
      } catch (error: unknown) {
        const message = formatError(error);
        const retryable = message.includes("is not a recent slot")
          || message.includes("UninitializedAccount")
          || message.includes("already in use");
        if (attempt < maxAttempts && retryable) {
          console.log(`init_position_tokens: retrying (${attempt}/${maxAttempts}): ${message.slice(0, 80)}`);
          await new Promise((resolve) => setTimeout(resolve, 2000));
          continue;
        }
        throw error;
      }
    }
  } else {
    console.log("position already initialized, skipping init_position_tokens + extend");
  }

  const instructions: Array<[string, import("@solana/web3.js").TransactionInstruction]> = [];

  // 3. Deposit to global — fund the global pool with collateral
  instructions.push([
    "deposit_to_global",
    client.positions().depositToGlobal()
      .user(keypair.publicKey)
      .mint(dMint)
      .amount(depositAmount)
      .buildIx(),
  ]);

  // 4. Global to market deposit — move capital into a specific market
  instructions.push([
    "global_to_market_deposit",
    client.positions().globalToMarketDeposit()
      .user(keypair.publicKey)
      .market(marketPubkey)
      .mint(dMint)
      .amount(amount)
      .numOutcomes(m.outcomes.length)
      .buildIx(),
  ]);

  // 5. Withdraw from global — pull remaining tokens back out
  instructions.push([
    "withdraw_from_global",
    client.positions().withdrawFromGlobal()
      .user(keypair.publicKey)
      .mint(dMint)
      .amount(amount)
      .buildIx(),
  ]);

  // 6. Merge — burn the complete set of conditional tokens minted in step 4
  //    back to the deposit asset, returning the collateral to the user's
  //    token account. Closes out the market position so the full example is
  //    net-neutral on the wallet's balance, the global pool, and the market
  //    position across CI runs.
  instructions.push([
    "merge",
    client.positions().merge()
      .user(keypair.publicKey)
      .market(m)
      .mint(dMint)
      .amount(amount)
      .buildIx(),
  ]);

  for (const [name, ix] of instructions) {
    const { blockhash, lastValidBlockHeight } = await client.rpc().getLatestBlockhash();
    const tx = new Transaction({ feePayer: keypair.publicKey, blockhash, lastValidBlockHeight }).add(ix);
    tx.sign(keypair);
    const signature = await connection.sendRawTransaction(tx.serialize());
    await confirmTransactionOrThrow(connection, signature, {
      blockhash,
      lastValidBlockHeight,
    });
    console.log(`${name}: confirmed ${signature}`);
  }

  // ── Unified deposit/withdraw/merge builders ─────────────────────────
  //
  // Deposit and withdraw builders dispatch based on the client's deposit
  // source setting (or a per-call override). Merge is market-only.

  // Deposit — explicitly override to Global
  const globalDepositIx = client
    .positions()
    .deposit()
    .user(keypair.publicKey)
    .mint(dMint)
    .amount(amount)
    .withGlobalDepositSource()
    .buildIx();
  console.log(`builder global deposit ix: ${globalDepositIx.keys.length} accounts`);

  // Deposit — explicitly override to Market (mints conditional tokens)
  const marketDepositIx = client
    .positions()
    .deposit()
    .user(keypair.publicKey)
    .mint(dMint)
    .amount(amount)
    .withMarketDepositSource(m)
    .buildIx();
  console.log(`builder market deposit ix: ${marketDepositIx.keys.length} accounts`);

  // Withdraw — Global mode (global pool -> wallet)
  const globalWithdrawIx = client
    .positions()
    .withdraw()
    .user(keypair.publicKey)
    .mint(dMint)
    .amount(amount)
    .withGlobalDepositSource()
    .buildIx();
  console.log(`builder global withdraw ix: ${globalWithdrawIx.keys.length} accounts`);

  // Withdraw — Market mode (position ATA -> user's wallet)
  const marketWithdrawIx = client
    .positions()
    .withdraw()
    .user(keypair.publicKey)
    .mint(dMint)
    .amount(amount)
    .withMarketDepositSource(m)
    .outcomeIndex(0)
    .token2022(true)
    .buildIx();
  console.log(`builder market withdraw ix: ${marketWithdrawIx.keys.length} accounts`);

  // Merge — burns complete set of conditional tokens, releases collateral
  const mergeIx = client
    .positions()
    .merge()
    .user(keypair.publicKey)
    .market(m)
    .mint(dMint)
    .amount(amount)
    .buildIx();
  console.log(`builder merge ix: ${mergeIx.keys.length} accounts`);
}

void runExample(main);
