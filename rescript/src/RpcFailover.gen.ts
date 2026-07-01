/* TypeScript file generated from RpcFailover.resi by genType. */

/* eslint-disable */
/* tslint:disable */

export type activeRpc = "primary" | "backup";

export type state = { active: activeRpc; flippedToBackupAtMs: (undefined | number) };
