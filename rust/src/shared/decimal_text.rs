//! Exact decimal text for aggregates wider than `Decimal`.

use rust_decimal::Decimal;
use serde::{Deserialize, Deserializer, Serialize};
use std::{fmt, str::FromStr};

/// An exact, canonical decimal string such as `"12.500000"` or `"-0.000001"`.
///
/// The committed backend formats some aggregates from `u128`/`i128` atom
/// counts (funding reservations, signed remainders). Those can exceed the
/// 96-bit range of [`Decimal`], so the SDK keeps them as validated text
/// instead of failing the whole response. Use [`Self::to_decimal`] when the
/// value is known to fit, which it does for every realistic balance.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize)]
#[serde(transparent)]
pub struct DecimalText(String);

impl DecimalText {
    /// The exact text as sent by the backend.
    pub fn as_str(&self) -> &str {
        &self.0
    }

    /// Exact [`Decimal`] value, or `None` when it exceeds `Decimal`'s range.
    pub fn to_decimal(&self) -> Option<Decimal> {
        Decimal::from_str_exact(&self.0).ok()
    }

    /// True when the value is negative (never `-0`, which is rejected).
    pub fn is_negative(&self) -> bool {
        self.0.starts_with('-')
    }

    fn validate(text: &str) -> Result<(), String> {
        let unsigned = text.strip_prefix('-').unwrap_or(text);
        let (whole, fraction) = match unsigned.split_once('.') {
            Some((whole, fraction)) => (whole, Some(fraction)),
            None => (unsigned, None),
        };
        let digits = |part: &str| !part.is_empty() && part.bytes().all(|b| b.is_ascii_digit());
        if !digits(whole) || fraction.is_some_and(|fraction| !digits(fraction)) {
            return Err(format!("invalid decimal text: {text:?}"));
        }
        if text.starts_with('-') && unsigned.bytes().all(|b| b == b'0' || b == b'.') {
            return Err(format!("negative zero is not canonical: {text:?}"));
        }
        Ok(())
    }
}

impl FromStr for DecimalText {
    type Err = String;

    fn from_str(text: &str) -> Result<Self, Self::Err> {
        Self::validate(text)?;
        Ok(Self(text.to_string()))
    }
}

impl<'de> Deserialize<'de> for DecimalText {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let text = String::deserialize(deserializer)?;
        text.parse().map_err(serde::de::Error::custom)
    }
}

impl fmt::Display for DecimalText {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(&self.0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_u128_aggregates_exact_beyond_decimal_range() {
        // Canonical backend fixture: u128::MAX atoms at six decimals.
        let text: DecimalText =
            serde_json::from_str(r#""340282366920938463463374607431768.211455""#).unwrap();
        assert_eq!(text.as_str(), "340282366920938463463374607431768.211455");
        assert_eq!(text.to_decimal(), None);
        assert_eq!(
            serde_json::to_string(&text).unwrap(),
            r#""340282366920938463463374607431768.211455""#
        );
    }

    #[test]
    fn converts_values_in_decimal_range() {
        let text: DecimalText = "-0.000001".parse().unwrap();
        assert!(text.is_negative());
        assert_eq!(text.to_decimal(), Some(Decimal::new(-1, 6)));
        let whole: DecimalText = "40".parse().unwrap();
        assert_eq!(whole.to_decimal(), Some(Decimal::from(40)));
    }

    #[test]
    fn rejects_non_canonical_text() {
        for text in [
            "", "-", "1.", ".5", "+1", "1e3", " 1", "-0", "-0.000", "0x1",
        ] {
            assert!(text.parse::<DecimalText>().is_err(), "{text:?}");
        }
        assert!(serde_json::from_str::<DecimalText>("12").is_err());
    }
}
