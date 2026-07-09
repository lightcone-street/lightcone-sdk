// Fmt__Str — string display helpers. Reached as `Fmt.Str`.

// First and last `qty / 2` characters joined by an ellipsis; strings of `qty`
// characters or fewer are returned unchanged.
let shorten: (string, ~qty: int) => string
