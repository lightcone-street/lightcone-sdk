open RescriptBun.Test
open RescriptBun.Test.Expect

// Runtime tests for the sorted-btree binding — run the compiled .res.mjs under Bun to prove
// the JS method names (`set`/`get`/`minKey`/`keysArray`/...), arg order, the undefined->option
// mapping, and ascending-by-key ordering. sorted-btree is a pure utility: no driver, just
// build trees and assert.

// `int` comparator must return an int (ReScript's Int.compare returns Ordering.t = float).
let intCompare = (left, right) => left - right
let emptyTree = () => SortedBtree.make(~compare=intCompare, ())

// Fresh seeded tree per test (the tree is mutable). Keys inserted out of order on purpose.
let seededTree = () => {
  let tree = emptyTree()
  let _ = tree->SortedBtree.set(3, "c")
  let _ = tree->SortedBtree.set(1, "a")
  let _ = tree->SortedBtree.set(2, "b")
  tree
}

describe("SortedBtree — construction", () => {
  test("make with ~entries seeds and sorts the tree", () => {
    let tree = SortedBtree.make(~entries=[(2, "b"), (1, "a"), (3, "c")], ~compare=intCompare, ())
    expect(tree->SortedBtree.keysArray)->toEqual([1, 2, 3])
    expect(tree->SortedBtree.get(1)->Option.getOr("?"))->toBe("a")
  })
})

describe("SortedBtree — mutation", () => {
  test("set returns true for a new key", () =>
    expect(emptyTree()->SortedBtree.set(5, "e"))->toBe(true)
  )
  test("set returns false when overwriting an existing key", () => {
    let tree = seededTree()
    expect(tree->SortedBtree.set(1, "A"))->toBe(false)
    expect(tree->SortedBtree.get(1)->Option.getOr("?"))->toBe("A")
  })
  test("delete returns true and shrinks the tree", () => {
    let tree = seededTree()
    expect(tree->SortedBtree.delete(2))->toBe(true)
    expect(tree->SortedBtree.size)->toBe(2)
    expect(tree->SortedBtree.has(2))->toBe(false)
  })
  test("delete returns false for an absent key", () =>
    expect(seededTree()->SortedBtree.delete(99))->toBe(false)
  )
  test("clear empties the tree", () => {
    let tree = seededTree()
    tree->SortedBtree.clear
    expect(tree->SortedBtree.size)->toBe(0)
    expect(tree->SortedBtree.minKey->Option.isNone)->toBe(true)
  })
})

describe("SortedBtree — lookup", () => {
  test("get returns Some for a present key", () =>
    expect(seededTree()->SortedBtree.get(2)->Option.getOr("?"))->toBe("b")
  )
  test("get returns None for an absent key", () =>
    expect(seededTree()->SortedBtree.get(9)->Option.isNone)->toBe(true)
  )
  test("has reflects membership", () => {
    let tree = seededTree()
    expect(tree->SortedBtree.has(1))->toBe(true)
    expect(tree->SortedBtree.has(9))->toBe(false)
  })
  test("size counts the entries", () => expect(seededTree()->SortedBtree.size)->toBe(3))
  test("minKey returns Some(smallest), None when empty", () => {
    expect(seededTree()->SortedBtree.minKey->Option.getOr(-1))->toBe(1)
    expect(emptyTree()->SortedBtree.minKey->Option.isNone)->toBe(true)
  })
  test("maxKey returns Some(largest), None when empty", () => {
    expect(seededTree()->SortedBtree.maxKey->Option.getOr(-1))->toBe(3)
    expect(emptyTree()->SortedBtree.maxKey->Option.isNone)->toBe(true)
  })
})

describe("SortedBtree — ordered snapshots", () => {
  test("keysArray is ascending regardless of insert order", () =>
    expect(seededTree()->SortedBtree.keysArray)->toEqual([1, 2, 3])
  )
  test("valuesArray follows key order", () =>
    expect(seededTree()->SortedBtree.valuesArray)->toEqual(["a", "b", "c"])
  )
  test("toArray returns (key, value) tuples in key order", () =>
    expect(seededTree()->SortedBtree.toArray)->toEqual([(1, "a"), (2, "b"), (3, "c")])
  )
  test("forEachPair visits pairs in ascending key order and returns the count", () => {
    let tree = seededTree()
    let seenKeys = []
    let visited = tree->SortedBtree.forEachPair((key, _value) => seenKeys->Array.push(key))
    expect(seenKeys)->toEqual([1, 2, 3])
    expect(visited)->toBe(3)
  })
})
