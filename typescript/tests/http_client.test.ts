import { once } from "node:events";
import { createServer, type Server } from "node:http";
import type { AddressInfo } from "node:net";
import { describe, it } from "node:test";
import assert from "node:assert/strict";
import { SdkError, isUnauthorized } from "../src/error";
import { LightconeHttp } from "../src/http/client";
import { RetryPolicy, type RetryConfig } from "../src/http/retry";
import { Auth } from "../src/auth/client";

type TestResponse = {
  status: number;
  body: string;
  /** Optional Set-Cookie header, for seeding the client's session token. */
  setCookie?: string;
  /** Optional Location header, for redirect responses. */
  location?: string;
  retryAfter?: string;
  retryAfterMs?: string;
};

async function withServer(
  responses: TestResponse[],
  test: (
    baseUrl: string,
    attempts: () => number,
    cookiesSeen: () => (string | undefined)[],
  ) => Promise<void>,
): Promise<void> {
  let attempts = 0;
  const queue = [...responses];
  const cookiesSeen: (string | undefined)[] = [];

  const server = createServer((request, response) => {
    attempts += 1;
    cookiesSeen.push(request.headers.cookie);
    const next = queue.shift() ?? {
      status: 500,
      body: '{"status":"error","error_details":{"reason":"unexpected extra request"}}',
    };
    const headers: Record<string, string> = { "content-type": "application/json" };
    if (next.retryAfterMs !== undefined) headers["retry-after-ms"] = next.retryAfterMs;
    if (next.retryAfter !== undefined) headers["retry-after"] = next.retryAfter;
    if (next.setCookie) {
      headers["set-cookie"] = next.setCookie;
    }
    if (next.location) {
      headers.location = next.location;
    }
    response.writeHead(next.status, headers);
    response.end(next.body);
  });

  await listen(server);
  const address = server.address() as AddressInfo;
  try {
    await test(`http://127.0.0.1:${address.port}`, () => attempts, () => cookiesSeen);
  } finally {
    await close(server);
  }
}

function fastRetry(statuses: readonly number[]) {
  const config: RetryConfig = {
    maxRetries: 1,
    initialDelayMs: 0,
    maxDelayMs: 0,
    backoffFactor: 1,
    jitter: false,
    retryableStatuses: statuses,
  };
  return RetryPolicy.custom(config);
}

async function listen(server: Server): Promise<void> {
  server.listen(0, "127.0.0.1");
  await once(server, "listening");
}

async function close(server: Server): Promise<void> {
  await new Promise<void>((resolve, reject) => {
    server.close((error) => {
      if (error) reject(error);
      else resolve();
    });
  });
}

