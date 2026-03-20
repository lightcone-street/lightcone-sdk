import { PublicKey, Transaction } from "@solana/web3.js";
import { getPositionAltPda, getPositionPda } from "../src/program";
import { depositMint, login, market, numOutcomes, rpcClient, wallet } from "./common";

async function main() {
  const client = rpcClient();
  const connection = client.rpc().inner();
  const keypair = wallet();
  await login(client, keypair);

  const m = await market(client);
  const marketPubkey = new PublicKey(m.pubkey);
  const dMint = depositMint(m);
  const outcomes = numOutcomes(m);
  const amount = 1_000_000n;

  // 1. Init position tokens — one-time setup per market (creates position + ALT)
  const recentSlot = BigInt(await connection.getSlot());

  const [positionPda] = getPositionPda(keypair.publicKey, marketPubkey);
  const [lookupTable] = getPositionAltPda(positionPda, recentSlot);

  const instructions: Array<[string, import("@solana/web3.js").TransactionInstruction]> = [
    // 1. Init position tokens
    [
      "init_position_tokens",
      client.positions().initPositionTokens()
        .payer(keypair.publicKey)
        .user(keypair.publicKey)
        .market(marketPubkey)
        .depositMints([dMint])
        .recentSlot(recentSlot)
        .numOutcomes(outcomes)
        .buildIx(),
    ],
    // 2. Deposit to global — fund the global pool with collateral
    [
      "deposit_to_global",
      client.positions().depositToGlobal()
        .user(keypair.publicKey)
        .mint(dMint)
        .amount(amount)
        .buildIx(),
    ],
    // 3. Global to market deposit — move capital into a specific market
    [
      "global_to_market_deposit",
      client.positions().globalToMarketDeposit()
        .user(keypair.publicKey)
        .market(marketPubkey)
        .mint(dMint)
        .amount(amount)
        .numOutcomes(outcomes)
        .buildIx(),
    ],
    // 4. Extend position tokens — add a new deposit mint to an existing ALT
    [
      "extend_position_tokens",
      client.positions().extendPositionTokens()
        .payer(keypair.publicKey)
        .user(keypair.publicKey)
        .market(marketPubkey)
        .lookupTable(lookupTable)
        .depositMints([dMint])
        .numOutcomes(outcomes)
        .buildIx(),
    ],
    // 5. Withdraw from global — pull tokens back out of the global pool
    [
      "withdraw_from_global",
      client.positions().withdrawFromGlobal()
        .user(keypair.publicKey)
        .mint(dMint)
        .amount(amount)
        .buildIx(),
    ],
  ];

  for (const [name, ix] of instructions) {
    const { blockhash, lastValidBlockHeight } = await client.rpc().getLatestBlockhash();
    const tx = new Transaction({ feePayer: keypair.publicKey, blockhash, lastValidBlockHeight }).add(ix);
    tx.sign(keypair);
    const signature = await connection.sendRawTransaction(tx.serialize());
    await connection.confirmTransaction(signature);
    console.log(`${name}: confirmed ${signature}`);
  }

  // ── Unified deposit/withdraw builders ──────────────────────────────
  //
  // These builders dispatch based on the client's deposit source setting
  // (or a per-call override).

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

  // Withdraw — Market mode (burns conditional tokens -> wallet collateral)
  const marketWithdrawIx = client
    .positions()
    .withdraw()
    .user(keypair.publicKey)
    .mint(dMint)
    .amount(amount)
    .withMarketDepositSource(m)
    .buildIx();
  console.log(`builder market withdraw ix: ${marketWithdrawIx.keys.length} accounts`);
}

main().catch(console.error);
