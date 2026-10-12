import { SdkError } from "../error";
import { RetryPolicy, type LightconeHttp } from "../http";
import type {
  ExportWalletRequest,
  ExportWalletResponse,
  SignAndSendTxResponse,
} from "./index";

interface ClientContext {
  http: LightconeHttp;
}

export class Privy {
  constructor(private readonly client: ClientContext) {}

  /** Privy sign-and-send cannot expose the signed bytes required by v1 validation. */
  async signAndSendTx(_walletId: string, _base64Tx: string): Promise<SignAndSendTxResponse> {
    throw SdkError.validation("Privy sign-and-send cannot verify v1 signed bytes; configure an ExternalSigner that returns signed transaction bytes");
  }

  async exportWallet(walletId: string, decodePubkeyBase64: string): Promise<ExportWalletResponse> {
    const url = `${this.client.http.baseUrl()}/api/privy/wallet/export`;
    const body: ExportWalletRequest = {
      wallet_id: walletId,
      decode_pubkey_base64: decodePubkeyBase64,
    };
    return this.client.http.post<ExportWalletResponse, ExportWalletRequest>(
      url,
      body,
      RetryPolicy.None
    );
  }
}