describe("LightconeHttp", () => {
  it("treats HTTP 200 error envelopes as rejections", async () => {
    await withServer(
      [
        {
          status: 200,
          body: '{"status":"error","error_details":{"reason":"invalid exact ratio","rejection_code":"PRICE_NOT_EXACTLY_REPRESENTABLE"}}',
        },
      ],
      async (baseUrl) => {
        const client = new LightconeHttp(baseUrl);
        await assert.rejects(
          () => client.get(`${baseUrl}/test`, RetryPolicy.None),
          (error) =>
            error instanceof SdkError &&
            error.variant === "ApiRejected" &&
            error.apiRejectedDetails?.rejectionCode?.wireName() ===
              "PRICE_NOT_EXACTLY_REPRESENTABLE",
        );
      },
    );
  });

  it("returns structured 500 rejection details", async () => {
    await withServer(
      [
        {
          status: 500,
          body: '{"status":"error","error_details":{"reason":"engine failed","error_code":"ENGINE","error_log_id":"LCERR_500"}}',
        },
      ],
      async (baseUrl) => {
        const client = new LightconeHttp(baseUrl);

        await assert.rejects(
          () => client.get(`${baseUrl}/test`, RetryPolicy.Idempotent),
          (error) => {
            assert(error instanceof SdkError);
            assert.equal(error.variant, "ApiRejected");
            assert.equal(error.apiRejectedDetails?.reason, "engine failed");
            assert.equal(error.apiRejectedDetails?.errorCode, "ENGINE");
            assert.equal(error.apiRejectedDetails?.errorLogId, "LCERR_500");
            assert.ok(error.apiRejectedDetails?.requestId);
            return true;
          },
        );
      },
    );
  });

  it("retries a raw 409 status when custom policy includes it", async () => {
    await withServer(
      [
        {
          status: 409,
          body: '{"status":"error","error_details":{"reason":"nonce mismatch","error_code":"NONCE_MISMATCH"}}',
        },
        { status: 200, body: '{"status":"success","body":{"ok":true}}' },
      ],
      async (baseUrl, attempts) => {
        const client = new LightconeHttp(baseUrl);
        const body = await client.get<{ ok: boolean }>(`${baseUrl}/retry`, fastRetry([409]));

        assert.deepEqual(body, { ok: true });
        assert.equal(attempts(), 2);
      },
    );
  });

  it("does not retry a 429 when custom policy excludes it", async () => {
    await withServer(
      [
        {
          status: 429,
          body: '{"status":"error","error_details":{"reason":"rate limited","error_code":"RATE_LIMITED","error_log_id":"LCERR_429"}}',
        },
        { status: 200, body: '{"status":"success","body":{"ok":true}}' },
      ],
      async (baseUrl, attempts) => {
        const client = new LightconeHttp(baseUrl);

        await assert.rejects(
          () => client.get(`${baseUrl}/retry`, fastRetry([503])),
          (error) => {
            assert(error instanceof SdkError);
            assert.equal(error.variant, "ApiRejected");
            assert.equal(error.apiRejectedDetails?.reason, "rate limited");
            assert.equal(error.apiRejectedDetails?.errorCode, "RATE_LIMITED");
            assert.equal(error.apiRejectedDetails?.errorLogId, "LCERR_429");
            return true;
          },
        );
        assert.equal(attempts(), 1);
      },
    );
  });

  it("preserves structured 503 details after retry exhaustion", async () => {
    await withServer(
      [
        {
          status: 503,
          body: '{"status":"error","error_details":{"reason":"temporarily unavailable","error_code":"UNAVAILABLE","error_log_id":"LCERR_503A"}}',
        },
        {
          status: 503,
          body: '{"status":"error","error_details":{"reason":"still unavailable","error_code":"UNAVAILABLE","error_log_id":"LCERR_503B"}}',
        },
      ],
      async (baseUrl, attempts) => {
        const client = new LightconeHttp(baseUrl);

        await assert.rejects(
          () => client.get(`${baseUrl}/retry`, fastRetry([503])),
          (error) => {
            assert(error instanceof SdkError);
            assert.equal(error.variant, "ApiRejected");
            assert.equal(error.apiRejectedDetails?.reason, "still unavailable");
            assert.equal(error.apiRejectedDetails?.errorLogId, "LCERR_503B");
            return true;
          },
        );
        assert.equal(attempts(), 2);
      },
    );
  });

  // ── Credential restorer (401 → restore → replay) ─────────────────────────

  function stubRestorer(restored: boolean): {
    restorer: () => Promise<boolean>;
    calls: () => number;
  } {
    let calls = 0;
    return {
      restorer: async () => {
        calls += 1;
        return restored;
      },
      calls: () => calls,
    };
  }

  it("replays once after a successful credential restore on 401", async () => {
    await withServer(
      [
        { status: 401, body: "Unauthorized" },
        { status: 200, body: '{"status":"success","body":{"ok":true}}' },
      ],
      async (baseUrl, attempts) => {
        const client = new LightconeHttp(baseUrl);
        const stub = stubRestorer(true);
        client.setCredentialRestorer(stub.restorer);

        const body = await client.get<{ ok: boolean }>(
          `${baseUrl}/me`,
          RetryPolicy.Idempotent,
        );

        assert.equal(body.ok, true);
        assert.equal(attempts(), 2);
        assert.equal(stub.calls(), 1);
      },
    );
  });

  it("restores but never replays no-retry POSTs", async () => {
    await withServer(
      [{ status: 401, body: "Unauthorized" }],
      async (baseUrl, attempts) => {
        const client = new LightconeHttp(baseUrl);
        const stub = stubRestorer(true);
        client.setCredentialRestorer(stub.restorer);

        // RetryPolicy.None declares the request non-idempotent (orders etc.):
        // a 401 still triggers restoration — healing the session for the
        // caller's next attempt — but the request itself is NEVER auto-
        // replayed; the original 401 propagates.
        await assert.rejects(
          () =>
            client.post<{ ok: boolean }, { side: string }>(
              `${baseUrl}/order`,
              { side: "buy" },
              RetryPolicy.None,
            ),
          (error) => {
            assert(isUnauthorized(error));
            return true;
          },
        );

        assert.equal(attempts(), 1);
        assert.equal(stub.calls(), 1);
      },
    );
  });

  it("propagates 401 unchanged without a restorer", async () => {
    await withServer(
      [{ status: 401, body: "Unauthorized" }],
      async (baseUrl, attempts) => {
        const client = new LightconeHttp(baseUrl);

        await assert.rejects(
          () => client.get(`${baseUrl}/me`, RetryPolicy.Idempotent),
          (error) => {
            assert(isUnauthorized(error));
            return true;
          },
        );
        assert.equal(attempts(), 1);
      },
    );
  });

  it("propagates 401 without replay when the restore fails", async () => {
    await withServer(
      [{ status: 401, body: "Unauthorized" }],
      async (baseUrl, attempts) => {
        const client = new LightconeHttp(baseUrl);
        const stub = stubRestorer(false);
        client.setCredentialRestorer(stub.restorer);

        await assert.rejects(
          () => client.get(`${baseUrl}/me`, RetryPolicy.Idempotent),
          (error) => {
            assert(isUnauthorized(error));
            return true;
          },
        );
        assert.equal(attempts(), 1);
        assert.equal(stub.calls(), 1);
      },
    );
  });

  it("consults the restorer at most once per request", async () => {
    // Restore "succeeds" but the replay still 401s (e.g. the restored session
    // is rejected too) — the second 401 must propagate rather than loop
    // through the restorer again.
    await withServer(
      [
        { status: 401, body: "Unauthorized" },
        { status: 401, body: "Unauthorized" },
      ],
      async (baseUrl, attempts) => {
        const client = new LightconeHttp(baseUrl);
        const stub = stubRestorer(true);
        client.setCredentialRestorer(stub.restorer);

        await assert.rejects(
          () => client.get(`${baseUrl}/me`, RetryPolicy.Idempotent),
          (error) => {
            assert(isUnauthorized(error));
            return true;
          },
        );
        assert.equal(attempts(), 2);
        assert.equal(stub.calls(), 1);
      },
    );
  });

  it("shares one restoration across concurrent 401s", async () => {
    // Two requests hit expiry together: both must recover, sharing a single
    // restorer run — the second awaits the in-flight restoration instead of
    // failing fast.
    await withServer(
      [
        { status: 401, body: "Unauthorized" },
        { status: 401, body: "Unauthorized" },
        { status: 200, body: '{"status":"success","body":{"ok":true}}' },
        { status: 200, body: '{"status":"success","body":{"ok":true}}' },
      ],
      async (baseUrl, attempts) => {
        const client = new LightconeHttp(baseUrl);
        let calls = 0;
        client.setCredentialRestorer(async () => {
          calls += 1;
          await new Promise((resolve) => setTimeout(resolve, 100));
          return true;
        });

        const [first, second] = await Promise.all([
          client.get<{ ok: boolean }>(`${baseUrl}/a`, RetryPolicy.Idempotent),
          client.get<{ ok: boolean }>(`${baseUrl}/b`, RetryPolicy.Idempotent),
        ]);

        assert.equal(first.ok, true);
        assert.equal(second.ok, true);
        assert.equal(calls, 1);
        assert.equal(attempts(), 4);
      },
    );
  });

  it("joiners share the restoration's deadline, not their own", async () => {
    // A joiner arriving mid-restoration must give up when the RESTORATION
    // times out, not a full timeout after it joined — otherwise it would sit
    // listening to the abandoned (zombie) restoration while a replacement
    // runs, and could act on the zombie's late outcome.
    await withServer(
      [
        { status: 401, body: "Unauthorized" },
        { status: 401, body: "Unauthorized" },
      ],
      async (baseUrl) => {
        const client = new LightconeHttp(baseUrl);
        (client as unknown as { credentialRestoreTimeoutMs: number }).credentialRestoreTimeoutMs =
          200;
        client.setCredentialRestorer(() => new Promise<boolean>(() => {}));

        const leader = assert.rejects(
          () => client.get(`${baseUrl}/a`, RetryPolicy.Idempotent),
          (error) => isUnauthorized(error),
        );
        await new Promise((resolve) => setTimeout(resolve, 100));
        const joinedAt = Date.now();
        await assert.rejects(
          () => client.get(`${baseUrl}/b`, RetryPolicy.Idempotent),
          (error) => isUnauthorized(error),
        );
        // ~100ms left on the shared deadline when it joined; a per-waiter
        // timer would have kept it waiting the full 200ms.
        assert(Date.now() - joinedAt < 180);
        await leader;
      },
    );
  });

  it("times out a hung restorer and stays usable", async () => {
    await withServer(
      [
        { status: 401, body: "Unauthorized" },
        { status: 401, body: "Unauthorized" },
        { status: 200, body: '{"status":"success","body":{"ok":true}}' },
      ],
      async (baseUrl, attempts) => {
        const client = new LightconeHttp(baseUrl);
        (client as unknown as { credentialRestoreTimeoutMs: number }).credentialRestoreTimeoutMs =
          200;
        let hungCalls = 0;
        let hungAborted = false;
        client.setCredentialRestorer((signal) => {
          hungCalls += 1;
          signal?.addEventListener("abort", () => {
            hungAborted = true;
          });
          return new Promise<boolean>(() => {});
        });

        const started = Date.now();
        await assert.rejects(
          () => client.get(`${baseUrl}/me`, RetryPolicy.Idempotent),
          (error) => {
            assert(isUnauthorized(error));
            return true;
          },
        );
        assert(Date.now() - started >= 150);
        assert.equal(hungCalls, 1);
        // The timeout aborted the hung restoration's signal — a well-behaved
        // restorer stops on it instead of racing the next restoration.
        assert.equal(hungAborted, true);

        // The client is not stuck "restoring": a replacement restorer works.
        const stub = stubRestorer(true);
        client.setCredentialRestorer(stub.restorer);
        const body = await client.get<{ ok: boolean }>(
          `${baseUrl}/again`,
          RetryPolicy.Idempotent,
        );
        assert.equal(body.ok, true);
        assert.equal(attempts(), 3);
      },
    );
  });

  it("preserves the original 401 when the restorer throws", async () => {
    await withServer(
      [{ status: 401, body: "Unauthorized" }],
      async (baseUrl, attempts) => {
        const client = new LightconeHttp(baseUrl);
        let calls = 0;
        client.setCredentialRestorer(async () => {
          calls += 1;
          throw new Error("restorer exploded");
        });

        await assert.rejects(
          () => client.get(`${baseUrl}/me`, RetryPolicy.Idempotent),
          (error) => {
            // The auth error, not the restorer's failure, reaches the caller.
            assert(isUnauthorized(error));
            return true;
          },
        );
        assert.equal(attempts(), 1);
        assert.equal(calls, 1);
      },
    );
  });

  it("does not follow API redirects", async () => {
    await withServer([], async (foreignOrigin, foreignAttempts) => {
      await withServer(
        [
          {
            status: 302,
            body: "",
            location: `${foreignOrigin}/steal`,
          },
        ],
        async (baseUrl, attempts) => {
          const client = new LightconeHttp(baseUrl);
          const stub = stubRestorer(true);
          client.setCredentialRestorer(stub.restorer);

          await assert.rejects(
            () => client.get(`${baseUrl}/me`, RetryPolicy.None),
            (error) => {
              assert(!isUnauthorized(error));
              return true;
            },
          );

          assert.equal(attempts(), 1);
          assert.equal(foreignAttempts(), 0);
          assert.equal(stub.calls(), 0);
        },
      );
    });
  });

  it("does not capture cookieOverride Set-Cookie into the shared token", async () => {
    await withServer(
      [
        {
          status: 200,
          body: '{"status":"success","body":{"ok":true}}',
          setCookie: "lightcone-token=evil-token; Path=/",
        },
        { status: 200, body: '{"status":"success","body":{"ok":true}}' },
      ],
      async (baseUrl, _attempts, cookiesSeen) => {
        const client = new LightconeHttp(baseUrl);

        await client.getWithCookies(`${baseUrl}/ssr`, RetryPolicy.Idempotent, "privy-token=fwd");
        await client.get(`${baseUrl}/me`, RetryPolicy.Idempotent);

        // The override response's Set-Cookie must not have been captured, so
        // the follow-up session request carries no cookie at all.
        assert.equal(cookiesSeen()[1], undefined);
      },
    );
  });

  it("does not consult the restorer for cookieOverride 401s", async () => {
    await withServer(
      [{ status: 401, body: "Unauthorized" }],
      async (baseUrl, attempts) => {
        const client = new LightconeHttp(baseUrl);
        const stub = stubRestorer(true);
        client.setCredentialRestorer(stub.restorer);

        await assert.rejects(
          () => client.getWithCookies(`${baseUrl}/ssr`, RetryPolicy.Idempotent, "privy-token=stale"),
          (error) => {
            assert(isUnauthorized(error));
            return true;
          },
        );

        assert.equal(attempts(), 1);
        assert.equal(stub.calls(), 0);
      },
    );
  });

  it("reports enveloped 401s as unauthorized via httpStatus", async () => {
    await withServer(
      [
        {
          status: 401,
          body: '{"status":"error","error_details":{"reason":"session expired","error_code":"SESSION_EXPIRED"}}',
        },
      ],
      async (baseUrl) => {
        const client = new LightconeHttp(baseUrl);

        await assert.rejects(
          () => client.get(`${baseUrl}/me`, RetryPolicy.Idempotent),
          (error) => {
            assert(error instanceof SdkError);
            assert.equal(error.variant, "ApiRejected");
            assert.equal(error.apiRejectedDetails?.httpStatus, 401);
            assert.equal(error.apiRejectedDetails?.reason, "session expired");
            assert(isUnauthorized(error));
            return true;
          },
        );
      },
    );
  });

  it("sends no cookie and consults no restorer for a foreign-origin 401", async () => {
    // The client's API origin is server A; the request goes to server B.
    // B answering 401 must neither receive the session cookie nor trigger a
    // credential restoration (a foreign endpoint could otherwise phish the
    // restored cookie via the replay).
    await withServer(
      [
        {
          status: 200,
          body: '{"status":"success","body":{"ok":true}}',
          setCookie: "lightcone-token=secret-token; Path=/",
        },
      ],
      async (apiOrigin, apiAttempts, apiCookies) => {
        await withServer(
          [{ status: 401, body: "Unauthorized" }],
          async (foreignOrigin, foreignAttempts, foreignCookies) => {
            const client = new LightconeHttp(apiOrigin);
            const stub = stubRestorer(true);
            client.setCredentialRestorer(stub.restorer);

            // Seed the session token via a same-origin response, and prove
            // the gate doesn't over-block: the second same-origin request
            // must carry the cookie.
            await client.get(`${apiOrigin}/login`, RetryPolicy.Idempotent);

            await assert.rejects(
              () => client.get(`${foreignOrigin}/me`, RetryPolicy.Idempotent),
              (error) => {
                assert(isUnauthorized(error));
                return true;
              },
            );

            assert.equal(foreignAttempts(), 1);
            assert.equal(stub.calls(), 0);
            assert.equal(foreignCookies()[0], undefined);
            assert.equal(apiAttempts(), 1);
            assert.equal(apiCookies()[0], undefined);
          },
        );
      },
    );
  });

  it("carries the session cookie on same-origin requests after capture", async () => {
    // Companion to the foreign-origin test: the gate must not over-block.
    await withServer(
      [
        {
          status: 200,
          body: '{"status":"success","body":{"ok":true}}',
          setCookie: "lightcone-token=secret-token; Path=/",
        },
        { status: 200, body: '{"status":"success","body":{"ok":true}}' },
      ],
      async (baseUrl, attempts, cookiesSeen) => {
        const client = new LightconeHttp(baseUrl);
        await client.get(`${baseUrl}/login`, RetryPolicy.Idempotent);
        await client.get(`${baseUrl}/me`, RetryPolicy.Idempotent);

        assert.equal(attempts(), 2);
        assert.equal(cookiesSeen()[1], "lightcone-token=secret-token");
      },
    );
  });

  it("skips the restorer for no-restore POSTs (login/logout)", async () => {
    await withServer(
      [{ status: 401, body: "Unauthorized" }],
      async (baseUrl, attempts) => {
        const client = new LightconeHttp(baseUrl);
        const stub = stubRestorer(true);
        client.setCredentialRestorer(stub.restorer);

        await assert.rejects(
          () =>
            client.postWithoutCredentialRestore(
              `${baseUrl}/api/auth/login_or_register_with_message`,
              { message: "m" },
              RetryPolicy.None,
            ),
          (error) => {
            assert(isUnauthorized(error));
            return true;
          },
        );

        assert.equal(attempts(), 1);
        assert.equal(stub.calls(), 0);
      },
    );
  });

  it("terminates a reentrant restorer and consults it once", async () => {
    // A restorer that re-logins through the SDK: its nested request also
    // 401s. The client-wide single-flight flag must stop that nested 401
    // from starting a second restoration.
    await withServer(
      [
        { status: 401, body: "Unauthorized" },
        { status: 401, body: "Unauthorized" },
      ],
      async (baseUrl, attempts) => {
        const client = new LightconeHttp(baseUrl);
        let calls = 0;
        client.setCredentialRestorer(async () => {
          calls += 1;
          // Restorer-internal SDK calls must use the no-restore variant: a
          // restore-enabled call here would await its own restoration and
          // only the restoration timeout would rescue it.
          await client
            .getWithoutCredentialRestore(`${baseUrl}/nested`, RetryPolicy.None)
            .catch(() => undefined);
          return false;
        });

        await assert.rejects(
          () => client.get(`${baseUrl}/me`, RetryPolicy.Idempotent),
          (error) => {
            assert(isUnauthorized(error));
            return true;
          },
        );

        // Outer request + the restorer's nested request; no replay and no
        // second restoration from the nested 401.
        assert.equal(attempts(), 2);
        assert.equal(calls, 1);
      },
    );
  });
});

