// One logical request owns its body and correlation ID across bounded retries.
// Browser cookies remain browser-owned; explicit SSR cookies never enter the jar.
module RetryConfig = {
  type t = {
    maxRetries: int,
    initialDelayMs: float,
    maxDelayMs: float,
    backoffFactor: float,
    jitter: bool,
    retryableStatuses: array<int>,
  }
}
module RetryPolicy = {
  type t = NoRetry | Idempotent | Custom(RetryConfig.t)
}
type restorer = unit => promise<bool>
type t = {
  baseUrl: string,
  mutable token: option<string>,
  mutable apiKey: option<string>,
  mutable restorer: option<restorer>,
  mutable restoration: option<promise<bool>>,
  mutable credentialGeneration: int,
}
let isBrowser: unit => bool = %raw(`function () {
  return typeof window !== 'undefined' || typeof WorkerGlobalScope !== 'undefined' || typeof importScripts !== 'undefined';
}`)
let now: unit => float = %raw(`() => Date.now()`)
let originAllowed: (string, string) => bool = %raw(`function(base, path) {
  try { return new URL(path, base).origin === new URL(base).origin; } catch { return false; }
}`)
let secureOrigin: string => bool = %raw(`function(base) {
  try { const u = new URL(base); return !u.username && !u.password &&
    (u.protocol === 'https:' || (u.protocol === 'http:' && ['localhost','127.0.0.1','[::1]'].includes(u.hostname)));
  } catch { return false; }
}`)
let readSetCookies: Fetch.headers => array<string> = %raw(`headers =>
  typeof headers.getSetCookie === 'function' ? headers.getSetCookie() : []`)
let parseRetryAfter: (Fetch.headers, float) => option<float> = %raw(`function(headers, received) {
function httpDateMs(value, receivedAtMs) {
  const modern = /^(Sun|Mon|Tue|Wed|Thu|Fri|Sat), (\d{2}) (Jan|Feb|Mar|Apr|May|Jun|Jul|Aug|Sep|Oct|Nov|Dec) (\d{4}) (\d{2}):(\d{2}):(\d{2}) GMT$/.exec(value);
  const obsolete = /^(Sunday|Monday|Tuesday|Wednesday|Thursday|Friday|Saturday), (\d{2})-(Jan|Feb|Mar|Apr|May|Jun|Jul|Aug|Sep|Oct|Nov|Dec)-(\d{2}) (\d{2}):(\d{2}):(\d{2}) GMT$/.exec(value);
  const ascii = /^(Sun|Mon|Tue|Wed|Thu|Fri|Sat) (Jan|Feb|Mar|Apr|May|Jun|Jul|Aug|Sep|Oct|Nov|Dec) ([ \d]\d) (\d{2}):(\d{2}):(\d{2}) (\d{4})$/.exec(value);
  const parts = modern ?? obsolete ?? (ascii && [ascii[0], ascii[1], ascii[3], ascii[2], ascii[7], ascii[4], ascii[5], ascii[6]]);
  if (!parts) return undefined;
  const [, weekday = "", day, month = "", year, hour, minute, second] = parts;
  let fullYear = Number(year);
  if (obsolete) {
    const upperYear = new Date(receivedAtMs).getUTCFullYear() + 50;
    fullYear += Math.floor(upperYear / 100) * 100;
    if (fullYear > upperYear) fullYear -= 100;
  }
  const monthIndex = ["Jan", "Feb", "Mar", "Apr", "May", "Jun", "Jul", "Aug", "Sep", "Oct", "Nov", "Dec"].indexOf(month);
  const date = new Date(0);
  date.setUTCFullYear(fullYear, monthIndex, Number(day));
  date.setUTCHours(Number(hour), Number(minute), Number(second), 0);
  if (date.getUTCFullYear() !== fullYear || date.getUTCMonth() !== monthIndex ||
      date.getUTCDate() !== Number(day) || date.getUTCHours() !== Number(hour) ||
      date.getUTCMinutes() !== Number(minute) || date.getUTCSeconds() !== Number(second) ||
      ["Sun", "Mon", "Tue", "Wed", "Thu", "Fri", "Sat"][date.getUTCDay()] !== weekday.slice(0, 3)) return undefined;
  return date.getTime();
}

  for (const [name, scale] of [['retry-after-ms',1],['retry-after',1000]]) {
    const value = headers.get(name)?.trim();
    if (value && /^\d+(?:\.\d+)?$/.test(value)) {
      const ms = Math.ceil(Number(value) * scale);
      if (Number.isSafeInteger(ms) && ms >= 0) return ms;
    }
  }
  const text = headers.get('retry-after')?.trim();
  if (!text || !/[A-Za-z]/.test(text)) return undefined;
  const date = httpDateMs(text, received);
  if (date === undefined) return undefined;
  const delay = Math.max(0, Math.ceil(date - received));
  return Number.isSafeInteger(delay) ? delay : undefined;
}`)
let rec sleepUntil = async deadline => {
  let remaining = deadline -. now()
  if remaining > 0.0 {
    await Promise.make((resolve, _) =>
      setTimeout(() => resolve(), Math.min(remaining, 2147483647.0)->Float.toInt)->ignore
    )
    await sleepUntil(deadline)
  }
}
// The timer is cleared on completion. The losing callback cannot clear a later run.
let boundedRestore: restorer => promise<bool> = %raw(`function(restore) {
  return new Promise(resolve => {
    const timer = setTimeout(() => resolve(false), 30000);
    Promise.resolve().then(restore).then(value => { clearTimeout(timer); resolve(value === true); },
      () => { clearTimeout(timer); resolve(false); });
  });
}`)
let make = baseUrl => {
  baseUrl: baseUrl->String.replaceRegExp(/\/+$/, ""),
  token: None,
  apiKey: None,
  restorer: None,
  restoration: None,
  credentialGeneration: 0,
}
let baseUrl = http => http.baseUrl
let authToken = http =>
  if isBrowser() {
    None
  } else {
    http.token
  }
