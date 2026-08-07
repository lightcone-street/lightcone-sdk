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
    return { auth, credentialWrites };
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

  it("treats 401 as success (already logged out)", async () => {
    await withServer([{ status: 401, body: "Unauthorized" }], async (baseUrl) => {
      const { auth, credentialWrites } = authWith(baseUrl);
      await auth.logout();
      assert.deepEqual(credentialWrites, [undefined]);
    });
  });
});
