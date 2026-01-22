import { describe, it, expect } from "vitest";
import { ApiError, getErrorMessage } from "./error";

describe("ApiError", () => {
  describe("static constructors", () => {
    it("creates http error", () => {
      const err = ApiError.http("network failure");
      expect(err.variant).toBe("Http");
      expect(err.message).toContain("network failure");
      expect(err.statusCode).toBeUndefined();
    });

    it("creates notFound error", () => {
      const err = ApiError.notFound("market not found");
      expect(err.variant).toBe("NotFound");
      expect(err.statusCode).toBe(404);
    });

    it("creates badRequest error", () => {
      const err = ApiError.badRequest("invalid params");
      expect(err.variant).toBe("BadRequest");
      expect(err.statusCode).toBe(400);
    });

    it("creates forbidden error", () => {
      const err = ApiError.forbidden("invalid signature");
      expect(err.variant).toBe("Forbidden");
      expect(err.statusCode).toBe(403);
    });

    it("creates conflict error", () => {
      const err = ApiError.conflict("order exists");
      expect(err.variant).toBe("Conflict");
      expect(err.statusCode).toBe(409);
    });

    it("creates serverError", () => {
      const err = ApiError.serverError("internal error");
      expect(err.variant).toBe("ServerError");
      expect(err.statusCode).toBe(500);
    });

    it("creates deserialize error", () => {
      const err = ApiError.deserialize("JSON parse failed");
      expect(err.variant).toBe("Deserialize");
    });

    it("creates rateLimited error", () => {
      const err = ApiError.rateLimited("too many requests");
      expect(err.variant).toBe("RateLimited");
      expect(err.statusCode).toBe(429);
      expect(err.message).toContain("too many requests");
    });

    it("creates unauthorized error", () => {
      const err = ApiError.unauthorized("invalid token");
      expect(err.variant).toBe("Unauthorized");
      expect(err.statusCode).toBe(401);
      expect(err.message).toContain("invalid token");
    });
  });

  describe("fromStatus", () => {
    it("maps 400 to BadRequest", () => {
      const err = ApiError.fromStatus(400, "bad");
      expect(err.variant).toBe("BadRequest");
    });

    it("maps 401 to Unauthorized", () => {
      const err = ApiError.fromStatus(401, "unauthorized");
      expect(err.variant).toBe("Unauthorized");
      expect(err.statusCode).toBe(401);
    });

    it("maps 403 to Forbidden", () => {
      const err = ApiError.fromStatus(403, "forbidden");
      expect(err.variant).toBe("Forbidden");
    });

    it("maps 404 to NotFound", () => {
      const err = ApiError.fromStatus(404, "not found");
      expect(err.variant).toBe("NotFound");
    });

    it("maps 409 to Conflict", () => {
      const err = ApiError.fromStatus(409, "conflict");
      expect(err.variant).toBe("Conflict");
    });

    it("maps 429 to RateLimited", () => {
      const err = ApiError.fromStatus(429, "too many requests");
      expect(err.variant).toBe("RateLimited");
      expect(err.statusCode).toBe(429);
    });

    it("maps 500 to ServerError", () => {
      const err = ApiError.fromStatus(500, "server error");
      expect(err.variant).toBe("ServerError");
    });

    it("maps 502 to ServerError", () => {
      const err = ApiError.fromStatus(502, "bad gateway");
      expect(err.variant).toBe("ServerError");
    });

    it("maps unknown status to UnexpectedStatus", () => {
      const err = ApiError.fromStatus(418, "teapot");
      expect(err.variant).toBe("UnexpectedStatus");
      expect(err.statusCode).toBe(418);
    });
  });

  describe("error inheritance", () => {
    it("is an instance of Error", () => {
      const err = ApiError.http("test");
      expect(err instanceof Error).toBe(true);
      expect(err instanceof ApiError).toBe(true);
    });

    it("has correct name", () => {
      const err = ApiError.http("test");
      expect(err.name).toBe("ApiError");
    });
  });
});

describe("getErrorMessage", () => {
  it("extracts message field", () => {
    expect(getErrorMessage({ message: "error message" })).toBe("error message");
  });

  it("extracts error field", () => {
    expect(getErrorMessage({ error: "error field" })).toBe("error field");
  });

  it("extracts details field", () => {
    expect(getErrorMessage({ details: "error details" })).toBe("error details");
  });

  it("prefers message over error", () => {
    expect(getErrorMessage({ message: "msg", error: "err" })).toBe("msg");
  });

  it("returns Unknown error for empty response", () => {
    expect(getErrorMessage({})).toBe("Unknown error");
  });
});
