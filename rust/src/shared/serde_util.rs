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
