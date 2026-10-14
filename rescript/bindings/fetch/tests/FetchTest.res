open RescriptBun.Test
open RescriptBun.Test.Expect

// Runtime tests for the platform `fetch` binding — run the compiled .res.mjs under Bun
// to prove the JS names (`status`/`ok`/`json`/`text`/`headers.get`), arg shapes, and
// return types. We drive `fetch` against `data:` URLs, which it resolves with zero
// network (Node/Bun/browsers all support them) — so the suite is deterministic and
// offline. data: URLs echo their declared content-type header and body verbatim.
let jsonUrl = "data:application/json,%7B%22ok%22%3Atrue%2C%22n%22%3A7%7D" // {"ok":true,"n":7}
let textUrl = "data:text/plain,hello%20fetch" // "hello fetch"

describe("Fetch — response accessors", () => {
  testAsync("status / ok / statusText on a 200 data: URL", async () => {
    let response = await Fetch.fetch(jsonUrl, {method: "GET"})
    expect(Fetch.status(response))->toBe(200)
    expect(Fetch.ok(response))->toBe(true)
    expect(Fetch.statusText(response))->toBe("OK")
  })

  testAsync("text reads the body as a string", async () => {
    let response = await Fetch.fetch(textUrl, {method: "GET"})
    expect(await Fetch.text(response))->toBe("hello fetch")
  })

  testAsync("json decodes the body to JSON.t (pattern-matched)", async () => {
    // Bare init: every field omitted via `?None` — compiles to a plain `{}`.
    let response = await Fetch.fetch(jsonUrl, {method: ?None})
    switch await Fetch.json(response) {
    | JSON.Object(fields) =>
      switch fields->Dict.get("ok") {
      | Some(JSON.Boolean(value)) => expect(value)->toBe(true)
      | _ => expect("missing ok:bool")->toBe("present")
      }
      switch fields->Dict.get("n") {
      | Some(JSON.Number(number)) => expect(number)->toBe(7.0)
      | _ => expect("missing n:number")->toBe("present")
      }
    | _ => expect("not a JSON object")->toBe("object")
    }
  })
})

describe("Fetch — headers", () => {
  testAsync("responseHeaders + getHeader read a present header", async () => {
    let response = await Fetch.fetch(jsonUrl, {method: "GET"})
    let contentType =
      Fetch.responseHeaders(response)
      ->Fetch.getHeader("content-type")
      ->Null.toOption
      ->Option.getOr("")
    expect(contentType->String.includes("application/json"))->toBe(true)
  })

  testAsync("getHeader returns null (-> None) for an absent header", async () => {
    let response = await Fetch.fetch(jsonUrl, {method: "GET"})
    let absent = Fetch.responseHeaders(response)->Fetch.getHeader("x-not-present")->Null.toOption
    expect(absent)->toEqual(None)
  })
})

describe("Fetch — requestInit & errors", () => {
  testAsync("fetch accepts a populated requestInit (method + headers)", async () => {
    let headers = Dict.fromArray([("x-request-id", "abc-123")])
    let response = await Fetch.fetch(textUrl, {method: "GET", headers})
    expect(Fetch.ok(response))->toBe(true)
    expect(await Fetch.text(response))->toBe("hello fetch")
  })

  testAsync("fetch rejects on an invalid URL — catchable as JsExn", async () => {
    switch await Fetch.fetch("http://", {method: ?None}) {
    | _response => expect("did not reject")->toBe("rejected")
    | exception JsExn(error) => expect(error->JsExn.message->Option.isSome)->toBe(true)
    }
  })
})

describe("Fetch — abort primitives (smoke)", () => {
  // We prove these values marshal into a real `fetch` call that resolves; the abort
  // *effect* is not asserted here (data: URLs resolve instantly / don't honor abort
  // in-process — a true abort assertion needs a slow request). See tests/README.md.
  testAsync("a requestInit can carry an AbortController signal", async () => {
    let controller = Fetch.makeAbortController()
    let response = await Fetch.fetch(textUrl, {signal: Fetch.signal(controller)})
    expect(Fetch.ok(response))->toBe(true)
    controller->Fetch.abort // callable; tears the (already-finished) request down
  })

  testAsync("a requestInit can carry an AbortSignal.timeout", async () => {
    let response = await Fetch.fetch(textUrl, {signal: Fetch.timeoutSignal(5000)})
    expect(Fetch.ok(response))->toBe(true)
  })
})
