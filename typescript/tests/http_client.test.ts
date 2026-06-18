import { once } from "node:events";
import { createServer, type Server } from "node:http";
import type { AddressInfo } from "node:net";
import { describe, it } from "node:test";
import assert from "node:assert/strict";
import { SdkError } from "../src/error";
import { LightconeHttp } from "../src/http/client";
import { RetryPolicy, type RetryConfig } from "../src/http/retry";

type TestResponse = {
  status: number;
  body: string;
};

async function withServer(
  responses: TestResponse[],
  test: (baseUrl: string, attempts: () => number) => Promise<void>,
): Promise<void> {
  let attempts = 0;
  const queue = [...responses];

  const server = createServer((_request, response) => {
    attempts += 1;
    const next = queue.shift() ?? {
      status: 500,
      body: '{"status":"error","error_details":{"reason":"unexpected extra request"}}',
    };
    response.writeHead(next.status, { "content-type": "application/json" });
    response.end(next.body);
  });

  await listen(server);
  const address = server.address() as AddressInfo;
  try {
    await test(`http://127.0.0.1:${address.port}`, () => attempts);
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
});
