import type { PubkeyStr } from "../shared";
import { shorten } from "../shared/fmt/str";

export * from "./client";
export * from "./native";

/**
 * How a session authenticated, as reported by the backend (derived from which
 * token verified the request).
 */
export enum AuthMethod {
  Privy = "privy",
  Lightcone = "lightcone",
}

export enum ChainType {
  Solana = "solana",
  Ethereum = "ethereum",
}

/** A Privy-managed embedded wallet. */
export interface PrivyEmbeddedWallet {
  privy_id: string;
  chain: ChainType;
  address: string;
}

/** Privy account data attached to an identity. */
export interface UserPrivyData {
  /** The Privy DID (`did:privy:...`). */
  id: string;
  /**
   * Always present: Privy registration provisions the embedded wallet in the
   * same transaction that creates the user.
   */
  wallet: PrivyEmbeddedWallet;
}

/**
 * X account data — the same shape whether X is the login identity or a
 * connected account on a Google/wallet identity.
 */
export interface XAccountData {
  /** X numeric user id (Privy `subject`); absent on legacy rows. */
  user_id?: string;
  username: string;
  display_name?: string;
  avatar_url?: string;
}

/** Google account data for a Google login identity. */
export interface GoogleAccountData {
  email: string;
  name?: string;
  given_name?: string;
  family_name?: string;
  avatar_url?: string;
}

/**
 * The login identity — how the user authenticates. `type` narrows the variant.
 *
 * Privy data lives on the variant because its presence is determined by the
 * identity type: Google/X login only exists via Privy OAuth (guaranteed DID +
 * embedded wallet), while wallet users opt into Privy (SIWS) or stay
 * self-custody.
 */
export type UserIdentity =
  | { type: "google"; account: GoogleAccountData; privy: UserPrivyData }
  | { type: "x"; account: XAccountData; privy: UserPrivyData }
  | { type: "wallet"; address: string; chain: ChainType; privy?: UserPrivyData };

/** Full user profile — the `user` object of {@link SessionResponse}. */
export interface User {
  user_id: string;
  identity: UserIdentity;
  /** X account connected by a non-X-identity user; absent when identity is X. */
  connected_x?: XAccountData;
}

/**
 * Session envelope returned by `loginWithMessage`, `register-privy`, and
 * `GET /api/auth/me`. There is no `wallet_address` field — derive the
 * session's trading wallet with {@link tradingWallet}.
 */
export interface SessionResponse {
  user: User;
  expires_at: number;
  auth_method: AuthMethod;
  is_beta: boolean;
}

/** Human-readable login-method label ("Google" / "X" / "Solana"). */
export function identityText(identity: UserIdentity): "Google" | "X" | "Solana" {
  switch (identity.type) {
    case "google":
      return "Google";
    case "x":
      return "X";
    case "wallet":
      return "Solana";
    default: {
      const exhaustive: never = identity;
      throw new Error(`Unknown identity: ${JSON.stringify(exhaustive)}`);
    }
  }
}

/** Privy account data, regardless of identity type. */
export function userPrivy(user: User): UserPrivyData | undefined {
  switch (user.identity.type) {
    case "google":
    case "x":
      return user.identity.privy;
    case "wallet":
      return user.identity.privy;
  }
}

/** The X account, whether it is the login identity or a connected account. */
export function userXAccount(user: User): XAccountData | undefined {
  return user.identity.type === "x" ? user.identity.account : user.connected_x;
}

/**
 * The wallet a session operates as.
 *
 * Google/X identities only exist via Privy registration, which always
 * provisions an embedded wallet — that wallet is the answer regardless of
 * auth method. Wallet identities depend on the session: a Privy (SIWS)
 * session trades via the embedded wallet, a Lightcone session trades via the
 * wallet that signed in.
 */
export function tradingWallet(user: User, authMethod: AuthMethod): string {
  switch (user.identity.type) {
    case "google":
    case "x":
      return user.identity.privy.wallet.address;
    case "wallet":
      return authMethod === AuthMethod.Privy
        ? (user.identity.privy?.wallet.address ?? user.identity.address)
        : user.identity.address;
  }
}

/** Short display label for the wallet a session operates as. */
export function walletDisplayName(user: User, authMethod: AuthMethod): string {
  return shorten(tradingWallet(user, authMethod), 8);
}

/**
 * Best display name for the user. Google: `name`, falling back to the email;
 * X: `display_name`, falling back to the username; wallet identities show the
 * shortened address (`FRGk...WcPR`).
 */
export function displayName(user: User): string {
  switch (user.identity.type) {
    case "google":
      return user.identity.account.name ?? user.identity.account.email;
    case "x":
      return user.identity.account.display_name ?? user.identity.account.username;
    case "wallet":
      return shorten(user.identity.address, 8);
  }
}

/** Avatar URL from the login identity's OAuth provider, if any. */
export function avatarUrl(user: User): string | undefined {
  switch (user.identity.type) {
    case "google":
    case "x":
      return user.identity.account.avatar_url;
    case "wallet":
      return undefined;
  }
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

export interface NonceResponse {
  nonce: string;
}
