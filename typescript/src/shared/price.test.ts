import { describe, it, expect } from "vitest";
import { parseDecimal, formatDecimal } from "./price";

describe("price utilities", () => {
  describe("parseDecimal", () => {
    it("parses valid decimal strings", () => {
      expect(parseDecimal("0.500000")).toBe(0.5);
      expect(parseDecimal("1.000000")).toBe(1.0);
      expect(parseDecimal("0.123456")).toBeCloseTo(0.123456);
      expect(parseDecimal("100")).toBe(100);
      expect(parseDecimal("-0.5")).toBe(-0.5);
    });

    it("throws on invalid decimal strings", () => {
      expect(() => parseDecimal("not a number")).toThrow("Invalid decimal string");
      expect(() => parseDecimal("")).toThrow("Invalid decimal string");
      expect(() => parseDecimal("abc")).toThrow("Invalid decimal string");
    });
  });

  describe("formatDecimal", () => {
    it("formats numbers with specified precision", () => {
      expect(formatDecimal(0.5, 6)).toBe("0.500000");
      expect(formatDecimal(1.0, 6)).toBe("1.000000");
      expect(formatDecimal(0.123456789, 6)).toBe("0.123457"); // Rounded
      expect(formatDecimal(0, 6)).toBe("0.000000");
    });

    it("handles different precisions", () => {
      expect(formatDecimal(0.5, 2)).toBe("0.50");
      expect(formatDecimal(0.5, 0)).toBe("1"); // Rounded
      expect(formatDecimal(1.234, 3)).toBe("1.234");
    });
  });
});
