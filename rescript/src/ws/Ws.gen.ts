/* TypeScript file generated from Ws.resi by genType. */

/* eslint-disable */
/* tslint:disable */

import * as WsJS from './Ws.res.mjs';

export abstract class connection { protected opaque!: any }; /* simulate opaque types */

export type readyState = "Connecting" | "Open" | "Closing" | "Closed";

export const readyState: (_1:connection) => readyState = WsJS.readyState as any;

export const clearAuthedSubscriptions: (_1:connection) => void = WsJS.clearAuthedSubscriptions as any;
