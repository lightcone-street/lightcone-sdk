//! Custom serde helpers for backend wire formats.

use rust_decimal::Decimal;
use serde::{Deserialize, Deserializer};

/// Deserializes an explicitly present decimal string or `null`.
///
/// Pair with `#[serde(default)]` when an omitted field should normalize to
/// `None`; without it, an omitted field remains invalid.
pub fn deserialize_required_nullable_decimal<'de, D>(
    deserializer: D,
) -> Result<Option<Decimal>, D::Error>
where
    D: Deserializer<'de>,
{
    Option::<Decimal>::deserialize(deserializer)
}

/// Deserializes a Unix-millis `u64` into `DateTime<Utc>`.
///
/// The backend's WebSocket sends `created_at` as epoch milliseconds (i64/u64),
/// not ISO 8601 strings.
pub mod timestamp_ms {
    use chrono::{DateTime, Utc};
    use serde::{Deserialize, Deserializer, Serializer};

    pub fn serialize<S>(dt: &DateTime<Utc>, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_u64(dt.timestamp_millis() as u64)
    }

    pub fn deserialize<'de, D>(deserializer: D) -> Result<DateTime<Utc>, D::Error>
    where
        D: Deserializer<'de>,
    {
        let millis = u64::deserialize(deserializer)?;
        DateTime::<Utc>::from_timestamp_millis(millis as i64)
            .ok_or_else(|| serde::de::Error::custom(format!("Invalid timestamp: {}", millis)))
    }
}

/// Serializes/deserializes `TimeInForce` as a numeric u32.
///
/// The backend sends TIF as a number in trigger order responses:
/// 0 = GTC, 1 = IOC, 2 = FOK, 3 = ALO.
pub mod tif_numeric {
    use crate::shared::TimeInForce;
    use serde::{Deserialize, Deserializer, Serializer};

    pub fn serialize<S>(tif: &TimeInForce, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let n: u32 = match tif {
            TimeInForce::Gtc => 0,
            TimeInForce::Ioc => 1,
            TimeInForce::Fok => 2,
            TimeInForce::Alo => 3,
        };
        serializer.serialize_u32(n)
    }

    pub fn deserialize<'de, D>(deserializer: D) -> Result<TimeInForce, D::Error>
    where
        D: Deserializer<'de>,
    {
        let n = u32::deserialize(deserializer)?;
        match n {
            0 => Ok(TimeInForce::Gtc),
            1 => Ok(TimeInForce::Ioc),
            2 => Ok(TimeInForce::Fok),
            3 => Ok(TimeInForce::Alo),
            _ => Err(serde::de::Error::custom(format!("unknown tif value: {n}"))),
        }
    }
}

/// Serializes/deserializes `Option<TimeInForce>` as a numeric u32.
///
/// `None` serializes as absent (via `skip_serializing_if`).
/// On deserialize, reads a u32 and maps it to `Some(TimeInForce)`.
/// Pair with `#[serde(default)]` so absent fields deserialize as `None`.
pub mod tif_numeric_opt {
    use crate::shared::TimeInForce;
    use serde::{Deserializer, Serializer};

    pub fn serialize<S>(value: &Option<TimeInForce>, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        match value {
            Some(tif) => super::tif_numeric::serialize(tif, serializer),
            None => serializer.serialize_none(),
        }
    }

    pub fn deserialize<'de, D>(deserializer: D) -> Result<Option<TimeInForce>, D::Error>
    where
        D: Deserializer<'de>,
    {
        super::tif_numeric::deserialize(deserializer).map(Some)
    }
}

/// Deserializes an empty string as `None`, non-empty string as `Some(T)`.
pub mod empty_string_as_none {
    use serde::{Deserialize, Deserializer, Serialize, Serializer};

    pub fn serialize<T, S>(value: &Option<T>, serializer: S) -> Result<S::Ok, S::Error>
    where
        T: Serialize,
        S: Serializer,
    {
        match value {
            Some(v) => v.serialize(serializer),
            None => serializer.serialize_str(""),
        }
    }

    pub fn deserialize<'de, T, D>(deserializer: D) -> Result<Option<T>, D::Error>
    where
        T: Deserialize<'de>,
        D: Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        if s.is_empty() {
            Ok(None)
        } else {
            T::deserialize(serde::de::value::StrDeserializer::<serde::de::value::Error>::new(&s))
                .map(Some)
                .map_err(serde::de::Error::custom)
        }
    }
}

