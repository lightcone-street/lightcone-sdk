// Plans contain the fee-estimated unsigned message and immutable projections.
// Submit once; retain projections until a full snapshot covers the confirmed slot.
module Kind = {
  @genType type t = Split | Merge | Redeem | NativeWithdraw | Wrap | UnwrapAll
}
module Costs = {
  @genType
  type t = {
    feeLamports: bigint,
    upfrontRentLamports: bigint,
    createsCanonicalWsolAccount: bool,
    sponsored: bool,
  }
}
module Availability = {
  @genType
  type t = {
    breakdown: Position__WalletState.solBalanceBreakdown,
    displayedLamports: bigint,
    reserveLamports: bigint,
    spendableLamports: bigint,
  }
}
module Delta = {
  @genType type t = {nativeLamports: bigint, canonicalWsolLamports: bigint}
}
module Plan = {
  @genType
  type t = {
    kind: Kind.t,
    transaction: V1Transaction.t,
    costs: Costs.t,
    availability: Availability.t,
    expectedDelta: Delta.t,
  }
}
module CanonicalAccount = {
  @genType
  type t = {accountLamports: bigint, tokenAmountLamports: bigint, nativeReserveLamports: bigint}
}
let isInteger: int => bool = %raw(`Number.isInteger`)
let maxU64 = 18446744073709551615n
let validAmount = amount => amount > 0n && amount <= maxU64
let fail = message => Error(SdkError.Validation(message))
let bind = async (promise, next) =>
  switch await promise {
  | Error(error) => Error(error)
  | Ok(value) => await next(value)
  }
let availability = (
  breakdown: Position__WalletState.solBalanceBreakdown,
  costs: Costs.t,
  ~unwrapAll=false,
): result<Availability.t, SdkError.t> => {
  let total = breakdown.nativeLamports + breakdown.canonicalWsolLamports
  let live = costs.feeLamports + costs.upfrontRentLamports
  let floor = costs.createsCanonicalWsolAccount ? 3500000n : 1000000n
  let reserve = costs.sponsored ? 0n : unwrapAll ? costs.feeLamports : live > floor ? live : floor
  if (
    breakdown.nativeLamports < 0n ||
    breakdown.canonicalWsolLamports < 0n ||
    costs.feeLamports < 0n ||
    costs.upfrontRentLamports < 0n ||
    total > maxU64 ||
    live > maxU64
  ) {
    fail("SOL balances or costs exceed u64")
  } else if (
    unwrapAll &&
    (costs.sponsored || costs.createsCanonicalWsolAccount || costs.upfrontRentLamports != 0n)
  ) {
    fail("unwrap-all requires unsponsored fee-only costs")
  } else if breakdown.nativeLamports < reserve {
    Error(
      SdkError.InsufficientSolForTransactionFees({
        availableLamports: breakdown.nativeLamports,
        requiredLamports: reserve,
      }),
    )
  } else {
    Ok({
      breakdown,
      displayedLamports: total,
      reserveLamports: reserve,
      spendableLamports: total > reserve ? total - reserve : 0n,
    })
  }
}
let wallet = (client: Client.t, state, ~nativeOnly=false) => {
  if client.sponsorshipEnabled {
    fail("sponsored SOL planning is unsupported")
  } else {
    switch client.credentials {
    | None => fail("authenticated wallet credentials are required")
    | Some(credentials) =>
      let native = switch client.signingStrategy {
      | Some(NativeSigner(_)) => true
      | _ => false
      }
      if (
        credentials.expiresAt <= Date.now() /. 1000. ||
        !SolanaKit.isAddress(credentials.walletAddress) ||
        credentials.walletAddress == "11111111111111111111111111111111" ||
        Position__WalletState.walletAddress(state) != Some(credentials.walletAddress) ||
        Position__WalletState.contextSlot(state)->Option.isNone
      ) {
        fail("unexpired credentials and complete matching wallet state are required")
      } else if (
        Client.signerAddress(client)->Option.map(SolanaKit.addressToString) !=
          Some(credentials.walletAddress) || (nativeOnly && !native)
      ) {
        fail(
          "planning signer must control the authenticated wallet; conversions require a native keypair",
        )
      } else {
        Ok(SolanaKit.address(credentials.walletAddress))
      }
    }
  }
}
// Primitive binary reads; policy checks remain typed ReScript.
let byte: (Uint8Array.t, int) => int = %raw(`(a,i) => a[i]`)
let size: Uint8Array.t => int = %raw(`a => a.length`)
let readU32: (
  Uint8Array.t,
  int,
) => float = %raw(`(a,i) => new DataView(a.buffer,a.byteOffset,a.byteLength).getUint32(i,true)`)
let readU64: (
  Uint8Array.t,
  int,
) => bigint = %raw(`(a,i) => new DataView(a.buffer,a.byteOffset,a.byteLength).getBigUint64(i,true)`)
let addressAt = (bytes, offset) =>
  SolanaKitCodec.decode(
    SolanaKitCodec.getAddressDecoder(),
    SolInstructions.slice(bytes, offset, offset + 32),
  )
