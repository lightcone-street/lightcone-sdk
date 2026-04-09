export * from "./client";
export * from "./wire";

export interface AdminNonceResponse {
  nonce: string;
  message: string;
}

export interface AdminLoginRequest {
  message: string;
  signature_bs58: string;
  pubkey_bytes: number[];
}

export interface AdminLoginResponse {
  wallet_address: string;
  expires_at: number;
}