describe("auth logout error propagation", () => {
  function authWith(baseUrl: string) {
    const http = new LightconeHttp(baseUrl);
    const credentialWrites: unknown[] = [];
    const auth = new Auth({
      http,
      authState: {
        getCredentials: () => undefined,
        setCredentials: (credentials) => {
          credentialWrites.push(credentials);
        },
        clearCaches: async () => {},
      },
    });
    return { auth, http, credentialWrites };
  }

  it("propagates server failure after clearing local state", async () => {
    // The app's logout teardown gate reads this rejection to decide whether
    // the WebSocket may reconnect — a swallowed failure would let it restart
    // with a still-valid server-side cookie.
    await withServer(
      [{ status: 500, body: '{"status":"error","error_details":{"reason":"session store down"}}' }],
      async (baseUrl) => {
        const { auth, credentialWrites } = authWith(baseUrl);
        await assert.rejects(() => auth.logout());
        // Local state was still cleared before the rethrow.
        assert.deepEqual(credentialWrites, [undefined]);
      },
    );
  });

  it("presents the token and exposes every incomplete revocation outcome", async () => {
    for (const code of [
      "AUTH_SESSION_UNAVAILABLE",
      "TOKEN_REVOCATION_UNAVAILABLE",
      "TOKEN_REVOCATION_RECOVERY_UNAVAILABLE",
      "TOKEN_REVOCATION_FENCE_PENDING",
      "AMBIGUOUS_LIGHTCONE_TOKEN",
    ]) {
      await withServer([
        { status: 200, body: '{"status":"success","body":{}}', setCookie: "lightcone-token=live-cookie; Path=/; HttpOnly; Secure; SameSite=Strict" },
        { status: code === "AMBIGUOUS_LIGHTCONE_TOKEN" ? 400 : 503,
          body: JSON.stringify({ status: "error", error_details: { reason: "incomplete", error_code: code } }) },
      ], async (baseUrl, attempts, cookiesSeen) => {
        const { auth, http, credentialWrites } = authWith(baseUrl);
        await http.get(`${baseUrl}/seed`, RetryPolicy.None);
        await assert.rejects(() => auth.logout(), (error: unknown) =>
          error instanceof SdkError && error.apiRejectedDetails?.errorCode === code);
        assert.deepEqual(cookiesSeen(), [undefined, "lightcone-token=live-cookie"]);
        assert.equal(attempts(), 2);
        assert.deepEqual(credentialWrites, [undefined]);
        assert.equal(await http.authTokenRef()(), undefined);
      });
    }
  });

  it("treats 401 as success (already logged out)", async () => {
    await withServer([{ status: 401, body: "Unauthorized" }], async (baseUrl) => {
      const { auth, credentialWrites } = authWith(baseUrl);
      await auth.logout();
      assert.deepEqual(credentialWrites, [undefined]);
    });
  });
});

