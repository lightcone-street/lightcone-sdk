import { Keypair } from "@solana/web3.js";
import bs58 from "bs58";
import nacl from "tweetnacl";
import { generateSigninMessage } from "./index";

export interface SignedLogin {
  message: string;
  signature_bs58: string;
  pubkey_bytes: Uint8Array;
}

export function signLoginMessage(keypair: Keypair, nonce: string): SignedLogin {
  const messageBytes = generateSigninMessage(nonce);
  const signature = nacl.sign.detached(messageBytes, keypair.secretKey);

  return {
    message: new TextDecoder().decode(messageBytes),
    signature_bs58: bs58.encode(signature),
    pubkey_bytes: keypair.publicKey.toBytes(),
  };
}
