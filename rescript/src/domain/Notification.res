// Notification domain — user notifications for market / order events. Mirrors the
// Rust `domain/notification`.
//
// `NotificationKind` is an adjacently-tagged serde enum (`tag = "notification_type",
// content = "data"`) that is `#[serde(flatten)]`-ed into `Notification`, so spice
// can't derive it — the whole tree is hand-decoded here (see Auth.res and
// `SdkError.decodeRejectedDetails` for the pattern). Optional payload fields are
// read tolerantly: an absent key OR an explicit JSON `null` both decode to `None`,
// matching serde's `Option` handling (spice's `?` field rejects an explicit null).

// ── Decode helpers ───────────────────────────────────────────────────────────
let field = (dict, key) => Dict.get(dict, key)

// Optional string: absent, null, or a non-string value all collapse to None.
let optString = (dict, key) =>
  switch Dict.get(dict, key) {
  | Some(JSON.String(value)) => Some(value)
  | _ => None
  }

// ── Market resolution ────────────────────────────────────────────────────────
// `MarketResolvedData` references the market domain's `MarketResolutionResponse`.
// The market domain is not yet ported to ReScript, so it is defined locally here;
// fold this into the shared market module once that lands.
module MarketResolution = {
  module Kind = {
    @spice
    type t =
      | @as("single_winner") @spice.as("single_winner") SingleWinner
      | @as("scalar") @spice.as("scalar") Scalar
  }

  @spice
  type payout = {
    @spice.key("outcome_index") outcomeIndex: float,
    @spice.key("payout_numerator") payoutNumerator: float,
  }

  type t = {
    kind: Kind.t,
    payoutDenominator: float,
    payouts: array<payout>,
    // Absent for scalar markets (no single winner); the wire sends `null`.
    singleWinningOutcome?: float,
  }

  let decode = (json: JSON.t): result<t, Spice.decodeError> =>
    switch json {
    | JSON.Object(dict) =>
      switch (
        Dict.get(dict, "kind"),
        Dict.get(dict, "payout_denominator"),
        Dict.get(dict, "payouts"),
      ) {
      | (Some(kindJson), Some(JSON.Number(payoutDenominator)), Some(payoutsJson)) =>
        switch (Kind.t_decode(kindJson), Spice.arrayFromJson(payout_decode, payoutsJson)) {
        | (Ok(kind), Ok(payouts)) =>
          let singleWinningOutcome = switch Dict.get(dict, "single_winning_outcome") {
          | Some(JSON.Number(value)) => Some(value)
          | _ => None
          }
          Ok({kind, payoutDenominator, payouts, singleWinningOutcome: ?singleWinningOutcome})
        | (Error(error), _) | (_, Error(error)) => Error(error)
        }
      | _ => Spice.error("market resolution missing kind/payout_denominator/payouts", json)
      }
    | _ => Spice.error("market resolution is not an object", json)
    }
}

// ── Notification payloads (one per NotificationKind variant) ─────────────────
type marketResolvedData = {
  marketPubkey: Shared.pubkeyStr,
  marketSlug?: string,
  marketName?: string,
  resolution?: MarketResolution.t,
}

type orderFilledData = {
  orderHash: string,
  marketPubkey: Shared.pubkeyStr,
  // Plain wire string (the Rust keeps `side` as a `String`, not the `Side` enum).
  side: string,
  // Decimal wire strings (no precision loss).
  price: string,
  filled: string,
  remaining: string,
  marketSlug?: string,
  marketName?: string,
  outcomeName?: string,
  outcomeNameLong?: string,
  outcomeIconUrlLow?: string,
  outcomeIconUrlMedium?: string,
  outcomeIconUrlHigh?: string,
}

type marketData = {
  marketPubkey: Shared.pubkeyStr,
  marketSlug?: string,
  marketName?: string,
}

let marketResolvedDataDecode = (json: JSON.t): result<marketResolvedData, Spice.decodeError> =>
  switch json {
  | JSON.Object(dict) =>
    switch field(dict, "market_pubkey") {
    | Some(JSON.String(marketPubkey)) =>
      let resolution =
        field(dict, "resolution")->Option.flatMap(value =>
          MarketResolution.decode(value)->Result.mapOr(None, value => Some(value))
        )
      Ok({
        marketPubkey,
        marketSlug: ?optString(dict, "market_slug"),
        marketName: ?optString(dict, "market_name"),
        resolution: ?resolution,
      })
    | _ => Spice.error("market_resolved data missing market_pubkey", json)
    }
  | _ => Spice.error("market_resolved data is not an object", json)
  }

let orderFilledDataDecode = (json: JSON.t): result<orderFilledData, Spice.decodeError> =>
  switch json {
  | JSON.Object(dict) =>
    switch (
      field(dict, "order_hash"),
      field(dict, "market_pubkey"),
      field(dict, "side"),
      field(dict, "price"),
      field(dict, "filled"),
      field(dict, "remaining"),
    ) {
    | (
        Some(JSON.String(orderHash)),
        Some(JSON.String(marketPubkey)),
        Some(JSON.String(side)),
        Some(JSON.String(price)),
        Some(JSON.String(filled)),
        Some(JSON.String(remaining)),
      ) =>
      Ok({
        orderHash,
        marketPubkey,
        side,
        price,
        filled,
        remaining,
        marketSlug: ?optString(dict, "market_slug"),
        marketName: ?optString(dict, "market_name"),
        outcomeName: ?optString(dict, "outcome_name"),
        outcomeNameLong: ?optString(dict, "outcome_name_long"),
        outcomeIconUrlLow: ?optString(dict, "outcome_icon_url_low"),
        outcomeIconUrlMedium: ?optString(dict, "outcome_icon_url_medium"),
        outcomeIconUrlHigh: ?optString(dict, "outcome_icon_url_high"),
      })
    | _ => Spice.error("order_filled data missing required fields", json)
    }
  | _ => Spice.error("order_filled data is not an object", json)
  }

