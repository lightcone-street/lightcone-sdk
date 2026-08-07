//! Exact decimal JSON numbers.

use std::{fmt, str::FromStr};

use serde::{Deserialize, Serialize};

/// A JSON number that preserves its decimal value without passing through
/// binary floating point.
///
/// This is primarily used for trigger prices, whose public wire field remains
/// a JSON number even though the SDK accepts and validates decimal strings.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct ExactDecimal(serde_json::Number);

impl ExactDecimal {
    /// Return the exact JSON number representation.
    pub fn as_str(&self) -> &str {
        self.0.as_str()
    }
}

impl fmt::Display for ExactDecimal {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.0.fmt(formatter)
    }
}

impl FromStr for ExactDecimal {
    type Err = serde_json::Error;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        serde_json::from_str(value).map(Self)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn preserves_values_beyond_f64_precision_as_json_numbers() {
        let input = "900719925474.0993";
        let value: ExactDecimal = input.parse().unwrap();

        assert_eq!(value.as_str(), input);
        assert_eq!(serde_json::to_string(&value).unwrap(), input);
        assert_eq!(serde_json::from_str::<ExactDecimal>(input).unwrap(), value);
    }
}
