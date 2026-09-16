import assert from "node:assert/strict";
import { once } from "node:events";
import type { AddressInfo } from "node:net";
import { it } from "node:test";
import { WebSocketServer } from "ws";
import { WsClient, type MessageIn, type WsEvent } from "../src/ws";
import type { PubkeyStr } from "../src/shared";

it("anonymous auth frames preserve explicit transport and public subscription traffic", { timeout: 10_000 }, async () => {
  const server = new WebSocketServer({ host: "127.0.0.1", port: 0 });
  await once(server, "listening");
  const address = server.address() as AddressInfo;
  let connections = 0;
  let publicSubscriptions = 0;
  const messages: MessageIn[] = [];
  const transportEvents: WsEvent[] = [];
  server.on("connection", (socket, request) => {
    connections += 1;
    assert.equal(request.headers.cookie, "lightcone-token=explicit-token");
    assert.equal(request.headers.origin, undefined);
    socket.on("message", (raw) => {
      const request = JSON.parse(raw.toString());
      if (request.method !== "subscribe") return;
      assert.equal(request.params.type, "market");
      publicSubscriptions += 1;
      for (const frame of [
        { type: "auth", data: { status: "authenticated", wallet: "11111111111111111111111111111111" } },
        { type: "auth", data: { status: "anonymous" } },
        { type: "auth", data: { status: "anonymous", reason: "TOKEN_EXPIRED" } },
        { type: "auth", data: { status: "anonymous", reason: "TOKEN_REVOKED" } },
        { type: "error", data: { error: "retry snapshot", code: "PRIVATE_SNAPSHOT_UNAVAILABLE", wallet_address: "wallet-a" } },
        { type: "pong", data: {} },
      ]) socket.send(JSON.stringify({ version: 0.1, ...frame }));
    });
  });
  const client = new WsClient({ url: `ws://127.0.0.1:${address.port}`, reconnect: true, pingIntervalMs: 60_000 }, async () => "explicit-token");
  const completed = new Promise<void>((resolve) => {
    client.on((event) => {
      if (event.type === "Message") {
        messages.push(event.message);
        if (event.message.type === "pong") resolve();
      } else transportEvents.push(event);
    });
  });
  try {
    await client.connect();
    client.subscribe({ type: "market", market_pubkey: "11111111111111111111111111111111" as PubkeyStr });
    await completed;
    await new Promise((resolve) => setTimeout(resolve, 750));
    const reasons = messages.flatMap((message) => message.type === "auth" && message.data.status === "anonymous" ? [message.data.reason] : []);
    assert.deepEqual(reasons, [undefined, "TOKEN_EXPIRED", "TOKEN_REVOKED"]);
    const error = messages.find((message) => message.type === "error");
    assert.equal(error?.type === "error" && error.data.code, "PRIVATE_SNAPSHOT_UNAVAILABLE");
    assert.equal(error?.type === "error" && error.data.wallet_address, "wallet-a");
    assert.equal(client.isConnected(), true);
    assert.equal(connections, 1);
    assert.equal(publicSubscriptions, 1);
    assert.deepEqual(transportEvents.map((event) => event.type), ["Connected"]);
  } finally {
    await client.disconnect();
    await new Promise<void>((resolve, reject) => server.close((error) => error ? reject(error) : resolve()));
  }
});

it("a replacement Node client recovers after handshake retry exhaustion", { timeout: 10_000 }, async () => {
  let available = false;
  let attempts = 0;
  const server = new WebSocketServer({ host: "127.0.0.1", port: 0, verifyClient: (_info, done) => {
    attempts += 1;
    done(available, 503, "Unavailable");
  } });
  await once(server, "listening");
  const config = { url: `ws://127.0.0.1:${(server.address() as AddressInfo).port}`, reconnect: true, maxReconnectAttempts: 0, pingIntervalMs: 60_000 };
  const token = async () => "retained-token";
  const exhaustedClient = new WsClient(config, token);
  let removeListener = () => {};
  const exhausted = new Promise<void>((resolve) => {
    removeListener = exhaustedClient.on((event) => { if (event.type === "MaxReconnectReached") resolve(); });
  });
  const subscribed = new Promise<void>((resolve) => server.on("connection", (socket, request) => {
    assert.equal(request.headers.cookie, "lightcone-token=retained-token");
    socket.on("message", (raw) => {
      const message = JSON.parse(raw.toString());
      if (message.method === "subscribe") {
        assert.equal(message.params.type, "market");
        resolve();
      }
    });
  }));
  let replacement: WsClient | undefined;
  try {
    await assert.rejects(exhaustedClient.connect());
    await exhausted;
    assert.equal(attempts, 1);
    removeListener();
    available = true;
    replacement = new WsClient(config, token);
    await replacement.connect();
    replacement.subscribe({ type: "market", market_pubkey: "11111111111111111111111111111111" as PubkeyStr });
    await subscribed;
    assert.equal(replacement.isConnected(), true);
    assert.equal(attempts, 2);
  } finally {
    removeListener();
    if (replacement?.isConnected()) await replacement.disconnect();
    for (const socket of server.clients) socket.terminate();
    await new Promise<void>((resolve, reject) => server.close((error) => error ? reject(error) : resolve()));
  }
});
