// Compile-guard for README.md — a compile-only file (no test blocks). If a README
// snippet drifts from the actual binding signature, `rescript build` fails here.

let intCompare = (left, right) => left - right

// Setup — trailing-() calling convention
let _makeComparator = (): SortedBtree.t<int, string> => SortedBtree.make(~compare=intCompare, ())
let _makeSeeded = (): SortedBtree.t<int, string> =>
  SortedBtree.make(~entries=[(1, "a")], ~compare=intCompare, ())
let _makeDefault = (): SortedBtree.t<int, string> => SortedBtree.make()

// How to read returned values — option<'a> (get / minKey / maxKey)
let _readOption = (book: SortedBtree.t<int, string>) => {
  let _viaSwitch = switch book->SortedBtree.get(2) {
  | Some(value) => value
  | None => "default"
  }
  let _viaGetOr: string = book->SortedBtree.get(2)->Option.getOr("default")
}

// How to read returned values — destructure toArray's (key, value) tuples
let _readArray = (book: SortedBtree.t<int, string>) =>
  book->SortedBtree.toArray->Array.forEach(((key, value)) => Console.log2(key, value))

// Quick start
let _quickStart = () => {
  let book = SortedBtree.make(~compare=intCompare, ())
  let _ = book->SortedBtree.set(3, "c")
  let _ = book->SortedBtree.set(1, "a")
  let _ = book->SortedBtree.set(2, "b")
  let _keys: array<int> = book->SortedBtree.keysArray
  let _two: option<string> = book->SortedBtree.get(2)
  let _absent: option<string> = book->SortedBtree.get(9)
  let _lowest: option<int> = book->SortedBtree.minKey
  let _highest: option<int> = book->SortedBtree.maxKey
  let _count: int = book->SortedBtree.size
}

// Reference — mutation
let _mutation = (book: SortedBtree.t<int, string>) => {
  let _set: bool = book->SortedBtree.set(1, "a")
  let _delete: bool = book->SortedBtree.delete(1)
  let _clear: unit = book->SortedBtree.clear
}

// Reference — lookup
let _lookup = (book: SortedBtree.t<int, string>) => {
  let _get: option<string> = book->SortedBtree.get(1)
  let _has: bool = book->SortedBtree.has(1)
  let _size: int = book->SortedBtree.size
  let _minKey: option<int> = book->SortedBtree.minKey
  let _maxKey: option<int> = book->SortedBtree.maxKey
}

// Reference — ordered snapshots
let _snapshots = (book: SortedBtree.t<int, string>) => {
  let _toArray: array<(int, string)> = book->SortedBtree.toArray
  let _keysArray: array<int> = book->SortedBtree.keysArray
  let _valuesArray: array<string> = book->SortedBtree.valuesArray
  let _forEachPair: int = book->SortedBtree.forEachPair((_key, _value) => ())
}
