import { signLoginMessage } from "../src/auth";
import { restClient, wallet } from "./common";

async function main() {
  const client = restClient();
  const keypair = wallet();

  const nonce = await client.auth().getNonce();
  const signed = signLoginMessage(keypair, nonce);
  const user = await client.auth().loginWithMessage(
    signed.message,
    signed.signature_bs58,
    signed.pubkey_bytes
  );
  console.log(`logged in: ${user.id} (${user.wallet_address})`);
  console.log("cached auth state:", client.auth().isAuthenticated());
  const me = await client.auth().checkSession();
  console.log("session wallet:", me.wallet_address);
  await client.auth().logout();
  console.log("logged out");
}

main().catch(console.error);
