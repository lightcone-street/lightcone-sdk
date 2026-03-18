import type { PubkeyStr } from "../shared";

export * from "./client";
export * from "./native";

export interface User {
  id: string;
  wallet_address: string;
  linked_account: LinkedAccount;
  privy_id?: string;
  embedded_wallet?: EmbeddedWallet;
  x_username?: string;
  x_user_id?: string;
  x_display_name?: string;
  google_email?: string;
}

export interface LinkedAccount {
  id: string;
  type: LinkedAccountType;
  chain?: ChainType;
  address: string;
}

export enum LinkedAccountType {
  Wallet = "wallet",
  TwitterOauth = "twitter_oauth",
  GoogleOauth = "google_oauth",
}

export enum ChainType {
  Solana = "solana",
  Ethereum = "ethereum",
}

export interface EmbeddedWallet {
  privy_id: string;
  chain: ChainType;
  address: string;
}

export interface AuthCredentials {
  user_id: string;
  wallet_address: PubkeyStr;
  expires_at: Date;
}

export function isAuthenticated(credentials?: AuthCredentials): boolean {
  if (!credentials) {
    return false;
  }
  return Date.now() < credentials.expires_at.getTime();
}

export function generateSigninMessage(nonce: string): Uint8Array {
  return new TextEncoder().encode(`Sign in to Lightcone\nNonce: ${nonce}`);
}

export interface LoginRequest {
  message: string;
  signature_bs58: string;
  pubkey_bytes: number[];
  use_embedded_wallet?: boolean;
}

export interface LoginResponse {
  user_id: string;
  wallet_address: string;
  expires_at: number;
  linked_account: LinkedAccount;
  privy_id?: string;
  embedded_wallet?: EmbeddedWallet;
  x_username?: string;
  x_user_id?: string;
  x_display_name?: string;
  google_email?: string;
}

export interface MeResponse {
  user_id: string;
  wallet_address: string;
  linked_account: LinkedAccount;
  privy_id?: string;
  embedded_wallet?: EmbeddedWallet;
  x_username?: string;
  x_user_id?: string;
  x_display_name?: string;
  google_email?: string;
  expires_at: number;
}

export interface NonceResponse {
  nonce: string;
}