describe("API key transport", () => {
  async function withHeaderCapture(
    test: (baseUrl: string, headersSeen: () => (string | undefined)[]) => Promise<void>,
  ): Promise<void> {
    const seen: (string | undefined)[] = [];
    const server = createServer((request, response) => {
      seen.push(request.headers["x-lightcone-api-key"] as string | undefined);
      response.writeHead(200, { "content-type": "application/json" });
      response.end('{"status":"success","body":{"ok":true}}');
    });
    await listen(server);
    const address = server.address() as AddressInfo;
    try {
      await test(`http://127.0.0.1:${address.port}`, () => seen);
    } finally {
      await close(server);
    }
  }

  it("sends the configured key only to the API origin", async () => {
    await withHeaderCapture(async (baseUrl, headersSeen) => {
      const http = new LightconeHttp(baseUrl, { apiKey: "lc_local_key" });
      assert.equal(http.hasApiKey(), true);
      for (const route of ["submit", "cancel", "cancel-all"]) {
        await http.post(`${baseUrl}/api/orders/${route}`, {}, RetryPolicy.None);
      }
      const foreign = new LightconeHttp("http://127.0.0.1:1", { apiKey: "lc_local_key" });
      await foreign.post(`${baseUrl}/api/orders/submit`, {}, RetryPolicy.None);
      await http.get(`${baseUrl}/api/markets`, RetryPolicy.None);
      await http.get(`${baseUrl}/api/orders/submit`, RetryPolicy.None);
      await http.post(`${baseUrl}/api/auth/login`, {}, RetryPolicy.None);
      assert.deepEqual(headersSeen(), ["lc_local_key", "lc_local_key", "lc_local_key", undefined, undefined, undefined, undefined]);
    });
  });

  it("sends no key header when none is configured", async () => {
    await withHeaderCapture(async (baseUrl, headersSeen) => {
      const http = new LightconeHttp(baseUrl, { apiKey: "  " });
      assert.equal(http.hasApiKey(), false);
      await http.get(`${baseUrl}/api/markets`, RetryPolicy.None);
      assert.deepEqual(headersSeen(), [undefined]);
    });
  });

  it("rejects API key configuration in a browser", () => {
    const previousWindow = Object.getOwnPropertyDescriptor(globalThis, "window");
    Object.defineProperty(globalThis, "window", { configurable: true, value: {} });
    try {
      assert.throws(
        () => new LightconeHttp("https://api.lightcone.xyz", { apiKey: "test-key" }),
        /server-side SDK builds/,
      );
    } finally {
      if (previousWindow) {
        Object.defineProperty(globalThis, "window", previousWindow);
      } else {
        Reflect.deleteProperty(globalThis, "window");
      }
    }
  });

  it("rejects API key configuration in a browser worker", () => {
    const previousImportScripts = Object.getOwnPropertyDescriptor(globalThis, "importScripts");
    Object.defineProperty(globalThis, "importScripts", {
      configurable: true,
      value: () => undefined,
    });
    try {
      assert.throws(
        () => new LightconeHttp("https://api.lightcone.xyz", { apiKey: "test-key" }),
        /server-side SDK builds/,
      );
    } finally {
      if (previousImportScripts) {
        Object.defineProperty(globalThis, "importScripts", previousImportScripts);
      } else {
        Reflect.deleteProperty(globalThis, "importScripts");
      }
    }
  });

  it("lets the browser own cookies in a worker", async () => {
    const previousImportScripts = Object.getOwnPropertyDescriptor(globalThis, "importScripts");
    const previousFetch = globalThis.fetch;
    const seen: RequestInit[] = [];
    Object.defineProperty(globalThis, "importScripts", {
      configurable: true,
      value: () => undefined,
    });
    globalThis.fetch = (async (_url, options) => {
      seen.push(options ?? {});
      return new Response('{"status":"success","body":{"ok":true}}', {
        status: 200,
        headers: { "content-type": "application/json" },
      });
    }) as typeof fetch;
    try {
      const http = new LightconeHttp("https://api.example.com");
      await http.getWithCookies(
        "https://api.example.com/api/markets",
        RetryPolicy.None,
        "lightcone-token=visitor-session",
      );
      assert.equal(seen.length, 1);
      assert.equal(seen[0].credentials, "include");
      assert.equal(new Headers(seen[0].headers).has("Cookie"), false);
    } finally {
      globalThis.fetch = previousFetch;
      if (previousImportScripts) {
        Object.defineProperty(globalThis, "importScripts", previousImportScripts);
      } else {
        Reflect.deleteProperty(globalThis, "importScripts");
      }
    }
  });

  it("rejects an API key on a non-loopback cleartext origin", () => {
    assert.throws(
      () => new LightconeHttp("http://api.example.com", { apiKey: "test-key" }),
      /HTTPS or a loopback HTTP origin/,
    );
    assert.equal(
      new LightconeHttp("http://127.0.0.1:3001", { apiKey: "test-key" }).hasApiKey(),
      true,
    );
    assert.equal(
      new LightconeHttp("https://api.example.com", { apiKey: "test-key" }).hasApiKey(),
      true,
    );
  });
});


