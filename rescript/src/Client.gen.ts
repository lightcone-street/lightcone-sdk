/* TypeScript file generated from Client.resi by genType. */

/* eslint-disable */
/* tslint:disable */

import type {Auth__Model_Credentials_t as LightconeSdk_Auth__Model_Credentials_t} from './LightconeSdk.gen.ts';

import type {Env_t as LightconeSdk_Env_t} from './LightconeSdk.gen.ts';

import type {Http_t as LightconeSdk_Http_t} from './LightconeSdk.gen.ts';

import type {RpcFailover_t as LightconeSdk_RpcFailover_t} from './LightconeSdk.gen.ts';

import type {Scaling_OrderbookRules_t as LightconeSdk_Scaling_OrderbookRules_t} from './LightconeSdk.gen.ts';

import type {SdkError_t as LightconeSdk_SdkError_t} from './LightconeSdk.gen.ts';

import type {Shared_DepositSource_t as LightconeSdk_Shared_DepositSource_t} from './LightconeSdk.gen.ts';

import type {SolanaKitRpc_t as LightconeSdk_SolanaKitRpc_t} from './LightconeSdk.gen.ts';

import type {SolanaKit_address as LightconeSdk_SolanaKit_address} from './LightconeSdk.gen.ts';

import type {SolanaKit_cryptoKeyPair as LightconeSdk_SolanaKit_cryptoKeyPair} from './LightconeSdk.gen.ts';

import type {SolanaKit_keyPairSigner as LightconeSdk_SolanaKit_keyPairSigner} from './LightconeSdk.gen.ts';

import type {V1Transaction_Resources_t as LightconeSdk_V1Transaction_Resources_t} from './LightconeSdk.gen.ts';

export type ExternalSigner_t = {
  readonly address: LightconeSdk_SolanaKit_address;
  readonly signMessage: (_1:Uint8Array) => Promise<Uint8Array>;
  readonly signTransaction: (_1:Uint8Array) => Promise<Uint8Array>
};

export type SigningStrategy_t =
    { TAG: "NativeSigner"; readonly keypair: LightconeSdk_SolanaKit_cryptoKeyPair; readonly signer: LightconeSdk_SolanaKit_keyPairSigner; readonly address: LightconeSdk_SolanaKit_address }
  | { TAG: "ExternalSigner"; _0: ExternalSigner_t };

export type t = {
  readonly http: LightconeSdk_Http_t;
  readonly env: LightconeSdk_Env_t;
  readonly programId: LightconeSdk_SolanaKit_address;
  readonly wsUrl: string;
  readonly rpcUrl: string;
  readonly backupRpcUrl: (undefined | string);
  readonly rpc: LightconeSdk_SolanaKitRpc_t;
  readonly backupRpc: LightconeSdk_SolanaKitRpc_t;
  readonly rpcFailover: LightconeSdk_RpcFailover_t;
  depositSource: LightconeSdk_Shared_DepositSource_t;
  orderNonce: (undefined | bigint);
  credentials: (undefined | LightconeSdk_Auth__Model_Credentials_t);
  readonly orderbookRules: {[id: string]: Promise<
    {
    TAG: "Ok";
    _0: LightconeSdk_Scaling_OrderbookRules_t
  }
  | {
    TAG: "Error";
    _0: LightconeSdk_SdkError_t
  }>};
  transactionResources: (undefined | LightconeSdk_V1Transaction_Resources_t);
  sponsorshipEnabled: boolean;
  signingStrategy: (undefined | SigningStrategy_t)
};