let field = RpcJson.field
let accountValue = async (client, address) =>
  (
    await RpcJson.read(
      client,
      "getAccountInfo",
      [JSON.String(SolanaKit.addressToString(address)), RpcJson.accountOptions],
    )
  )->Result.flatMap(response =>
    switch response {
    | JSON.Object(fields) =>
      switch Dict.get(fields, "value") {
      | Some(value) => Ok(value)
      | None => fail("RPC omitted account value")
      }
    | _ => fail("malformed account response")
    }
  )
let canonicalAccount = async (client, wallet): result<option<CanonicalAccount.t>, SdkError.t> => {
  let account = await SolInstructions.canonical(wallet)
  (await accountValue(client, account))->Result.flatMap(value => {
    if value == JSON.Null {
      Ok(None)
    } else {
      let decode = () =>
        switch (RpcJson.u64(field(value, "lamports")), field(value, "data")) {
        | (Ok(accountLamports), JSON.Array([JSON.String(encoded), JSON.String("base64")])) =>
          let data = SolanaKitCodec.encode(SolanaKitCodec.getBase64Encoder(), encoded)
          if (
            field(value, "owner") !=
              JSON.String(SolanaKit.addressToString(Constants.tokenProgram)) || size(data) != 165
          ) {
            fail("invalid canonical WSOL account owner or layout")
          } else if (
            addressAt(data, 0) != SolInstructions.mint ||
            addressAt(data, 32) != wallet ||
            byte(data, 108) != 1 ||
            readU32(data, 109) != 1. ||
            readU32(data, 72) != 0. && readU32(data, 72) != 1. ||
            (readU32(data, 129) != 0. &&
              (readU32(data, 129) != 1. || addressAt(data, 133) != wallet))
          ) {
            fail(
              "incompatible canonical WSOL mint, authority, initialization, native reserve, or close authority",
            )
          } else {
            let tokenAmountLamports = readU64(data, 64)
            let nativeReserveLamports = readU64(data, 113)
            if (
              tokenAmountLamports + nativeReserveLamports > accountLamports ||
                tokenAmountLamports + nativeReserveLamports > maxU64
            ) {
              fail("canonical WSOL accounted lamports exceed live account balance")
            } else {
              Ok(
                Some({
                  CanonicalAccount.accountLamports,
                  tokenAmountLamports,
                  nativeReserveLamports,
                }),
              )
            }
          }
        | _ => fail("canonical WSOL account fields missing or malformed")
        }
      switch decode() {
      | result => result
      | exception _ => fail("canonical WSOL account data is malformed")
      }
    }
  })
}
let prepare = async (client, state, ~nativeOnly=false) =>
  switch (wallet(client, state, ~nativeOnly), Position__WalletState.solBalanceBreakdown(state)) {
  | (Error(error), _) | (_, Error(error)) => Error(error)
  | (Ok(wallet), Ok(breakdown)) =>
    (await canonicalAccount(client, wallet))->Result.flatMap(account =>
      if account->Option.isNone && breakdown.canonicalWsolLamports > 0n {
        fail("positive WSOL snapshot has no live canonical account")
      } else {
        Ok((wallet, breakdown, account))
      }
    )
  }