describe("manual Privy retry contract", () => {
  for (const [header, delay] of [["60", 60000], ["0.0001", 1], [undefined, undefined], ["-1", undefined], ["NaN", undefined], ["5junk", undefined]] as const) {
    it(`returns the first rejection and retains timing ${header}`, async () => {
      await withServer([{status:503, retryAfter:header, body:JSON.stringify({status:"error",error_details:{reason:"Unavailable",error_code:"PRIVY_VERIFICATION_UNAVAILABLE"}})}], async (base, attempts) => {
        const http = new LightconeHttp(base);
        let restores = 0;
        http.setCredentialRestorer(async () => { restores++; return true; });
        await assert.rejects(() => http.get(base, RetryPolicy.Idempotent), (error: unknown) => {
          assert(error instanceof SdkError);
          assert.equal(error.apiRejectedDetails?.isPrivyVerificationUnavailable(), true);
          assert.equal(error.apiRejectedDetails?.retryAfterMs, delay);
          return true;
        });
        assert.equal(attempts(), 1);
        assert.equal(restores, 0);
      });
    });
  }
  it("session checks retain cached credentials on authority failure", async () => {
    await withServer(Array.from({length:2},()=>({status:503, body:JSON.stringify({status:"error",error_details:{reason:"Unavailable",error_code:"PRIVY_VERIFICATION_UNAVAILABLE"}})})), async (base, attempts) => {
      let writes = 0;
      const cached = {user_id:"existing-account",wallet_address:"wallet",expires_at:new Date(Date.now()+60_000)};
      const auth = new Auth({http:new LightconeHttp(base),authState:{getCredentials:()=>cached,setCredentials:()=>{writes++;},clearCaches:async()=>{}}});
      await assert.rejects(() => auth.checkSession());
      assert.equal(writes,0);
      assert.equal(attempts(),1);
      await assert.rejects(() => auth.registerPrivy({attempted_identity:{type:"email",email:"fixture@example.test"}}));
      assert.equal(writes,0);
      assert.equal(attempts(),2);
    });
  });
});

