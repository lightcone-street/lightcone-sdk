export * from "./client";
export * from "./wire";

export interface AdminEnvelope<T> {
  payload: T;
  signature: string;
}
