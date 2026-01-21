import { describe, it, expect } from "vitest";
import { Resolution } from "./types";

describe("shared types", () => {
  describe("Resolution", () => {
    it("has correct string values", () => {
      expect(Resolution.OneMinute).toBe("1m");
      expect(Resolution.FiveMinutes).toBe("5m");
      expect(Resolution.FifteenMinutes).toBe("15m");
      expect(Resolution.OneHour).toBe("1h");
      expect(Resolution.FourHours).toBe("4h");
      expect(Resolution.OneDay).toBe("1d");
    });

    it("can be used in comparisons", () => {
      const resolution: Resolution = Resolution.OneMinute;
      expect(resolution === "1m").toBe(true);
    });
  });
});