describe("Privy policy and malformed-response regressions", () => {
  const rejection = JSON.stringify({ status: "error", error_details: {
    reason: "Unavailable", error_code: "PRIVY_VERIFICATION_UNAVAILABLE",
  } });
  for (const policy of [RetryPolicy.Idempotent, fastRetry([503]), RetryPolicy.None]) {
    for (const method of ["GET", "POST"]) {
      it(`${method} surfaces Privy once under policy ${JSON.stringify(policy)}`, async () => {
        await withServer([{ status: 503, body: rejection, retryAfter: "60" }], async (base, attempts) => {
          const http = new LightconeHttp(base);
          let restores = 0;
          http.setCredentialRestorer(async () => { restores++; return true; });
          const call = () => method === "GET" ? http.get(base, policy) : http.post(base, { signature: "original" }, policy);
          await assert.rejects(call, (error: unknown) => {
            assert(error instanceof SdkError);
            assert.equal(error.apiRejectedDetails?.isPrivyVerificationUnavailable(), true);
            assert.equal(error.apiRejectedDetails?.retryAfterMs, 60000);
            return true;
          });
          assert.equal(attempts(), 1);
          assert.equal(restores, 0);
        });
      });
    }
  }
  for (const details of [123, { reason: "Unavailable", error_code: "OTHER", rejection_code: 123 }]) {
    it(`malformed 503 details retain generic retry: ${JSON.stringify(details)}`, async () => {
      await withServer([
        { status: 503, body: JSON.stringify({ status: "error", error_details: details }) },
        { status: 200, body: JSON.stringify({ status: "success", body: { ok: true } }) },
      ], async (base, attempts) => {
        assert.deepEqual(await new LightconeHttp(base).get(base, fastRetry([503])), { ok: true });
        assert.equal(attempts(), 2);
      });
    });
  }
  for (const [header, expected] of [
    ["Wed, 21 Oct 2015 07:28:05 GMT", 5000],
    ["Wed, 21 Oct 2015 07:27:00 GMT", 0],
    ["Wed, invalid", undefined],
    ["Wednesday, 21-Oct-15 07:28:05 GMT", 5000],
    ["Wed Oct 21 07:28:05 2015", 5000],
    ["Thu Oct 01 07:28:05 2015", 0],
    ["Thu Oct  1 07:28:05 2015", 0],
    ["Wed, 31 Feb 2027 07:28:05 GMT", undefined],
    ["Oct 21 2099", undefined],
    ["Wed 1", undefined],
    ["Wed, 21 Oct 2015 07:28:05 +0000", undefined],
    ["Thu, 21 Oct 2015 07:28:05 GMT", undefined],
  ] as const) {
    it(`retains HTTP-date guidance ${header}`, async (context) => {
      context.mock.method(Date, "now", () => Date.parse("Wed, 21 Oct 2015 07:28:00 GMT"));
      await withServer([{ status: 503, body: rejection, retryAfter: header }], async (base, attempts) => {
        await assert.rejects(() => new LightconeHttp(base).get(base, RetryPolicy.None), (error: unknown) => {
          assert(error instanceof SdkError);
          assert.equal(error.apiRejectedDetails?.retryAfterMs, expected);
          return true;
        });
        assert.equal(attempts(), 1);
      });
    });
  }
});

