// Notification wire decoding. The kind is adjacently tagged
// (`notification_type` + `data`) and flattened into the notification's own
// object, so the whole tree is hand-decoded. Optional payload fields are read
// tolerantly: an absent key OR an explicit JSON `null` both decode to `None`
// (spice's `?` field rejects an explicit null).

// ── Decode helpers ───────────────────────────────────────────────────────────
let field = (dict, key) => Dict.get(dict, key)

// Optional string: absent, null, or a non-string value all collapse to None.
let optString = (dict, key) =>
  switch Dict.get(dict, key) {
  | Some(JSON.String(value)) => Some(value)
  | _ => None
  }

let marketResolutionDecode = (json: JSON.t): result<
  Notification__Model.MarketResolution.t,
  Spice.decodeError,
> =>
  switch json {
  | JSON.Object(dict) =>
    switch (
      Dict.get(dict, "kind"),
      Dict.get(dict, "payout_denominator"),
      Dict.get(dict, "payouts"),
    ) {
    | (Some(kindJson), Some(JSON.Number(payoutDenominator)), Some(payoutsJson)) =>
      switch (
        Notification__Model.MarketResolution.Kind.t_decode(kindJson),
        Spice.arrayFromJson(Notification__Model.MarketResolution.payout_decode, payoutsJson),
      ) {
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

let marketResolvedDecode = (json: JSON.t): result<
  Notification__Model.MarketResolved.t,
  Spice.decodeError,
> =>
  switch json {
  | JSON.Object(dict) =>
    switch field(dict, "market_pubkey") {
    | Some(JSON.String(marketPubkey)) =>
      let resolution =
        field(dict, "resolution")->Option.flatMap(value =>
          marketResolutionDecode(value)->Result.mapOr(None, value => Some(value))
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

let orderFilledDecode = (json: JSON.t): result<
  Notification__Model.OrderFilled.t,
  Spice.decodeError,
> =>
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

let marketDataDecode = (json: JSON.t): result<Notification__Model.MarketData.t, Spice.decodeError> =>
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

// Adjacently-tagged + flattened: `notification_type` and `data` live in the same
// object as the notification's own fields.
let kindDecode = (
  dict: Dict.t<JSON.t>,
  json: JSON.t,
): result<Notification__Model.Kind.t, Spice.decodeError> =>
  switch field(dict, "notification_type") {
  | Some(JSON.String("market_resolved")) =>
    switch field(dict, "data") {
    | Some(data) => marketResolvedDecode(data)->Result.map(value => Notification__Model.Kind.MarketResolved(value))
    | None => Spice.error("market_resolved notification missing data", json)
    }
  | Some(JSON.String("order_filled")) =>
    switch field(dict, "data") {
    | Some(data) => orderFilledDecode(data)->Result.map(value => Notification__Model.Kind.OrderFilled(value))
    | None => Spice.error("order_filled notification missing data", json)
    }
  | Some(JSON.String("new_market")) =>
    switch field(dict, "data") {
    | Some(data) => marketDataDecode(data)->Result.map(value => Notification__Model.Kind.NewMarket(value))
    | None => Spice.error("new_market notification missing data", json)
    }
  | Some(JSON.String("rules_clarified")) =>
    switch field(dict, "data") {
    | Some(data) => marketDataDecode(data)->Result.map(value => Notification__Model.Kind.RulesClarified(value))
    | None => Spice.error("rules_clarified notification missing data", json)
    }
  | Some(JSON.String("global")) => Ok(Notification__Model.Kind.Global)
  | _ => Spice.error("unknown notification_type", json)
  }

// Decode one notification.
let decode = (json: JSON.t): result<Notification__Model.t, Spice.decodeError> =>
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
      switch kindDecode(dict, json) {
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

// Decode the list response `{ notifications: [...] }`.
let decodeResponse = (json: JSON.t): result<array<Notification__Model.t>, Spice.decodeError> =>
  switch json {
  | JSON.Object(dict) =>
    switch field(dict, "notifications") {
    | Some(notificationsJson) => Spice.arrayFromJson(decode, notificationsJson)
    | None => Spice.error("notifications response missing 'notifications'", json)
    }
  | _ => Spice.error("notifications response is not an object", json)
  }