// ─── Committed trading-state encodings ──────────────────────────────────────
//
// The committed trading backend encodes exact integers as JSON strings on
// REST and in WebSocket snapshots, but as JSON numbers in live WebSocket
// facts. Every helper below accepts both encodings and buffers through an
// untagged enum, which keeps them working inside internally tagged enums and
// `#[serde(flatten)]` under serde_json's `arbitrary_precision` feature.

#[derive(Deserialize)]
#[serde(untagged)]
enum IntegerRepr {
    Text(String),
    Unsigned(u64),
    Signed(i64),
}

impl IntegerRepr {
    fn parse<T, E>(self) -> Result<T, E>
    where
        T: std::str::FromStr + TryFrom<u64> + TryFrom<i64>,
        E: serde::de::Error,
    {
        match self {
            Self::Text(text) => {
                let digits = text.strip_prefix('-').unwrap_or(&text);
                // Reject signs, whitespace, and empty text that `FromStr`
                // would otherwise accept or report ambiguously.
                if digits.is_empty() || !digits.bytes().all(|byte| byte.is_ascii_digit()) {
                    return Err(E::custom(format!("invalid integer text: {text:?}")));
                }
                text.parse()
                    .map_err(|_| E::custom(format!("integer out of range: {text}")))
            }
            Self::Unsigned(value) => {
                T::try_from(value).map_err(|_| E::custom(format!("integer out of range: {value}")))
            }
            Self::Signed(value) => {
                T::try_from(value).map_err(|_| E::custom(format!("integer out of range: {value}")))
            }
        }
    }
}

/// Exact `u64` carried as decimal text (JSON numbers are also accepted).
/// Serializes as text, mirroring the REST encoding.
pub mod u64_text {
    use serde::{Deserialize, Deserializer, Serializer};

    pub fn serialize<S: Serializer>(value: &u64, serializer: S) -> Result<S::Ok, S::Error> {
        serializer.collect_str(value)
    }

    pub fn deserialize<'de, D: Deserializer<'de>>(deserializer: D) -> Result<u64, D::Error> {
        super::IntegerRepr::deserialize(deserializer)?.parse()
    }
}

/// Optional [`u64_text`]; `null` or an omitted field (with `#[serde(default)]`)
/// is `None`.
pub mod opt_u64_text {
    use serde::{Deserialize, Deserializer, Serializer};

    pub fn serialize<S: Serializer>(value: &Option<u64>, serializer: S) -> Result<S::Ok, S::Error> {
        match value {
            Some(value) => serializer.collect_str(value),
            None => serializer.serialize_none(),
        }
    }

    pub fn deserialize<'de, D: Deserializer<'de>>(
        deserializer: D,
    ) -> Result<Option<u64>, D::Error> {
        Option::<super::IntegerRepr>::deserialize(deserializer)?
            .map(super::IntegerRepr::parse)
            .transpose()
    }
}

/// Exact signed `i128` carried as decimal text (raw signed atoms).
pub mod i128_text {
    use serde::{Deserialize, Deserializer, Serializer};

    pub fn serialize<S: Serializer>(value: &i128, serializer: S) -> Result<S::Ok, S::Error> {
        serializer.collect_str(value)
    }

    pub fn deserialize<'de, D: Deserializer<'de>>(deserializer: D) -> Result<i128, D::Error> {
        super::IntegerRepr::deserialize(deserializer)?.parse()
    }
}

/// Optional [`i128_text`]; `null` or an omitted field (with `#[serde(default)]`)
/// is `None`.
pub mod opt_i128_text {
    use serde::{Deserialize, Deserializer, Serializer};

    pub fn serialize<S: Serializer>(
        value: &Option<i128>,
        serializer: S,
    ) -> Result<S::Ok, S::Error> {
        match value {
            Some(value) => serializer.collect_str(value),
            None => serializer.serialize_none(),
        }
    }

    pub fn deserialize<'de, D: Deserializer<'de>>(
        deserializer: D,
    ) -> Result<Option<i128>, D::Error> {
        Option::<super::IntegerRepr>::deserialize(deserializer)?
            .map(super::IntegerRepr::parse)
            .transpose()
    }
}

/// [`Side`](crate::shared::Side) from either its text form (`"bid"`/`"ask"`)
/// or the numeric engine encoding (`0` = bid, `1` = ask) used by live
/// committed facts. Serializes as text.
pub mod side_text_or_number {
    use crate::shared::Side;
    use serde::{Deserialize, Deserializer, Serialize, Serializer};

