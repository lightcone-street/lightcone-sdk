/* TypeScript file generated from ServerApi.res by genType. */

/* eslint-disable */
/* tslint:disable */

import * as ServerApiJS from './ServerApi.res.mjs';

import type {Auth__Model_Session_t as LightconeSdk_Auth__Model_Session_t} from './LightconeSdk.gen.ts';

import type {Client_t as LightconeSdk_Client_t} from './LightconeSdk.gen.ts';

import type {Messages_t as LightconeSdk_Messages_t} from './LightconeSdk.gen.ts';

import type {SdkError_t as LightconeSdk_SdkError_t} from './LightconeSdk.gen.ts';

import type {TypeScriptApi_WsClient_wsConnection as LightconeSdk_TypeScriptApi_WsClient_wsConnection} from './LightconeSdk.gen.ts';

export const useNativeSigner: (_1:LightconeSdk_Client_t, _2:Uint8Array) => Promise<void> = ServerApiJS.useNativeSigner as any;

export const login: (client:LightconeSdk_Client_t, useEmbeddedWallet:(undefined | boolean)) => Promise<LightconeSdk_Auth__Model_Session_t> = ServerApiJS.login as any;

export const authToken: (_1:LightconeSdk_Client_t) => (undefined | string) = ServerApiJS.authToken as any;

export const setAuthToken: (client:LightconeSdk_Client_t, token:string) => void = ServerApiJS.setAuthToken as any;

export const setServerApiKey: (client:LightconeSdk_Client_t, key:string) => Promise<void> = ServerApiJS.setServerApiKey as any;

export const connect: (client:LightconeSdk_Client_t, onMessage:((_1:LightconeSdk_Messages_t) => void), onConnected:(undefined | ((() => void))), onError:(undefined | (((_1:LightconeSdk_SdkError_t) => void)))) => LightconeSdk_TypeScriptApi_WsClient_wsConnection = ServerApiJS.connect as any;
