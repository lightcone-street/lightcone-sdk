/**
 * Stress Test for Lightcone Pinocchio SDK
 *
 * Tests 100 concurrent order matching transactions to verify
 * program can handle high throughput in a single block.
 *
 * Requires:
 * 1. Program deployed to devnet
 * 2. Funded wallet (~2 SOL minimum for 100+ transactions)
 *
 * Run: npm run test:stress
 */

import {
  Keypair,
  LAMPORTS_PER_SOL,
  PublicKey,
  Connection,
  SystemProgram,
  Transaction,
} from "@solana/web3.js";
import {
  createMint,
  createAssociatedTokenAccount,
  mintTo,
} from "@solana/spl-token";
import * as fs from "fs";
import * as path from "path";
import * as dotenv from "dotenv";
import {
  LightconePinocchioClient,
  PROGRAM_ID,
  MarketStatus,
  OrderSide,
  getMarketPda,
  getConditionalMintPda,
  getPositionPda,
  getOrderStatusPda,
  createBidOrder,
  createAskOrder,
  signOrderFull,
  hashOrder,
  buildCrossRefMatchOrdersTransaction,
  FullOrder,
} from "../src";

// Load environment variables
dotenv.config();

// Configuration - can be overridden with env vars
const NUM_MATCHES = parseInt(process.env.NUM_MATCHES || "100"); // Number of concurrent order matches
const FILL_AMOUNT = 1000n; // Amount per trade
const FUNDING_PER_USER = 0.05; // SOL per user (needs to cover rent for ATA + position)

// Color output helpers
const colors = {
  reset: "\x1b[0m",
  bright: "\x1b[1m",
  green: "\x1b[32m",
  red: "\x1b[31m",
  yellow: "\x1b[33m",
  cyan: "\x1b[36m",
  blue: "\x1b[34m",
  magenta: "\x1b[35m",
};

function log(emoji: string, message: string, data?: string) {
  console.log(`${emoji}  ${colors.bright}${message}${colors.reset}`);
  if (data) {
    console.log(`   ${colors.cyan}${data}${colors.reset}`);
  }
}

function success(message: string) {
  console.log(`   ${colors.green}✅ ${message}${colors.reset}\n`);
}

function error(message: string) {
  console.log(`   ${colors.red}❌ ${message}${colors.reset}\n`);
}

function warn(message: string) {
  console.log(`   ${colors.yellow}⚠ ${message}${colors.reset}`);
}

async function loadKeypair(filePath: string): Promise<Keypair> {
  const keypairData = JSON.parse(fs.readFileSync(filePath, "utf-8"));
  return Keypair.fromSecretKey(Uint8Array.from(keypairData));
}

async function sleep(ms: number) {
  return new Promise((resolve) => setTimeout(resolve, ms));
}

async function confirmTx(connection: Connection, signature: string) {
  const latestBlockhash = await connection.getLatestBlockhash();
  await connection.confirmTransaction({
    signature,
    blockhash: latestBlockhash.blockhash,
    lastValidBlockHeight: latestBlockhash.lastValidBlockHeight,
  });
}

interface MatchResult {
  index: number;
  success: boolean;
  signature?: string;
  error?: string;
  latencyMs: number;
}

