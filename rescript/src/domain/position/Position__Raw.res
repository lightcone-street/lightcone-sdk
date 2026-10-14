// Position wire types — the exact JSON the backend sends on the position read
// endpoints (`Position__Client` returns these `*Response` shapes directly) plus
// the deposit-token balance rows.
//
// Wire conventions: Decimal balances stay strings (no precision loss,
// gentype-clean); ids / counts / token decimals are floats (JS numbers); the
// position-entry timestamps ship as ISO-8601 strings.

// ── Response rows ─────────────────────────────────────────────────────────────
module OutcomeBalance = {
  @spice
  type t = {
    @spice.key("outcome_index") outcomeIndex: float,
    @spice.key("conditional_token") conditionalToken: Shared.pubkeyStr,
    balance: string,
    @spice.key("balance_idle") balanceIdle: string,
    @spice.key("balance_on_book") balanceOnBook: string,
  }
}

module VaultBalance = {
  @spice
  type t = {
    @spice.key("deposit_mint") depositMint: Shared.pubkeyStr,
    vault: Shared.pubkeyStr,
    balance: string,
  }
}

module GlobalDeposit = {
  @spice
  type t = {
    @spice.key("deposit_mint") depositMint: Shared.pubkeyStr,
    symbol: string,
    balance: string,
  }
}

// A user's position row in a single market.
module Entry = {
  @spice
  type t = {
    id: float,
    @spice.key("position_pubkey") positionPubkey: Shared.pubkeyStr,
    owner: Shared.pubkeyStr,
    @spice.key("market_pubkey") marketPubkey: Shared.pubkeyStr,
    outcomes: array<OutcomeBalance.t>,
    @spice.default([]) @spice.key("vault_balances") vaultBalances: array<VaultBalance.t>,
    // ISO-8601 timestamps (the wire ships these as strings).
    @spice.key("created_at") createdAt: string,
    @spice.key("updated_at") updatedAt: string,
  }
}

// ── Responses ─────────────────────────────────────────────────────────────────
// Response for `GET /api/users/{user_pubkey}/positions`.
module PositionsResponse = {
  @spice
  type t = {
    owner: Shared.pubkeyStr,
    @spice.key("total_markets") totalMarkets: float,
    positions: array<Entry.t>,
    @spice.default([]) @spice.key("global_deposits") globalDeposits: array<GlobalDeposit.t>,
    // Mint pubkey → token decimals.
    decimals: dict<float>,
  }
}

// Response for `GET /api/users/{user_pubkey}/markets/{market_pubkey}/positions`.
module MarketPositionsResponse = {
  @spice
  type t = {
    owner: Shared.pubkeyStr,
    @spice.key("market_pubkey") marketPubkey: Shared.pubkeyStr,
    positions: array<Entry.t>,
    @spice.default([]) @spice.key("global_deposits") globalDeposits: array<GlobalDeposit.t>,
    decimals: dict<float>,
  }
}

// Combined balance + metadata for a deposit token. Both the wire row AND the
// domain shape — `Position__Client.depositTokenBalances` returns these directly
// (keyed by mint).
module DepositTokenBalance = {
  @spice
  type t = {
    mint: Shared.pubkeyStr,
    idle: string,
    symbol: string,
    name: string,
    @spice.key("icon_url_low") iconUrlLow?: string,
    @spice.key("icon_url_medium") iconUrlMedium?: string,
    @spice.key("icon_url_high") iconUrlHigh?: string,
  }

  // Deposit-token row → token balance: nothing rests on the book (on_book = 0),
  // classified as a deposit asset.
  let toTokenBalance = (value: t): Position__Model.TokenBalance.t => {
    mint: value.mint,
    idle: value.idle,
    onBook: "0",
    tokenType: Position__Model.TokenBalance.DepositAsset,
  }
  let t_decode = json => t_decode(SdkError.stripNulls(json))
}

module NativeSolBalance = {
  @genType type t = string
  let t_decode = json =>
    switch json {
    | JSON.String(value) if RegExp.test(/^(0|[1-9][0-9]*)\.[0-9]{9}$/, value) => Ok(value)
    | _ =>
      Spice.error(
        "native SOL must be canonical non-negative text with nine fractional digits",
        json,
      )
    }
  let t_encode = value => JSON.String(value)
  let t_encodeJson = t_encode
}

module DepositTokenBalancesSnapshot = {
  @genType @spice
  type t = {
    @spice.key("context_slot") contextSlot: ProtocolInt.U64.t,
    balances: dict<DepositTokenBalance.t>,
    @spice.key("native_sol_balance") nativeSolBalance: NativeSolBalance.t,
  }
}
module WalletDepositBalanceStatus = {
  @genType @spice
  type t =
    | @as("reconnecting") @spice.as("reconnecting") Reconnecting
    | @as("metadata_unavailable") @spice.as("metadata_unavailable") MetadataUnavailable
}
module WalletDepositBalancesEvent = {
  @genType @spice
  type snapshot = {
    @spice.key("wallet_address") walletAddress: string,
    @spice.key("context_slot") contextSlot: ProtocolInt.U64.t,
    balances: dict<DepositTokenBalance.t>,
    @spice.key("native_sol_balance") nativeSolBalance: NativeSolBalance.t,
  }
  @genType @spice
  type balanceUpdate = {
    @spice.key("wallet_address") walletAddress: string,
    @spice.key("context_slot") contextSlot: ProtocolInt.U64.t,
    balance: DepositTokenBalance.t,
  }
  @genType @spice
  type nativeUpdate = {
    @spice.key("wallet_address") walletAddress: string,
    @spice.key("context_slot") contextSlot: ProtocolInt.U64.t,
    @spice.key("native_sol_balance") nativeSolBalance: NativeSolBalance.t,
  }
  @genType @spice
  type status = {
    @spice.key("wallet_address") walletAddress: string,
    status: WalletDepositBalanceStatus.t,
    code: string,
  }
  @genType
  type t =
    | Snapshot(snapshot)
    | BalanceUpdate(balanceUpdate)
    | NativeSolBalanceUpdate(nativeUpdate)
    | Status(status)
  let t_decode = json =>
    switch json {
    | JSON.Object(dict) =>
      switch Dict.get(dict, "event_type") {
      | Some(JSON.String("wallet_deposit_balance_snapshot")) =>
        snapshot_decode(json)->Result.map(value => Snapshot(value))
      | Some(JSON.String("wallet_deposit_balance_update")) =>
        balanceUpdate_decode(json)->Result.map(value => BalanceUpdate(value))
      | Some(JSON.String("wallet_native_sol_balance_update")) =>
        nativeUpdate_decode(json)->Result.map(value => NativeSolBalanceUpdate(value))
      | Some(JSON.String("wallet_deposit_balance_status")) =>
        status_decode(json)->Result.map(value => Status(value))
      | _ => Spice.error("unknown wallet balance event", json)
      }
    | _ => Spice.error("invalid wallet balance event", json)
    }
}