let finish = async (
  client,
  kind: Kind.t,
  transaction,
  breakdown,
  upfrontRentLamports,
  createsCanonicalWsolAccount,
  delta,
) => {
  (await Transactions.estimateFee(client, transaction))->Result.flatMap(feeLamports => {
    let costs: Costs.t = {
      feeLamports,
      upfrontRentLamports,
      createsCanonicalWsolAccount,
      sponsored: false,
    }
    availability(breakdown, costs, ~unwrapAll=kind == UnwrapAll)->Result.flatMap(availability =>
      delta(costs, availability)->Result.map(
        expectedDelta => {Plan.kind, transaction, costs, availability, expectedDelta},
      )
    )
  })
}
let rentFor = async (client, create) =>
  if create {
    await RpcJson.rent(client, 165)
  } else {
    Ok(0n)
  }
let planWrap = async (client, ~amount, ~state) => {
  if !validAmount(amount) {
    fail("wrap amount must be positive u64")
  } else {
    await bind(prepare(client, state, ~nativeOnly=true), async ((wallet, breakdown, account)) => {
      let synchronized =
        account->Option.mapOr(true, info =>
          info.tokenAmountLamports == breakdown.canonicalWsolLamports &&
          info.accountLamports == info.tokenAmountLamports + info.nativeReserveLamports &&
          info.accountLamports + amount <= maxU64 &&
          info.tokenAmountLamports + amount <= maxU64
        )
      if !synchronized {
        fail("wrap requires matching synchronized canonical WSOL without donated excess")
      } else {
        let create = account->Option.isNone
        await bind(rentFor(client, create), async rent => {
          let canonical = await SolInstructions.canonical(wallet)
          let instructions = create
            ? [SolInstructions.createAta(wallet, canonical, ~strict=true)]
            : []
          Array.push(instructions, SolInstructions.transfer(wallet, canonical, amount))
          Array.push(instructions, SolInstructions.sync(canonical))
          await bind(
            Transactions.build(client, ~payer=wallet, ~instructions),
            tx =>
              finish(
                client,
                Wrap,
                tx,
                breakdown,
                rent,
                create,
                (costs, availability) => {
                  if amount + availability.reserveLamports > breakdown.nativeLamports {
                    fail("native SOL cannot fund wrap and reserve")
                  } else {
                    Ok({
                      Delta.nativeLamports: 0n - amount - costs.feeLamports - rent,
                      canonicalWsolLamports: amount,
                    })
                  }
                },
              ),
          )
        })
      }
    })
  }
}
let planUnwrapAll = async (client, ~state) =>
  await bind(prepare(client, state, ~nativeOnly=true), async ((wallet, breakdown, account)) =>
    switch account {
    | Some(info)
      if breakdown.canonicalWsolLamports > 0n &&
        info.tokenAmountLamports == breakdown.canonicalWsolLamports =>
      let canonical = await SolInstructions.canonical(wallet)
      await bind(
        Transactions.build(
          client,
          ~payer=wallet,
          ~instructions=[SolInstructions.close(wallet, canonical)],
        ),
        tx =>
          finish(
            client,
            UnwrapAll,
            tx,
            breakdown,
            0n,
            false,
            (costs, _) => {
              if breakdown.nativeLamports - costs.feeLamports + info.accountLamports > maxU64 {
                fail("unwrap-all native balance would overflow u64")
              } else {
                Ok({
                  Delta.nativeLamports: info.accountLamports - costs.feeLamports,
                  canonicalWsolLamports: 0n - info.tokenAmountLamports,
                })
              }
            },
          ),
      )
    | _ => fail("unwrap-all requires a positive matching live canonical WSOL amount")
    }
  )
