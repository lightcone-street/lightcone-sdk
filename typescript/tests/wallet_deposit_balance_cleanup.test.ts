import assert from "node:assert/strict";
import { describe, it } from "node:test";
import type { PubkeyStr } from "../src/shared";
import { WsClient as NodeWsClient } from "../src/ws/client.node";
import { WsClient as BrowserWsClient } from "../src/ws/client.browser";
import type { SubscribeParams } from "../src/ws/subscriptions";
import type { MessageOut } from "../src/ws";

const wallet = "WalletA" as PubkeyStr;

function activeSubscriptions(client: object): SubscribeParams[] {
  return (client as { activeSubscriptions: SubscribeParams[] }).activeSubscriptions;
}

function pendingMessages(client: object): MessageOut[] {
  return (client as { pendingMessages: MessageOut[] }).pendingMessages;
}

describe("authenticated subscription cleanup", () => {
  for (const [name, Client] of [
    ["Node", NodeWsClient],
    ["browser", BrowserWsClient],
  ] as const) {
    it(`clears user and wallet balances for ${name}`, () => {
      const client = new Client({ reconnect: false });
      client.subscribe({ type: "user", wallet_address: wallet });
      client.subscribe({ type: "wallet_deposit_balances", wallet_address: wallet });
      client.subscribe({ type: "market", market_pubkey: wallet });

      client.clearAuthedSubscriptions();
      assert.deepEqual(activeSubscriptions(client), [
        { type: "market", market_pubkey: wallet },
      ]);
      assert.deepEqual(pendingMessages(client), [
        {
          method: "subscribe",
          params: { type: "market", market_pubkey: wallet },
        },
      ]);
    });
  }
});