for (const operation of ["submit", "cancel", "cancelAll"] as const) {
  it(`public ${operation} surfaces one Privy rejection`, async () => {
    const { LightconeClient } = await import("../src/client");
    const { asPubkeyStr, asOrderBookId } = await import("../src/shared");
    const wallet = asPubkeyStr("11111111111111111111111111111111");
    const orderbook = asOrderBookId(wallet);
    const rules = { orderbook_id: orderbook, base_decimals: 8, quote_decimals: 6, price_decimals: 4,
      trading_rules: { base_size_decimals: 5, max_price_decimals: 1, max_price_significant_figures: 5,
        integer_prices_always_allowed: true, price_quantum: "0.1000", price_quantum_raw: "1000",
        base_size_quantum: "0.00001000", base_size_quantum_raw: "1000" } };
    const responses: TestResponse[] = operation === "submit" ? [{ status: 200, body: JSON.stringify({ status: "success", body: rules }) }] : [];
    responses.push({ status: 503, retryAfter: "60", body: JSON.stringify({ status: "error", error_details: {
      reason: "Unavailable", error_code: "PRIVY_VERIFICATION_UNAVAILABLE",
    } }) });
    await withServer(responses, async (base, attempts) => {
      const orders = LightconeClient.builder().baseUrl(base).build().orders();
      const call = () => operation === "submit" ? orders.submit({ maker: wallet, nonce: 0, salt: 0n,
        market_pubkey: wallet, base_token: wallet, quote_token: wallet, side: 0,
        amount_in: 15_185_088n, amount_out: 123_456_000n, expiration: 0n, signature: "fixture", orderbook_id: orderbook,
      }) : operation === "cancel" ? orders.cancel({ order_hash: "fixture", maker: wallet, signature: "fixture" })
        : orders.cancelAll({ user_pubkey: wallet, orderbook_id: orderbook, signature: "fixture", timestamp: 0, salt: "fixture" });
      await assert.rejects(call, (error: unknown) => {
        assert(error instanceof SdkError);
        assert.equal(error.apiRejectedDetails?.isPrivyVerificationUnavailable(), true);
        assert.equal(error.apiRejectedDetails?.retryAfterMs, 60000);
        return true;
      });
      assert.equal(attempts(), operation === "submit" ? 2 : 1);
    });
  });
}


