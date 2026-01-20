//! Shared type definitions for the Lightcone SDK.
//!
//! This module contains types that are used by both the REST API and WebSocket modules.

// ============================================================================
// Resolution Enum (shared between API and WebSocket)
// ============================================================================

/// Price history candle resolution.
///
/// Used by both REST API and WebSocket for price history queries.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
pub enum Resolution {
    /// 1 minute candles
    #[default]
    #[serde(rename = "1m")]
    OneMinute,
    /// 5 minute candles
    #[serde(rename = "5m")]
    FiveMinutes,
    /// 15 minute candles
    #[serde(rename = "15m")]
    FifteenMinutes,
    /// 1 hour candles
    #[serde(rename = "1h")]
    OneHour,
    /// 4 hour candles
    #[serde(rename = "4h")]
    FourHours,
    /// 1 day candles
    #[serde(rename = "1d")]
    OneDay,
}

impl Resolution {
    /// Get the string representation.
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::OneMinute => "1m",
            Self::FiveMinutes => "5m",
            Self::FifteenMinutes => "15m",
            Self::OneHour => "1h",
            Self::FourHours => "4h",
            Self::OneDay => "1d",
        }
    }
}

impl std::fmt::Display for Resolution {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_str())
    }
}