let planMarket = async (
  client,
  kind: Kind.t,
  ~market,
  ~numOutcomes,
  ~amount,
  ~outcomeIndex=0,
  ~state,
) => {
  if (
    !validAmount(amount) ||
    !isInteger(numOutcomes) ||
    !isInteger(outcomeIndex) ||
    numOutcomes < 2 ||
    numOutcomes > 6 ||
    outcomeIndex < 0 ||
    outcomeIndex >= numOutcomes
  ) {
    fail("invalid SOL market amount or outcome")
  } else {
    await bind(prepare(client, state), async ((wallet, breakdown, account)) => {
      let create = account->Option.isNone
      let shortfall =
        kind == Split && amount > breakdown.canonicalWsolLamports
          ? amount - breakdown.canonicalWsolLamports
          : 0n
      await bind(rentFor(client, create), async rent => {
        let canonical = await SolInstructions.canonical(wallet)
        let instructions = create ? [SolInstructions.createAta(wallet, canonical)] : []
        if shortfall > 0n {
          Array.push(instructions, SolInstructions.transfer(wallet, canonical, shortfall))
          Array.push(instructions, SolInstructions.sync(canonical))
        }
        let instruction = switch kind {
        | Split =>
          await Instructions.deposit(
            ~programId=client.Client.programId,
            ~user=wallet,
            ~market,
            ~mint=SolInstructions.mint,
            ~amount,
            ~numOutcomes,
          )
        | Merge =>
          await Instructions.mergeCompleteSet(
            ~programId=client.programId,
            ~user=wallet,
            ~market,
            ~mint=SolInstructions.mint,
            ~amount,
            ~numOutcomes,
          )
        | _ =>
          await Instructions.redeemWinnings(
            ~programId=client.programId,
            ~user=wallet,
            ~market,
            ~mint=SolInstructions.mint,
            ~amount,
            ~outcomeIndex,
          )
        }
        Array.push(instructions, instruction)
        await bind(
          Transactions.build(client, ~payer=wallet, ~instructions),
          tx =>
            finish(
              client,
              kind,
              tx,
              breakdown,
              rent,
              create,
              (costs, availability) => {
                if (
                  kind == Split &&
                    (amount > availability.spendableLamports ||
                      shortfall + availability.reserveLamports > breakdown.nativeLamports)
                ) {
                  fail("SOL cannot fund split and reserve")
                } else {
                  Ok({
                    Delta.nativeLamports: 0n - shortfall - costs.feeLamports - rent,
                    canonicalWsolLamports: kind == Split ? shortfall - amount : amount,
                  })
                }
              },
            ),
        )
      })
    })
  }
}
let planSplit = (client, ~market, ~numOutcomes, ~amount, ~state) =>
  planMarket(client, Split, ~market, ~numOutcomes, ~amount, ~state)
let planMerge = (client, ~market, ~numOutcomes, ~amount, ~state) =>
  planMarket(client, Merge, ~market, ~numOutcomes, ~amount, ~state)
let planRedeem = (client, ~market, ~numOutcomes, ~outcomeIndex, ~amount, ~state) =>
  planMarket(client, Redeem, ~market, ~numOutcomes, ~outcomeIndex, ~amount, ~state)