let setAuthToken = (http, token) =>
  if !isBrowser() {
    http.token = Some(token)
  }
let clearAuthToken = http => {
  http.token = None
  http.credentialGeneration = http.credentialGeneration + 1
}
let setCredentialRestorer = (http, callback) => {
  http.restorer = callback
  http.restoration = None
  http.credentialGeneration = http.credentialGeneration + 1
}
let setServerApiKey = (http, key): result<unit, SdkError.t> => {
  let key = String.trim(key)
  if key == "" {
    http.apiKey = None
    Ok()
  } else if isBrowser() {
    Error(Validation("server API keys are unavailable in browsers and workers"))
  } else if (
    !secureOrigin(http.baseUrl) || String.includes(key, "\r") || String.includes(key, "\n")
  ) {
    Error(Validation("API keys require a secure origin and a valid header value"))
  } else {
    http.apiKey = Some(key)
    Ok()
  }
}
let restore = async (http, generation) => {
  if generation != http.credentialGeneration {
    true
  } else {
    let pending = switch http.restoration {
    | Some(pending) => pending
    | None =>
      let pending = switch http.restorer {
      | None => Promise.resolve(false)
      | Some(callback) => boundedRestore(callback)
      }
      http.restoration = Some(pending)
      pending
    }
    let restored = await pending
    if (
      switch http.restoration {
      | Some(current) => current === pending
      | None => false
      }
    ) {
      http.restoration = None
      if restored {
        http.credentialGeneration = http.credentialGeneration + 1
      }
    }
    restored
  }
}
let defaultConfig: RetryConfig.t = {
  maxRetries: 3,
  initialDelayMs: 200.0,
  maxDelayMs: 10000.0,
  backoffFactor: 2.0,
  jitter: true,
  retryableStatuses: [429, 502, 503, 504],
}
let statusError = (status: int, body: string, retryAfterMs: option<float>): SdkError.t =>
  switch status {
  | 401 => Http(Unauthorized)
  | 400 => Http(BadRequest(body))
  | 404 => Http(NotFound(body))
  | 429 => SdkError.Http(SdkError.HttpError.RateLimited({retryAfterMs: retryAfterMs}))
  | _ => Http(ServerError({status, body}))
  }
