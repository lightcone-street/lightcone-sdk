//! Custom serde helpers for backend wire formats.

/// Deserializes a Unix-millis `u64` into `DateTime<Utc>`.
///
/// The backend's WebSocket sends `created_at` as epoch milliseconds (i64/u64),
/// not ISO 8601 strings.
pub mod timestamp_ms {
    use chrono::{DateTime, Utc};
    use serde::{Deserialize, Deserializer};

    pub fn deserialize<'de, D>(deserializer: D) -> Result<DateTime<Utc>, D::Error>
    where
        D: Deserializer<'de>,
    {
        let millis = u64::deserialize(deserializer)?;
        DateTime::<Utc>::from_timestamp_millis(millis as i64)
            .ok_or_else(|| serde::de::Error::custom(format!("Invalid timestamp: {}", millis)))
    }
}
