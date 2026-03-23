//! Custom serde helpers for backend wire formats.

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