let marketDataDecode = (json: JSON.t): result<marketData, Spice.decodeError> =>
  switch json {
  | JSON.Object(dict) =>
    switch field(dict, "market_pubkey") {
    | Some(JSON.String(marketPubkey)) =>
      Ok({
        marketPubkey,
        marketSlug: ?optString(dict, "market_slug"),
        marketName: ?optString(dict, "market_name"),
      })
    | _ => Spice.error("market data missing market_pubkey", json)
    }
  | _ => Spice.error("market data is not an object", json)
  }

// ── Notification kind + notification ─────────────────────────────────────────
type notificationKind =
  | MarketResolved(marketResolvedData)
  | OrderFilled(orderFilledData)
  | NewMarket(marketData)
  | RulesClarified(marketData)
  | Global

type notification = {
  id: string,
  kind: notificationKind,
  title: string,
  message: string,
  // ISO-8601 timestamps, kept as strings (matching the Rust `String` fields).
  expiresAt?: string,
  createdAt: string,
}

// Adjacently-tagged + flattened: `notification_type` and `data` live in the same
// object as the notification's own fields.
let notificationKindDecode = (
  dict: Dict.t<JSON.t>,
  json: JSON.t,
): result<notificationKind, Spice.decodeError> =>
  switch field(dict, "notification_type") {
  | Some(JSON.String("market_resolved")) =>
    switch field(dict, "data") {
    | Some(data) => marketResolvedDataDecode(data)->Result.map(value => MarketResolved(value))
    | None => Spice.error("market_resolved notification missing data", json)
    }
  | Some(JSON.String("order_filled")) =>
    switch field(dict, "data") {
    | Some(data) => orderFilledDataDecode(data)->Result.map(value => OrderFilled(value))
    | None => Spice.error("order_filled notification missing data", json)
    }
  | Some(JSON.String("new_market")) =>
    switch field(dict, "data") {
    | Some(data) => marketDataDecode(data)->Result.map(value => NewMarket(value))
    | None => Spice.error("new_market notification missing data", json)
    }
  | Some(JSON.String("rules_clarified")) =>
    switch field(dict, "data") {
    | Some(data) => marketDataDecode(data)->Result.map(value => RulesClarified(value))
    | None => Spice.error("rules_clarified notification missing data", json)
    }
  | Some(JSON.String("global")) => Ok(Global)
  | _ => Spice.error("unknown notification_type", json)
  }

let notificationDecode = (json: JSON.t): result<notification, Spice.decodeError> =>
  switch json {
  | JSON.Object(dict) =>
    switch (
      field(dict, "id"),
      field(dict, "title"),
      field(dict, "message"),
      field(dict, "created_at"),
    ) {
    | (
        Some(JSON.String(id)),
        Some(JSON.String(title)),
        Some(JSON.String(message)),
        Some(JSON.String(createdAt)),
      ) =>
      switch notificationKindDecode(dict, json) {
      | Ok(kind) =>
        Ok({
          id,
          kind,
          title,
          message,
          expiresAt: ?optString(dict, "expires_at"),
          createdAt,
        })
      | Error(error) => Error(error)
      }
    | _ => Spice.error("notification missing id/title/message/created_at", json)
    }
  | _ => Spice.error("notification is not an object", json)
  }

// NotificationsResponse { notifications: Vec<Notification> }
let notificationsResponseDecode = (json: JSON.t): result<array<notification>, Spice.decodeError> =>
  switch json {
  | JSON.Object(dict) =>
    switch field(dict, "notifications") {
    | Some(notificationsJson) => Spice.arrayFromJson(notificationDecode, notificationsJson)
    | None => Spice.error("notifications response missing 'notifications'", json)
    }
  | _ => Spice.error("notifications response is not an object", json)
  }

// ── Helpers (mirror the Rust `Notification` methods) ─────────────────────────
let isGlobal = (notification: notification): bool =>
  switch notification.kind {
  | Global => true
  | _ => false
  }

// The market slug associated with this notification, if any.
let marketSlug = (notification: notification): option<string> =>
  switch notification.kind {
  | MarketResolved(data) => data.marketSlug
  | OrderFilled(data) => data.marketSlug
  | NewMarket(data) | RulesClarified(data) => data.marketSlug
  | Global => None
  }

// ── Client functions ─────────────────────────────────────────────────────────
// Fetch all notifications for the authenticated user.
let fetch = async (
  client: Client.t,
  ~cookieHeader: option<string>=?,
): result<array<notification>, SdkError.t> =>
  await Http.get(
    client.http,
    ~path="/api/notifications",
    ~cookieHeader?,
    ~decode=notificationsResponseDecode,
  )

// Dismiss a single notification by id.
let dismiss = async (client: Client.t, ~notificationId: string): result<unit, SdkError.t> => {
  let body = JSON.Object(Dict.fromArray([("notification_id", JSON.String(notificationId))]))
  await Http.post(client.http, ~path="/api/notifications/dismiss", ~body, ~decode=_ => Ok())
}