let request = async (
  http,
  ~method,
  ~path,
  ~query=[],
  ~body=?,
  ~retry=RetryPolicy.NoRetry,
  ~cookieHeader=?,
  ~restoreCredentials=true,
  ~decode,
) => {
  if (
    !originAllowed(http.baseUrl, path) ||
    !String.startsWith(path, "/") ||
    String.startsWith(path, "//")
  ) {
    Error(SdkError.Validation("request path must belong to the configured API origin"))
  } else {
    let config = switch retry {
    | NoRetry => None
    | Idempotent => Some(defaultConfig)
    | Custom(config) => Some(config)
    }
    let requestId = WebCrypto.randomUUID()
    let serializedBody = body->Option.map(ExactJson.stringify)
    let url =
      http.baseUrl ++
      path ++ if Array.length(query) == 0 {
        ""
      } else {
        "?" ++
        query
        ->Array.map(((key, value)) => `${encodeURIComponent(key)}=${encodeURIComponent(value)}`)
        ->Array.join("&")
      }
    let generation = http.credentialGeneration
    let eligibleRestore =
      restoreCredentials &&
      cookieHeader->Option.isNone &&
      path != "/api/auth/logout" &&
      path != "/api/auth/login_or_register_with_message" &&
      path != "/api/auth/nonce"
    let rec attempt = async (n, replayed) => {
      let headers = Dict.fromArray([("x-request-id", requestId)])
      serializedBody->Option.forEach(_ => Dict.set(headers, "content-type", "application/json"))
      if !isBrowser() {
        switch cookieHeader {
        | Some(cookie) => Dict.set(headers, "cookie", cookie)
        | None =>
          http.token->Option.forEach(token =>
            Dict.set(headers, "cookie", `lightcone-token=${token}`)
          )
        }
        if (
          method == "POST" &&
            Array.includes(
              ["/api/orders/submit", "/api/orders/cancel", "/api/orders/cancel-all"],
              path,
            )
        ) {
          http.apiKey->Option.forEach(key => Dict.set(headers, "x-lightcone-api-key", key))
        }
      }
      let retryLater = async deadline => {
        await sleepUntil(deadline)
        await attempt(n + 1, replayed)
      }
      let backoffDeadline = config->Option.mapOr(now(), config => {
        let delay = Math.min(
          config.initialDelayMs *. Math.pow(config.backoffFactor, ~exp=Int.toFloat(n)),
          config.maxDelayMs,
        )
        now() +.
        Math.max(
          0.0,
          delay *. if config.jitter {
            0.75 +. Math.random() *. 0.5
          } else {
            1.0
          },
        )
      })
      let canRetry = config->Option.mapOr(false, config => n < config.maxRetries)
      let response = switch await Fetch.fetch(
        url,
        {
          method,
          headers,
          body: ?serializedBody,
          credentials: "include",
          redirect: "error",
          signal: Fetch.timeoutSignal(180000),
        },
      ) {
      | response => Ok(response)
      | exception _ => Error(SdkError.Http(Network("request failed")))
      }
      switch response {
      | Error(error) =>
        if canRetry {
          await retryLater(backoffDeadline)
        } else {
          Error(error)
        }
      | Ok(response) =>
        let status = Fetch.status(response)
        let received = now()
        let retryAfterMs = parseRetryAfter(Fetch.responseHeaders(response), received)
        let text = switch await Fetch.text(response) {
        | text => text
        | exception _ => ""
        }
        let result = switch ExactJson.parse(text) {
        | Ok(json) => SdkError.parseApiResponse(decode, json)
        | Error(_) => Error(SdkError.Decode("expected a JSON response body"))
        }
        let result = switch result {
        | Error(ApiRejected(details)) =>
          Error(SdkError.ApiRejected({...details, requestId, httpStatus: status, ?retryAfterMs}))
        | _ if status < 200 || status >= 300 => Error(statusError(status, text, retryAfterMs))
        | result => result
        }
        switch result {
        | Error(error) if SdkError.isPrivyVerificationUnavailable(error) => Error(error)
        | _ if status == 401 && eligibleRestore && !replayed =>
          let restored = await restore(http, generation)
          if restored && config->Option.isSome {
            await attempt(n, true)
          } else {
            result
          }
        | _
          if canRetry &&
          config->Option.mapOr(false, config => Array.includes(config.retryableStatuses, status)) =>
          await retryLater(retryAfterMs->Option.mapOr(backoffDeadline, delay => received +. delay))
        | Ok(_) =>
          if !isBrowser() && cookieHeader->Option.isNone && status >= 200 && status < 300 {
            readSetCookies(Fetch.responseHeaders(response))->Array.forEach(cookie => {
              if String.startsWith(cookie, "lightcone-token=") {
                http.token =
                  cookie
                  ->String.split(";")
                  ->Array.get(0)
                  ->Option.map(value => String.slice(value, ~start=16, ~end=String.length(value)))
              }
            })
          }
          result
        | Error(_) => result
        }
      }
    }
    await attempt(0, false)
  }
}
let get = (
  http,
  ~path,
  ~query=[],
  ~retry=RetryPolicy.Idempotent,
  ~cookieHeader=?,
  ~restoreCredentials=true,
  ~decode,
) =>
  request(http, ~method="GET", ~path, ~query, ~retry, ~cookieHeader?, ~restoreCredentials, ~decode)
let post = (
  http,
  ~path,
  ~body,
  ~retry=RetryPolicy.NoRetry,
  ~cookieHeader=?,
  ~restoreCredentials=true,
  ~decode,
) =>
  request(http, ~method="POST", ~path, ~body, ~retry, ~cookieHeader?, ~restoreCredentials, ~decode)
let delete = (http, ~path, ~retry=RetryPolicy.Idempotent, ~cookieHeader=?, ~decode) =>
  request(http, ~method="DELETE", ~path, ~retry, ~cookieHeader?, ~decode)
