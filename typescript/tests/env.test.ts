import assert from "node:assert/strict";
import { it } from "node:test";

import { apiUrl, LightconeEnv, wsUrl } from "../src/env";

for (const [environment, expectedApiUrl, expectedWsUrl] of [
  [
    LightconeEnv.Local,
    "https://api.local.internalcone.com",
    "wss://ws.local.internalcone.com/ws",
  ],
  [
    LightconeEnv.Staging,
    "https://api.staging.internalcone.com",
    "wss://ws.staging.internalcone.com/ws",
  ],
  [
    LightconeEnv.Prod,
    "https://api.lightcone.xyz",
    "wss://ws.lightcone.xyz/ws",
  ],
] as const) {
  it(`${environment} defaults target the expected backend`, () => {
    const overrides = ["SDK_API_URL", "SDK_WS_URL"].map(
      (name) => [name, process.env[name]] as const
    );
    try {
      for (const [name] of overrides) delete process.env[name];

      assert.equal(apiUrl(environment), expectedApiUrl);
      assert.equal(wsUrl(environment), expectedWsUrl);
    } finally {
      for (const [name, value] of overrides) {
        if (value === undefined) delete process.env[name];
        else process.env[name] = value;
      }
    }
  });
}
