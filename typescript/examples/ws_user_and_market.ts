import {
  asPubkeyStr,
  WalletDepositBalancesState,
  type WsEvent,
} from "../src";
import { tradingWallet } from "../src/auth";
import {
  getKeypair,
  login,
  market,
  restClient,
  runExample,
  withTimeout,
} from "./common";

async function main() {
  const client = restClient();
  const keypair = getKeypair();
  const session = await login(client, keypair);
  const wallet = asPubkeyStr(tradingWallet(session.user, session.auth_method));
  const m = await market(client);
  const ws = client.ws();
  const state = new WalletDepositBalancesState();
  let streamError: Error | undefined;

  let resolveDone!: () => void;
  const done = new Promise<void>((resolve) => {
    resolveDone = resolve;
  });

  const unsubscribe = ws.on((event: WsEvent) => {
    if (
      event.type === "Message" &&
      event.message.type === "wallet_deposit_balances"
    ) {
      const update = event.message.data;
      if (
        state.applyEvent(update).kind === "applied" &&
        update.event_type === "wallet_deposit_balance_snapshot"
      ) {
        resolveDone();
      }
    } else if (event.type === "Message" && event.message.type === "error") {
      streamError = new Error(event.message.data.error);
      resolveDone();
    } else if (event.type === "Error") {
      streamError = new Error(event.error);
      resolveDone();
    } else if (event.type === "MaxReconnectReached") {
      streamError = new Error("WebSocket reconnect attempts exhausted");
      resolveDone();
    }
  });

  try {
    await ws.connect();
    ws.subscribe({
      type: "user",
      wallet_address: wallet,
    });
    ws.subscribe({
      type: "market",
      market_pubkey: m.pubkey,
    });
    ws.subscribe({ type: "wallet_deposit_balances", wallet_address: wallet });
    await withTimeout(
      done,
      30_000,
      "timed out waiting for a complete wallet balance snapshot"
    );
    if (streamError) throw streamError;
    if (state.contextSlot === undefined) {
      throw new Error("complete snapshot did not establish a slot");
    }
    console.log(
      `wallet=${wallet} slot=${state.contextSlot} count=${state.balances.size}`
    );
  } finally {
    ws.unsubscribe({ type: "user", wallet_address: wallet });
    ws.unsubscribe({ type: "market", market_pubkey: m.pubkey });
    ws.unsubscribe({ type: "wallet_deposit_balances", wallet_address: wallet });
    unsubscribe();
    await ws.disconnect();
  }
}

void runExample(main);
