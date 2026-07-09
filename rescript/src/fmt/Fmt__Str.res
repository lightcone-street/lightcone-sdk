// Fmt__Str — string display helpers. Reached as `Fmt.Str`.

// First and last `qty / 2` characters joined by an ellipsis — e.g.
// shorten("FRGkJho6fY7…nWcPR", ~qty=8) → "FRGk...WcPR". Strings of `qty`
// characters or fewer are returned unchanged.
let shorten = (value: string, ~qty: int): string => {
  let length = String.length(value)
  if length > qty {
    let charsToShow = qty / 2
    let head = value->String.slice(~start=0, ~end=charsToShow)
    let tail = value->String.slice(~start=length - charsToShow, ~end=length)
    `${head}...${tail}`
  } else {
    value
  }
}