it("anchors HTTP-date guidance before consuming a delayed error body", async (context) => {
  let now = Date.parse("Wed, 21 Oct 2015 07:28:00 GMT");
  context.mock.method(Date, "now", () => now);
  const response = new Response(JSON.stringify({ status: "error", error_details: {
    reason: "Unavailable", error_code: "PRIVY_VERIFICATION_UNAVAILABLE",
  } }), { status: 503, headers: { "Retry-After": "Wed, 21 Oct 2015 07:28:05 GMT" } });
  const readBody = response.text.bind(response);
  context.mock.method(response, "text", async () => { now += 2000; return readBody(); });
  context.mock.method(globalThis, "fetch", async () => response);
  await assert.rejects(() => new LightconeHttp("http://localhost").get("/test", RetryPolicy.None), (error: unknown) => {
    assert(error instanceof SdkError);
    assert.equal(error.apiRejectedDetails?.retryAfterMs, 5000);
    return true;
  });
});

for (const [milliseconds, expected] of [["1.1", 2], ["0", 0], ["bad", 5000], ["9007199254740992", 5000]] as const) {
  it(`preserves millisecond guidance precedence ${milliseconds}`, async () => {
    await withServer([{ status: 503, retryAfterMs: milliseconds, retryAfter: "5", body: JSON.stringify({status:"error", error_details:{reason:"Unavailable", error_code:"PRIVY_VERIFICATION_UNAVAILABLE"}}) }], async base => {
      await assert.rejects(() => new LightconeHttp(base).get(base, fastRetry([503])), (error: unknown) => {
        assert(error instanceof SdkError); assert.equal(error.apiRejectedDetails?.retryAfterMs, expected); return true;
      });
    });
  });
}
it("malformed optional fields cannot enable Privy replay", async () => {
  await withServer([{status:503, body:JSON.stringify({status:"error",error_details:{reason:"Unavailable",error_code:"PRIVY_VERIFICATION_UNAVAILABLE",rejection_code:123}})}], async (base, attempts) => {
    await assert.rejects(() => new LightconeHttp(base).get(base, fastRetry([503])), (error:unknown) => {
      assert(error instanceof SdkError); assert.equal(error.apiRejectedDetails?.isPrivyVerificationUnavailable(), true); return true;
    });
    assert.equal(attempts(),1);
  });
});
it("obsolete HTTP years use the receipt year", async context => {
  context.mock.method(Date,"now",() => Date.UTC(2026,0,1));
  await withServer([{status:503,retryAfter:"Wednesday, 01-Jan-70 00:00:00 GMT",body:JSON.stringify({status:"error",error_details:{reason:"Unavailable",error_code:"PRIVY_VERIFICATION_UNAVAILABLE"}})}],async base => {
    await assert.rejects(() => new LightconeHttp(base).get(base,RetryPolicy.None),(error:unknown) => {
      assert(error instanceof SdkError); assert.equal(error.apiRejectedDetails?.retryAfterMs,Date.UTC(2070,0,1)-Date.UTC(2026,0,1));return true;
    });
  });
});
it("chunks generic retry timers beyond the runtime limit", async context => {
  const waits:number[]=[];
  context.mock.method(globalThis,"setTimeout",(callback:()=>void,ms:number)=>{
    if(ms!==180000) {waits.push(ms);queueMicrotask(callback);}
    return 0 as unknown as ReturnType<typeof setTimeout>;
  });
  let calls=0;
  context.mock.method(globalThis,"fetch",async()=>++calls===1
    ? new Response("unavailable",{status:503,headers:{"Retry-After-MS":"2147483648"}})
    : new Response(JSON.stringify({status:"success",body:{ok:true}})));
  assert.deepEqual(await new LightconeHttp("http://localhost").get("/test",fastRetry([503])),{ok:true});
  assert.deepEqual(waits,[2147483647,1]);
});