let planNativeWithdrawal = async (client, ~recipient, ~amount, ~state) => {
  if !validAmount(amount) || !SolanaKit.isAddress(SolanaKit.addressToString(recipient)) {
    fail("invalid withdrawal amount or recipient")
  } else {
    switch (wallet(client, state), Position__WalletState.solBalanceBreakdown(state)) {
    | (Error(error), _) | (_, Error(error)) => Error(error)
    | (Ok(wallet), Ok(breakdown)) =>
      await bind(
        Transactions.build(
          client,
          ~payer=wallet,
          ~instructions=[SolInstructions.transfer(wallet, recipient, amount)],
        ),
        async direct =>
          await bind(Transactions.estimateFee(client, direct), async fee => {
            let costs: Costs.t = {
              feeLamports: fee,
              upfrontRentLamports: 0n,
              createsCanonicalWsolAccount: false,
              sponsored: false,
            }
            switch availability(breakdown, costs) {
            | Error(error) => Error(error)
            | Ok(available) =>
              if (
                amount > available.spendableLamports || amount + available.reserveLamports > maxU64
              ) {
                fail("withdrawal exceeds spendable SOL")
              } else if breakdown.nativeLamports >= amount + available.reserveLamports {
                Ok({
                  Plan.kind: NativeWithdraw,
                  transaction: direct,
                  costs,
                  availability: available,
                  expectedDelta: {nativeLamports: 0n - amount - fee, canonicalWsolLamports: 0n},
                })
              } else {
                await bind(
                  canonicalAccount(client, wallet),
                  async account => {
                    if account->Option.isNone {
                      fail("canonical WSOL is required for withdrawal shortfall")
                    } else {
                      await bind(
                        RpcJson.rent(client, 165),
                        async rent =>
                          await bind(
                            Transactions.context(client),
                            async context => {
                              let rec select = async attempt =>
                                if attempt >= 8 {
                                  fail("temporary WSOL seed attempts exhausted")
                                } else {
                                  let seed = SolInstructions.nativeWithdrawSeed(
                                    ~blockhash=SolanaKit.address(context.blockhash),
                                    ~wallet,
                                    ~recipient,
                                    ~amount,
                                    ~attempt,
                                  )
                                  let temporary = SolInstructions.temporaryAccount(wallet, seed)
                                  await bind(
                                    accountValue(client, temporary),
                                    async value =>
                                      if value == JSON.Null {
                                        Ok((seed, temporary))
                                      } else {
                                        await select(attempt + 1)
                                      },
                                  )
                                }
                              await bind(
                                select(0),
                                async ((seed, temporary)) => {
                                  let canonical = await SolInstructions.canonical(wallet)
                                  let build = shortfall =>
                                    V1Transaction.compile(
                                      ~payer=wallet,
                                      ~context,
                                      ~instructions=SolInstructions.temporaryWithdraw(
                                        ~wallet,
                                        ~recipient,
                                        ~amount,
                                        ~canonical,
                                        ~temporary,
                                        ~seed,
                                        ~rent,
                                        ~shortfall,
                                      ),
                                    )
                                  let costsFor = (feeLamports): Costs.t => {
                                    feeLamports,
                                    upfrontRentLamports: rent,
                                    createsCanonicalWsolAccount: false,
                                    sponsored: false,
                                  }
                                  let needed = fee =>
                                    availability(breakdown, costsFor(fee))->Result.flatMap(
                                      available => {
                                        let total = amount + available.reserveLamports
                                        let shortfall = total - breakdown.nativeLamports
                                        if (
                                          total > maxU64 ||
                                          shortfall <= 0n ||
                                          shortfall > breakdown.canonicalWsolLamports
                                        ) {
                                          fail("canonical WSOL cannot fund withdrawal shortfall")
                                        } else {
                                          Ok((shortfall, available))
                                        }
                                      },
                                    )
                                  await bind(
                                    Promise.resolve(build(1n)),
                                    async initial =>
                                      await bind(
                                        Transactions.estimateFee(client, initial),
                                        async firstFee => {
                                          switch needed(firstFee) {
                                          | Error(error) => Error(error)
                                          | Ok((firstNeeded, _)) =>
                                            await bind(
                                              Promise.resolve(build(firstNeeded)),
                                              async rebuilt =>
                                                await bind(
                                                  Transactions.estimateFee(client, rebuilt),
                                                  async finalFee => {
                                                    switch needed(finalFee) {
                                                    | Error(error) => Error(error)
                                                    | Ok((shortfall, available)) =>
                                                      let finish = transaction => Ok({
                                                        Plan.kind: NativeWithdraw,
                                                        transaction,
                                                        costs: costsFor(finalFee),
                                                        availability: available,
                                                        expectedDelta: {
                                                          nativeLamports: shortfall -
                                                          amount -
                                                          finalFee,
                                                          canonicalWsolLamports: 0n - shortfall,
                                                        },
                                                      })
                                                      if shortfall == firstNeeded {
                                                        finish(rebuilt)
                                                      } else {
                                                        await bind(
                                                          Promise.resolve(build(shortfall)),
                                                          async stable =>
                                                            await bind(
                                                              Transactions.estimateFee(
                                                                client,
                                                                stable,
                                                              ),
                                                              async stableFee =>
                                                                if stableFee != finalFee {
                                                                  fail(
                                                                    "transaction fee changed while rebuilding withdrawal",
                                                                  )
                                                                } else {
                                                                  finish(stable)
                                                                },
                                                            ),
                                                        )
                                                      }
                                                    }
                                                  },
                                                ),
                                            )
                                          }
                                        },
                                      ),
                                  )
                                },
                              )
                            },
                          ),
                      )
                    }
                  },
                )
              }
            }
          }),
      )
    }
  }
}
let submit = (client, plan: Plan.t) => Transactions.signAndSubmitConfirmed(client, plan.transaction)
