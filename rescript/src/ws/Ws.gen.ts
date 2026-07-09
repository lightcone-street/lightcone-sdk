/* TypeScript file generated from Ws.resi by genType. */

/* eslint-disable */
/* tslint:disable */

import * as WsJS from './Ws.res.mjs';

export abstract class t { protected opaque!: any }; /* simulate opaque types */

export type ReadyState_t = "Connecting" | "Open" | "Closing" | "Closed";

export const readyState: (_1:t) => ReadyState_t = WsJS.readyState as any;

export const clearAuthedSubscriptions: (_1:t) => void = WsJS.clearAuthedSubscriptions as any;
