open RescriptBun.Test
open RescriptBun.Test.Expect

// State machine — mirrors the unit tests in rust/src/rpc_failover.rs.
describe("RpcFailover state machine", () => {
  test("default state is primary with no flip timestamp", () => {
    let state = RpcFailover.make()
    expect(RpcFailover.active(state)->RpcFailover.toString)->toBe("primary")
    expect(state.flippedToBackupAtMs->Option.isNone)->toBe(true)
  })

  test("flipTo Backup records a timestamp", () => {
    let state = RpcFailover.make()
    RpcFailover.flipTo(state, Backup)
    expect(RpcFailover.active(state)->RpcFailover.toString)->toBe("backup")
    expect(state.flippedToBackupAtMs->Option.isSome)->toBe(true)
  })

  test("flipTo Primary clears the timestamp", () => {
    let state = RpcFailover.make()
    RpcFailover.flipTo(state, Backup)
    RpcFailover.flipTo(state, Primary)
    expect(RpcFailover.active(state)->RpcFailover.toString)->toBe("primary")
    expect(state.flippedToBackupAtMs->Option.isNone)->toBe(true)
  })

  test("no recovery to primary before the cooldown elapses", () => {
    let state = RpcFailover.make()
    RpcFailover.flipTo(state, Backup)
    RpcFailover.maybeRecoverToPrimary(state)
    expect(RpcFailover.active(state)->RpcFailover.toString)->toBe("backup")
  })

  test("maybeRecover is a no-op when never flipped", () => {
    let state = RpcFailover.make()
    RpcFailover.maybeRecoverToPrimary(state)
    expect(RpcFailover.active(state)->RpcFailover.toString)->toBe("primary")
  })

  test("other flips the endpoint", () => {
    expect(RpcFailover.other(Primary)->RpcFailover.toString)->toBe("backup")
    expect(RpcFailover.other(Backup)->RpcFailover.toString)->toBe("primary")
  })
})

// Executor — exercises the full retry/failover dance in-process (no network).
describe("RpcFailover.withFailover", () => {
  testAsync("dead primary + live backup → Ok, and flips state to backup", async () => {
    let state = RpcFailover.make()
    let tryOn = async (target: RpcFailover.Active.t): result<string, SdkError.t> =>
      switch target {
      | Primary => Error(SdkError.Other("primary down"))
      | Backup => Ok("blockhash-from-backup")
      }
    switch await RpcFailover.withFailover(state, ~hasBackup=true, ~tryOn) {
    | Ok(value) => expect(value)->toBe("blockhash-from-backup")
    | Error(_) => expect("unexpected error")->toBe("ok")
    }
    expect(RpcFailover.active(state)->RpcFailover.toString)->toBe("backup")
  })

  testAsync("live primary → Ok without flipping", async () => {
    let state = RpcFailover.make()
    let tryOn = async (_target: RpcFailover.Active.t): result<string, SdkError.t> =>
      Ok("blockhash-from-primary")
    switch await RpcFailover.withFailover(state, ~hasBackup=true, ~tryOn) {
    | Ok(value) => expect(value)->toBe("blockhash-from-primary")
    | Error(_) => expect("unexpected error")->toBe("ok")
    }
    expect(RpcFailover.active(state)->RpcFailover.toString)->toBe("primary")
  })

  testAsync("no backup → surfaces the error and stays on primary", async () => {
    let state = RpcFailover.make()
    let tryOn = async (_target: RpcFailover.Active.t): result<string, SdkError.t> =>
      Error(SdkError.Other("down"))
    let result = await RpcFailover.withFailover(state, ~hasBackup=false, ~tryOn)
    let isError = switch result {
    | Ok(_) => false
    | Error(_) => true
    }
    expect(isError)->toBe(true)
    expect(RpcFailover.active(state)->RpcFailover.toString)->toBe("primary")
  })
})
