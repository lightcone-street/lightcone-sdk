import { tradingWallet, displayName, identityText, signLoginMessage } from "../src/auth";
import { restClient, getKeypair, runExample } from "./common";

async function main() {
  const client = restClient();
  const keypair = getKeypair();

  const nonce = await client.auth().getNonce();
  const signed = signLoginMessage(keypair, nonce);
  const session = await client.auth().loginWithMessage(
    signed.message,
    signed.signature_bs58,
    signed.pubkey_bytes
  );
  const wallet = tradingWallet(session.user, session.auth_method);
  console.log(`logged in: ${session.user.user_id} (${wallet})`);
  console.log("identity:", identityText(session.user.identity));
  console.log("display name:", displayName(session.user));
  console.log("cached auth state:", client.auth().isAuthenticated());
  const me = await client.auth().checkSession();
  console.log("session wallet:", tradingWallet(me.user, me.auth_method));
  await client.auth().logout();
  console.log("logged out");
}

void runExample(main);