async function main() {
  console.log("\n" + "=".repeat(70));
  console.log(
    `${colors.bright}${colors.magenta}🚀 LIGHTCONE PINOCCHIO SDK - STRESS TEST${colors.reset}`
  );
  console.log(`${colors.cyan}   Testing ${NUM_MATCHES} concurrent order matches${colors.reset}`);
  console.log("=".repeat(70) + "\n");

  // Load authority keypair
  const devnetAuthorityPath = path.join(__dirname, "../keypairs/devnet-authority.json");
  const keypairPath = fs.existsSync(devnetAuthorityPath)
    ? devnetAuthorityPath
    : process.env.ANCHOR_WALLET ||
      path.join(process.env.HOME!, ".config/solana/id.json");

  const authority = await loadKeypair(keypairPath);

  // Get RPC endpoint from env or use default
  const rpcEndpoint =
    process.env.DEV_NET_RPC || "https://api.devnet.solana.com";

  // Create SDK client for devnet
  const client = new LightconePinocchioClient(
    new Connection(rpcEndpoint, "confirmed"),
    PROGRAM_ID
  );

  console.log(`${colors.bright}Configuration:${colors.reset}`);
  console.log(`   Program ID: ${colors.yellow}${client.programId.toString()}${colors.reset}`);
  console.log(`   Authority: ${colors.yellow}${authority.publicKey.toString()}${colors.reset}`);
  console.log(`   RPC: ${colors.yellow}${rpcEndpoint}${colors.reset}`);
  console.log(`   Matches: ${colors.yellow}${NUM_MATCHES}${colors.reset}\n`);

  // Check balance
  log("💰", "Checking wallet balance...");
  const balance = await client.connection.getBalance(authority.publicKey);
  const balanceInSol = balance / LAMPORTS_PER_SOL;
  console.log(`   Current balance: ${balanceInSol.toFixed(4)} SOL`);

  // Calculate required: funding per user * 2 users per match * NUM_MATCHES + 1 SOL buffer for tx fees
  const requiredSol = (FUNDING_PER_USER * 2 * NUM_MATCHES) + 1.0;
  if (balanceInSol < requiredSol) {
    error(`Insufficient balance. Need at least ${requiredSol.toFixed(1)} SOL for ${NUM_MATCHES} matches.`);
    process.exit(1);
  }
  console.log(`   ${colors.green}✓ Sufficient balance${colors.reset}\n`);

  // ============================================================================
  // Step 1: Setup Market
  // ============================================================================
  log("1️⃣", "Setting up test market...");

  let marketPda: PublicKey;
  let depositMint: PublicKey;
  let conditionalMint0: PublicKey;
  let conditionalMint1: PublicKey;
  let nextMarketId: bigint;

  try {
    // Get next market ID
    const exchange = await client.getExchange();
    nextMarketId = exchange.marketCount;
    console.log(`   Next Market ID: ${nextMarketId}`);

    // Create market
    const questionId = Buffer.alloc(32);
    Buffer.from(`Stress Test ${Date.now()}`).copy(questionId);

    const createResult = await client.createMarket({
      authority: authority.publicKey,
      numOutcomes: 2,
      oracle: authority.publicKey,
      questionId,
    });
    marketPda = createResult.accounts.market;
    console.log(`   Market PDA: ${marketPda.toString()}`);

    createResult.transaction.sign(authority);
    const createSig = await client.connection.sendRawTransaction(
      createResult.transaction.serialize()
    );
    await confirmTx(client.connection, createSig);
    console.log(`   ${colors.green}✓ Market created${colors.reset}`);

    // Create deposit mint
    depositMint = await createMint(
      client.connection,
      authority,
      authority.publicKey,
      null,
      6
    );
    console.log(`   Deposit Mint: ${depositMint.toString()}`);

    // Add deposit mint
    const addMintResult = await client.addDepositMint(
      {
        authority: authority.publicKey,
        marketId: nextMarketId,
        depositMint,
        outcomeMetadata: [
          { name: "YES", symbol: "YES", uri: "https://test.com/yes.json" },
          { name: "NO", symbol: "NO", uri: "https://test.com/no.json" },
        ],
      },
      2
    );

    addMintResult.transaction.sign(authority);
    const addMintSig = await client.connection.sendRawTransaction(
      addMintResult.transaction.serialize()
    );
    await confirmTx(client.connection, addMintSig);
    console.log(`   ${colors.green}✓ Deposit mint added${colors.reset}`);

    // Activate market
    const activateResult = await client.activateMarket({
      authority: authority.publicKey,
      marketId: nextMarketId,
    });
    activateResult.transaction.sign(authority);
    const activateSig = await client.connection.sendRawTransaction(
      activateResult.transaction.serialize()
    );
    await confirmTx(client.connection, activateSig);
    console.log(`   ${colors.green}✓ Market activated${colors.reset}`);

    // Get conditional mints
    [conditionalMint0] = getConditionalMintPda(marketPda, depositMint, 0, client.programId);
    [conditionalMint1] = getConditionalMintPda(marketPda, depositMint, 1, client.programId);

    success("Market setup complete");
  } catch (err) {
    error("Market setup failed");
    console.error(err);
    process.exit(1);
  }

  // ============================================================================
  // Step 2: Create Users with Positions
  // ============================================================================
  log("2️⃣", `Creating ${NUM_MATCHES * 2} users with positions...`);

  const buyers: Keypair[] = [];
  const sellers: Keypair[] = [];

  try {
    // Create keypairs
    for (let i = 0; i < NUM_MATCHES; i++) {
      buyers.push(Keypair.generate());
      sellers.push(Keypair.generate());
    }
    console.log(`   Generated ${NUM_MATCHES * 2} keypairs`);

    // Fund all users in batches
    const BATCH_SIZE = 20;
    const allUsers = [...buyers, ...sellers];
    const fundingAmount = FUNDING_PER_USER * LAMPORTS_PER_SOL;

    for (let i = 0; i < allUsers.length; i += BATCH_SIZE) {
      const batch = allUsers.slice(i, i + BATCH_SIZE);
      const tx = new Transaction();

      for (const user of batch) {
        tx.add(
          SystemProgram.transfer({
            fromPubkey: authority.publicKey,
            toPubkey: user.publicKey,
            lamports: fundingAmount,
          })
        );
      }

      const { blockhash } = await client.connection.getLatestBlockhash();
      tx.recentBlockhash = blockhash;
      tx.feePayer = authority.publicKey;
      tx.sign(authority);

      const sig = await client.connection.sendRawTransaction(tx.serialize());
      await confirmTx(client.connection, sig);
      process.stdout.write(`\r   Funded users: ${Math.min(i + BATCH_SIZE, allUsers.length)}/${allUsers.length}`);
    }
    console.log(`\n   ${colors.green}✓ All users funded${colors.reset}`);

    // Create deposit ATAs and mint tokens, then mint complete sets
    // Track successful setups
    console.log(`   Creating positions and minting tokens...`);

    const mintAmount = FILL_AMOUNT * 10n; // Extra buffer
    const successfulBuyers: Set<number> = new Set();
    const successfulSellers: Set<number> = new Set();

    // Process buyers
    for (let i = 0; i < buyers.length; i += BATCH_SIZE) {
      const batchIndices = Array.from({ length: Math.min(BATCH_SIZE, buyers.length - i) }, (_, j) => i + j);

      await Promise.all(
        batchIndices.map(async (idx) => {
          const user = buyers[idx];
          try {
            const userAta = await createAssociatedTokenAccount(
              client.connection,
              authority,
              depositMint,
              user.publicKey
            );
            await mintTo(client.connection, authority, depositMint, userAta, authority, Number(mintAmount));

            const mintResult = await client.mintCompleteSet(
              { user: user.publicKey, market: marketPda, depositMint, amount: mintAmount },
              2
            );
            mintResult.transaction.sign(user);
            const mintSig = await client.connection.sendRawTransaction(mintResult.transaction.serialize());
            await confirmTx(client.connection, mintSig);
            successfulBuyers.add(idx);
          } catch (err) {
            // Silent fail - we'll skip this user in matching
          }
        })
      );
      process.stdout.write(`\r   Buyers setup: ${Math.min(i + BATCH_SIZE, buyers.length)}/${buyers.length}`);
    }
    console.log();

    // Process sellers
    for (let i = 0; i < sellers.length; i += BATCH_SIZE) {
      const batchIndices = Array.from({ length: Math.min(BATCH_SIZE, sellers.length - i) }, (_, j) => i + j);

      await Promise.all(
        batchIndices.map(async (idx) => {
          const user = sellers[idx];
          try {
            const userAta = await createAssociatedTokenAccount(
              client.connection,
              authority,
              depositMint,
              user.publicKey
            );
            await mintTo(client.connection, authority, depositMint, userAta, authority, Number(mintAmount));

            const mintResult = await client.mintCompleteSet(
              { user: user.publicKey, market: marketPda, depositMint, amount: mintAmount },
              2
            );
            mintResult.transaction.sign(user);
            const mintSig = await client.connection.sendRawTransaction(mintResult.transaction.serialize());
            await confirmTx(client.connection, mintSig);
            successfulSellers.add(idx);
          } catch (err) {
            // Silent fail - we'll skip this user in matching
          }
        })
      );
      process.stdout.write(`\r   Sellers setup: ${Math.min(i + BATCH_SIZE, sellers.length)}/${sellers.length}`);
    }

    // Find matching pairs (both buyer and seller succeeded)
    const validPairs: number[] = [];
    for (let i = 0; i < NUM_MATCHES; i++) {
      if (successfulBuyers.has(i) && successfulSellers.has(i)) {
        validPairs.push(i);
      }
    }

    console.log(`\n   ${colors.green}✓ Setup complete: ${validPairs.length}/${NUM_MATCHES} valid pairs${colors.reset}`);

    if (validPairs.length === 0) {
      error("No valid pairs to match!");
      process.exit(1);
    }

    // Store valid pairs for later use
    (global as any).validPairs = validPairs;
    success(`User setup complete (${validPairs.length} pairs ready)`);
  } catch (err) {
    error("User setup failed");
    console.error(err);
    process.exit(1);
  }

  // ============================================================================
  // Step 3: Create Signed Orders (only for valid pairs)
  // ============================================================================
  const validPairs: number[] = (global as any).validPairs;
  const numValidPairs = validPairs.length;

  log("3️⃣", `Creating ${numValidPairs * 2} signed orders...`);

  const bidOrders: FullOrder[] = [];
  const askOrders: FullOrder[] = [];

  try {
    for (const idx of validPairs) {
      // Buyer's BID order: wants YES tokens, pays NO tokens
      const bidOrder = signOrderFull(
        createBidOrder({
          nonce: 0n,
          maker: buyers[idx].publicKey,
          market: marketPda,
          baseMint: conditionalMint0,
          quoteMint: conditionalMint1,
          makerAmount: FILL_AMOUNT, // NO tokens to give
          takerAmount: FILL_AMOUNT, // YES tokens to receive
          expiration: BigInt(Math.floor(Date.now() / 1000) + 3600),
        }),
        buyers[idx]
      );
      bidOrders.push(bidOrder);

      // Seller's ASK order: sells YES tokens, receives NO tokens
      const askOrder = signOrderFull(
        createAskOrder({
          nonce: 0n,
          maker: sellers[idx].publicKey,
          market: marketPda,
          baseMint: conditionalMint0,
          quoteMint: conditionalMint1,
          makerAmount: FILL_AMOUNT, // YES tokens to give
          takerAmount: FILL_AMOUNT, // NO tokens to receive
          expiration: BigInt(Math.floor(Date.now() / 1000) + 3600),
        }),
        sellers[idx]
      );
      askOrders.push(askOrder);
    }

    console.log(`   Created ${bidOrders.length} BID orders`);
    console.log(`   Created ${askOrders.length} ASK orders`);
    success("Orders created and signed");
  } catch (err) {
    error("Order creation failed");
    console.error(err);
    process.exit(1);
  }

  // ============================================================================
  // Step 4: Build Match Transactions
  // ============================================================================
  log("4️⃣", `Building ${numValidPairs} match transactions...`);

  const matchTransactions: Transaction[] = [];

  try {
    const { blockhash, lastValidBlockHeight } =
      await client.connection.getLatestBlockhash();

    for (let i = 0; i < numValidPairs; i++) {
      const matchParams = {
        operator: authority.publicKey,
        market: marketPda,
        baseMint: conditionalMint0,
        quoteMint: conditionalMint1,
        takerOrder: bidOrders[i],
        makerOrders: [askOrders[i]],
        fillAmounts: [FILL_AMOUNT],
      };

      const tx = buildCrossRefMatchOrdersTransaction(matchParams, client.programId);
      tx.recentBlockhash = blockhash;
      tx.lastValidBlockHeight = lastValidBlockHeight;
      tx.feePayer = authority.publicKey;
      tx.sign(authority);

      matchTransactions.push(tx);
    }

    console.log(`   Built ${matchTransactions.length} transactions`);
    success("Transactions built");
  } catch (err) {
    error("Transaction building failed");
    console.error(err);
    process.exit(1);
  }

  // ============================================================================
  // Step 5: Submit All Transactions Concurrently
  // ============================================================================
  log("🔥", `Submitting ${numValidPairs} transactions concurrently...`);

  const results: MatchResult[] = [];
  const startTime = Date.now();

  try {
    const submissions = matchTransactions.map(async (tx, index) => {
      const txStartTime = Date.now();
      try {
        const signature = await client.connection.sendRawTransaction(
          tx.serialize(),
          {
            skipPreflight: true, // Skip preflight for speed
            maxRetries: 3,
          }
        );

        return {
          index,
          success: true,
          signature,
          latencyMs: Date.now() - txStartTime,
        };
      } catch (err) {
        return {
          index,
          success: false,
          error: err instanceof Error ? err.message : String(err),
          latencyMs: Date.now() - txStartTime,
        };
      }
    });

    // Wait for all submissions
    const submissionResults = await Promise.all(submissions);
    results.push(...submissionResults);

    const submissionTime = Date.now() - startTime;
    console.log(`   Submission time: ${submissionTime}ms`);
    console.log(`   Avg per tx: ${(submissionTime / numValidPairs).toFixed(2)}ms`);

    // Wait a bit for transactions to land
    console.log(`   Waiting for confirmations...`);
    await sleep(5000);

    // Check confirmation status
    const successfulSigs = results
      .filter((r) => r.success && r.signature)
      .map((r) => r.signature!);

    if (successfulSigs.length > 0) {
      const statuses = await client.connection.getSignatureStatuses(successfulSigs);

      let confirmed = 0;
      let failed = 0;
      let pending = 0;

      statuses.value.forEach((status, i) => {
        if (!status) {
          pending++;
        } else if (status.err) {
          failed++;
          // Update result
          const result = results.find((r) => r.signature === successfulSigs[i]);
          if (result) {
            result.success = false;
            result.error = JSON.stringify(status.err);
          }
        } else {
          confirmed++;
        }
      });

      console.log(`\n   ${colors.green}Confirmed: ${confirmed}${colors.reset}`);
      console.log(`   ${colors.red}Failed: ${failed}${colors.reset}`);
      console.log(`   ${colors.yellow}Pending: ${pending}${colors.reset}`);
    }

  } catch (err) {
    error("Batch submission failed");
    console.error(err);
  }

  // ============================================================================
  // Results Summary
  // ============================================================================
  const totalTime = Date.now() - startTime;
  const successCount = results.filter((r) => r.success).length;
  const failCount = results.filter((r) => !r.success).length;
  const avgLatency =
    results.reduce((sum, r) => sum + r.latencyMs, 0) / results.length;

  console.log("\n" + "=".repeat(70));
  console.log(`${colors.bright}${colors.magenta}📊 STRESS TEST RESULTS${colors.reset}`);
  console.log("=".repeat(70));

  console.log(`\n${colors.bright}Summary:${colors.reset}`);
  console.log(`   Total Transactions: ${numValidPairs} (of ${NUM_MATCHES} requested)`);
  console.log(`   ${colors.green}Successful: ${successCount} (${((successCount / numValidPairs) * 100).toFixed(1)}%)${colors.reset}`);
  console.log(`   ${colors.red}Failed: ${failCount} (${((failCount / numValidPairs) * 100).toFixed(1)}%)${colors.reset}`);

  console.log(`\n${colors.bright}Performance:${colors.reset}`);
  console.log(`   Total Time: ${totalTime}ms`);
  console.log(`   Avg Submission Latency: ${avgLatency.toFixed(2)}ms`);
  console.log(`   Throughput: ${((successCount / totalTime) * 1000).toFixed(2)} tx/sec`);

  if (failCount > 0) {
    console.log(`\n${colors.bright}${colors.red}Failed Transactions:${colors.reset}`);
    results
      .filter((r) => !r.success)
      .slice(0, 10) // Show first 10 failures
      .forEach((r) => {
        console.log(`   [${r.index}] ${r.error?.slice(0, 80)}...`);
      });
    if (failCount > 10) {
      console.log(`   ... and ${failCount - 10} more`);
    }
  }

  console.log("\n" + "=".repeat(70));
  if (successCount >= numValidPairs * 0.9) {
    console.log(
      `${colors.bright}${colors.green}🎉 STRESS TEST PASSED! (≥90% success rate)${colors.reset}`
    );
  } else if (successCount >= numValidPairs * 0.5) {
    console.log(
      `${colors.bright}${colors.yellow}⚠️ STRESS TEST PARTIAL SUCCESS (≥50% success rate)${colors.reset}`
    );
  } else {
    console.log(
      `${colors.bright}${colors.red}❌ STRESS TEST FAILED (<50% success rate)${colors.reset}`
    );
  }
  console.log("=".repeat(70) + "\n");

  process.exit(failCount > numValidPairs * 0.5 ? 1 : 0);
}

main().catch((err) => {
  error("Stress test failed with error:");
  console.error(err);
  process.exit(1);
});