    #[derive(Deserialize)]
    #[serde(untagged)]
    enum SideRepr {
        Text(Side),
        Number(u64),
    }

    pub fn serialize<S: Serializer>(side: &Side, serializer: S) -> Result<S::Ok, S::Error> {
        side.serialize(serializer)
    }

    pub fn deserialize<'de, D: Deserializer<'de>>(deserializer: D) -> Result<Side, D::Error> {
        match SideRepr::deserialize(deserializer)? {
            SideRepr::Text(side) => Ok(side),
            SideRepr::Number(0) => Ok(Side::Bid),
            SideRepr::Number(1) => Ok(Side::Ask),
            SideRepr::Number(other) => Err(serde::de::Error::custom(format!(
                "unknown numeric side: {other}"
            ))),
        }
    }
}

/// Deserializes `null`, an omitted field (with `#[serde(default)]`), or `""`
/// as `None`. The committed backend sends an empty string where a protobuf
/// string field is unset (closure reasons, snapshot cursors, global-account
/// market keys) and `null` elsewhere.
pub fn deserialize_nonempty_string<'de, D, T>(deserializer: D) -> Result<Option<T>, D::Error>
where
    D: Deserializer<'de>,
    T: From<String>,
{
    Ok(Option::<String>::deserialize(deserializer)?
        .filter(|value| !value.is_empty())
        .map(T::from))
}

#[cfg(test)]
mod committed_encoding_tests {
    use super::*;
    use crate::shared::Side;
    use serde::{Deserialize, Serialize};

    #[derive(Debug, Deserialize, Serialize, PartialEq)]
    struct Integers {
        #[serde(with = "u64_text")]
        revision: u64,
        #[serde(default, with = "opt_u64_text")]
        slot: Option<u64>,
        #[serde(with = "i128_text")]
        fee: i128,
        #[serde(default, with = "opt_i128_text")]
        remaining: Option<i128>,
        #[serde(with = "side_text_or_number")]
        side: Side,
        #[serde(default, deserialize_with = "deserialize_nonempty_string")]
        reason: Option<String>,
    }

    #[derive(Debug, Deserialize)]
    #[serde(tag = "event_type")]
    enum Tagged {
        #[serde(rename = "fact")]
        Fact(Integers),
    }

    #[test]
    fn integers_accept_text_and_numbers_and_serialize_as_text() {
        let value: Integers = serde_json::from_str(
            r#"{"revision":"18446744073709551615","slot":7,"fee":"-170141183460469231731687303715884105728","remaining":null,"side":1,"reason":""}"#,
        )
        .unwrap();
        assert_eq!(value.revision, u64::MAX);
        assert_eq!(value.slot, Some(7));
        assert_eq!(value.fee, i128::MIN);
        assert_eq!(value.remaining, None);
        assert_eq!(value.side, Side::Ask);
        assert_eq!(value.reason, None);

        let json = serde_json::to_value(&value).unwrap();
        assert_eq!(json["revision"], "18446744073709551615");
        assert_eq!(json["slot"], "7");
        assert_eq!(json["side"], "ask");
    }

    #[test]
    fn integers_decode_inside_internally_tagged_enums() {
        let Tagged::Fact(value) = serde_json::from_str(
            r#"{"event_type":"fact","revision":42,"fee":"12","side":"bid","reason":"expired"}"#,
        )
        .unwrap();
        assert_eq!(value.revision, 42);
        assert_eq!(value.slot, None);
        assert_eq!(value.fee, 12);
        assert_eq!(value.side, Side::Bid);
        assert_eq!(value.reason.as_deref(), Some("expired"));
    }

    #[test]
    fn integers_reject_malformed_or_out_of_range_values() {
        for json in [
            r#"{"revision":"-1","fee":"0","side":0}"#,
            r#"{"revision":"+1","fee":"0","side":0}"#,
            r#"{"revision":"1.0","fee":"0","side":0}"#,
            r#"{"revision":"18446744073709551616","fee":"0","side":0}"#,
            r#"{"revision":-1,"fee":"0","side":0}"#,
            r#"{"revision":"1","fee":"","side":0}"#,
            r#"{"revision":"1","fee":"0","side":2}"#,
        ] {
            assert!(serde_json::from_str::<Integers>(json).is_err(), "{json}");
        }
    }
}
