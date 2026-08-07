import JSONBigFactory from "json-bigint";

const exactJson = JSONBigFactory({
  useNativeBigInt: true,
  alwaysParseAsBig: false,
  protoAction: "error",
  constructorAction: "error",
});

/** Parse JSON without first rounding integer tokens through a JS number. */
export function parseJsonExact<T = unknown>(input: string): T {
  // json-bigint deliberately creates null-prototype objects. Restore ordinary
  // JSON.parse-compatible objects after its prototype-pollution checks have
  // rejected reserved keys, so this remains a drop-in transport parser.
  return normalizeParsedJson(exactJson.parse(input)) as T;
}

/** Serialize bigint values as ordinary JSON integer tokens, not strings. */
export function stringifyJsonExact(value: unknown): string {
  return exactJson.stringify(value);
}

function normalizeParsedJson(value: unknown): unknown {
  if (Array.isArray(value)) return value.map(normalizeParsedJson);
  if (value !== null && typeof value === "object") {
    const normalized: Record<string, unknown> = {};
    for (const [key, child] of Object.entries(value)) {
      normalized[key] = normalizeParsedJson(child);
    }
    return normalized;
  }
  return value;
}
