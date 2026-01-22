import { describe, it, expect } from "vitest";
import { WebSocketError } from "./error";

describe("WebSocketError", () => {
  describe("static constructors", () => {
    it("creates connectionFailed error", () => {
      const err = WebSocketError.connectionFailed("timeout");
      expect(err.variant).toBe("ConnectionFailed");
      expect(err.message).toContain("timeout");
    });

    it("creates connectionClosed error", () => {
      const err = WebSocketError.connectionClosed(1000, "normal");
      expect(err.variant).toBe("ConnectionClosed");
      expect(err.code).toBe("1000");
    });

    it("creates rateLimited error", () => {
      const err = WebSocketError.rateLimited();
      expect(err.variant).toBe("RateLimited");
    });

    it("creates messageParseError", () => {
      const err = WebSocketError.messageParseError("invalid json");
      expect(err.variant).toBe("MessageParseError");
      expect(err.message).toContain("invalid json");
    });

    it("creates sequenceGap error", () => {
      const err = WebSocketError.sequenceGap(5, 10);
      expect(err.variant).toBe("SequenceGap");
      expect(err.details).toEqual({ expected: 5, received: 10 });
    });

    it("creates resyncRequired error", () => {
      const err = WebSocketError.resyncRequired("ob1");
      expect(err.variant).toBe("ResyncRequired");
      expect(err.details).toEqual({ orderbookId: "ob1" });
    });

    it("creates pingTimeout error", () => {
      const err = WebSocketError.pingTimeout();
      expect(err.variant).toBe("PingTimeout");
    });

    it("creates serverError", () => {
      const err = WebSocketError.serverError("ENGINE_UNAVAILABLE", "engine down");
      expect(err.variant).toBe("ServerError");
      expect(err.code).toBe("ENGINE_UNAVAILABLE");
    });

    it("creates authenticationFailed error", () => {
      const err = WebSocketError.authenticationFailed("invalid token");
      expect(err.variant).toBe("AuthenticationFailed");
    });

    it("creates channelClosed error", () => {
      const err = WebSocketError.channelClosed();
      expect(err.variant).toBe("ChannelClosed");
    });

    it("creates notConnected error", () => {
      const err = WebSocketError.notConnected();
      expect(err.variant).toBe("NotConnected");
    });
  });

  describe("error inheritance", () => {
    it("is an instance of Error", () => {
      const err = WebSocketError.connectionFailed("test");
      expect(err instanceof Error).toBe(true);
      expect(err instanceof WebSocketError).toBe(true);
    });

    it("has correct name", () => {
      const err = WebSocketError.connectionFailed("test");
      expect(err.name).toBe("WebSocketError");
    });
  });
});
