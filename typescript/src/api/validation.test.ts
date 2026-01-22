import { describe, it, expect } from "vitest";
import {
  validatePubkey,
  validateSignature,
  validateLimit,
  MAX_PAGINATION_LIMIT,
  DEFAULT_TIMEOUT_MS,
} from "./validation";
import { ApiError } from "./error";

// Valid Solana pubkeys for testing
const VALID_PUBKEY = "11111111111111111111111111111111"; // System Program
const VALID_PUBKEY_2 = "TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA"; // Token Program

describe("validation", () => {
  describe("validatePubkey", () => {
    it("accepts valid pubkey", () => {
      expect(() => validatePubkey(VALID_PUBKEY, "test")).not.toThrow();
      expect(() => validatePubkey(VALID_PUBKEY_2, "test")).not.toThrow();
    });

    it("rejects empty string", () => {
      expect(() => validatePubkey("", "test")).toThrow(ApiError);
      expect(() => validatePubkey("", "test")).toThrow("cannot be empty");
    });

    it("rejects invalid base58", () => {
      expect(() => validatePubkey("invalid!@#$", "test")).toThrow(ApiError);
      expect(() => validatePubkey("0OIl", "test")).toThrow(ApiError); // Invalid base58 chars
    });

    it("rejects wrong length", () => {
      expect(() => validatePubkey("abc", "test")).toThrow(ApiError);
      expect(() => validatePubkey("tooshort", "test")).toThrow(ApiError);
    });

    it("includes field name in error", () => {
      try {
        validatePubkey("invalid", "myField");
      } catch (e) {
        expect((e as ApiError).message).toContain("myField");
      }
    });
  });

  describe("validateSignature", () => {
    it("accepts valid 128-char hex signature", () => {
      const validSig = "a".repeat(128);
      expect(() => validateSignature(validSig)).not.toThrow();
    });

    it("accepts mixed case hex", () => {
      const validSig = "aAbBcCdDeEfF0123456789".padEnd(128, "0");
      expect(() => validateSignature(validSig)).not.toThrow();
    });

    it("rejects too short signature", () => {
      expect(() => validateSignature("abc")).toThrow(ApiError);
      expect(() => validateSignature("abc")).toThrow("128 hex characters");
    });

    it("rejects too long signature", () => {
      const tooLong = "a".repeat(130);
      expect(() => validateSignature(tooLong)).toThrow(ApiError);
    });

    it("rejects non-hex characters", () => {
      const invalidSig = "g".repeat(128); // 'g' is not hex
      expect(() => validateSignature(invalidSig)).toThrow(ApiError);
      expect(() => validateSignature(invalidSig)).toThrow("only hex characters");
    });
  });

  describe("validateLimit", () => {
    it("accepts undefined limit", () => {
      expect(() => validateLimit(undefined)).not.toThrow();
    });

    it("accepts valid limits", () => {
      expect(() => validateLimit(1)).not.toThrow();
      expect(() => validateLimit(100)).not.toThrow();
      expect(() => validateLimit(MAX_PAGINATION_LIMIT)).not.toThrow();
    });

    it("rejects limit below 1", () => {
      expect(() => validateLimit(0)).toThrow(ApiError);
      expect(() => validateLimit(-1)).toThrow(ApiError);
    });

    it("rejects limit above MAX_PAGINATION_LIMIT", () => {
      expect(() => validateLimit(MAX_PAGINATION_LIMIT + 1)).toThrow(ApiError);
      expect(() => validateLimit(1000)).toThrow(ApiError);
    });
  });

  describe("constants", () => {
    it("exports MAX_PAGINATION_LIMIT as 500", () => {
      expect(MAX_PAGINATION_LIMIT).toBe(500);
    });

    it("exports DEFAULT_TIMEOUT_MS as 30000", () => {
      expect(DEFAULT_TIMEOUT_MS).toBe(30000);
    });
  });
});
