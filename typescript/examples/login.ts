import { signLoginMessage } from "../src/auth/native";
import { restClient, wallet } from "./common";

async function main() {
  const client = restClient();
  const keypair = wallet();

  // 1. Request a nonce
  const nonce = await client.auth().getNonce();
  console.log("nonce:", nonce);

  // 2. Sign the nonce
  const signed = signLoginMessage(keypair, nonce);
  console.log("message:", signed.message);

  // 3. Login
  const user = await client.auth().loginWithMessage(
    signed.message,
    signed.signature_bs58,
    signed.pubkey_bytes
  );
  console.log("logged in:", user.wallet_address);

  // 4. Check auth state
  console.log("authenticated:", client.auth().isAuthenticated());

  // 5. Verify session
  const me = await client.auth().checkSession();
  console.log("session valid for:", me.wallet_address);

  // 6. Logout
  await client.auth().logout();
  console.log("logged out, authenticated:", client.auth().isAuthenticated());
}

main().catch(console.error);
