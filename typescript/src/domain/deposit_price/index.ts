import type { Resolution } from "../../shared";

export * from "./client";
export * from "./state";
export * from "./wire";

export interface DepositPriceKey {
  depositAsset: string;
  resolution: Resolution;
}
