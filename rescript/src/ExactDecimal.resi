// Parse a non-negative decimal at an exact integer scale, without rounding.
let scaledInteger: (string, int) => result<bigint, string>
// Format a non-negative integer at a fixed decimal scale without precision loss.
let formatScaled: (bigint, int) => result<string, string>
