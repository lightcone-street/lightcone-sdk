import type { MessageOut, ReadyState, WsEvent } from "./index";
import type { SubscribeParams, UnsubscribeParams } from "./subscriptions";

/**
 * Shared interface for WebSocket clients.
 *
 * Two implementations exist:
 * - `client.node.ts`    — Node.js, backed by the `ws` npm module
 * - `client.browser.ts` — Browser, backed by native `globalThis.WebSocket`
 *
 * The active implementation is selected at build time via the `"browser"`
 * field in package.json.
 */
export interface IWsClient {
  connect(): Promise<void>;
  disconnect(): Promise<void>;
  send(message: MessageOut): void;
  subscribe(params: SubscribeParams): void;
  unsubscribe(params: UnsubscribeParams): void;
  isConnected(): boolean;
  readyState(): ReadyState;
  restartConnection(): Promise<void>;
  /**
   * Purge User/wallet replay tracking and queued authenticated messages.
   * Public subscriptions remain. This does not unsubscribe an open server
   * stream, so callers must also send unsubscribe or disconnect on logout.
   */
  clearAuthedSubscriptions(): void;
  on(callback: (event: WsEvent) => void): () => void;
}
