export * from "./client";

export interface SignAndSendTxRequest {
  wallet_id: string;
  base64_tx: string;
}

export interface SignAndSendTxResponse {
  hash: string;
}

export interface ExportWalletRequest {
  wallet_id: string;
  decode_pubkey_base64: string;
}

export interface ExportWalletResponse {
  encryption_type: string;
  ciphertext: string;
  encapsulated_key: string;
}
