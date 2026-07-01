/* TypeScript file generated from Client.resi by genType. */

/* eslint-disable */
/* tslint:disable */

import type {DepositSource_t as Shared_DepositSource_t} from './Shared.gen.ts';

import type {address as SolanaKit_address} from '../bindings/solana-kit/SolanaKit.gen.ts';

import type {cryptoKeyPair as SolanaKit_cryptoKeyPair} from '../bindings/solana-kit/SolanaKit.gen.ts';

import type {keyPairSigner as SolanaKit_keyPairSigner} from '../bindings/solana-kit/SolanaKit.gen.ts';

import type {state as RpcFailover_state} from './RpcFailover.gen.ts';

import type {t as Env_t} from './Env.gen.ts';

import type {t as Http_t} from './Http.gen.ts';

import type {t as SolanaKitRpc_t} from '../bindings/solana-kit/SolanaKitRpc.gen.ts';

export type signingStrategy = 
    { TAG: "NativeSigner"; readonly keypair: SolanaKit_cryptoKeyPair; readonly signer: SolanaKit_keyPairSigner; readonly address: SolanaKit_address };

export type t = {
  readonly http: Http_t; 
  readonly env: Env_t; 
  readonly programId: SolanaKit_address; 
  readonly wsUrl: string; 
  readonly rpcUrl: string; 
  readonly backupRpcUrl: (undefined | string); 
  readonly rpc: SolanaKitRpc_t; 
  readonly backupRpc: SolanaKitRpc_t; 
  readonly rpcFailover: RpcFailover_state; 
  depositSource: Shared_DepositSource_t; 
  orderNonce: (undefined | bigint); 
  signingStrategy: (undefined | signingStrategy)
};
