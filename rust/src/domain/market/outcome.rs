//! Outcome — market outcome definitions (sub-entity of market).

use super::resolve_icon_urls;
use super::wire::OutcomeResponse;
use serde::{Deserialize, Serialize};
use std::fmt;

/// A validated market outcome.
/// One market result whose artwork qualities are all populated or all absent.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Outcome {
    pub index: i16,
    /// Low-quality outcome artwork, cross-filled from another quality when available.
    pub icon_url_low: Option<String>,
    /// Medium-quality outcome artwork, cross-filled from another quality when available.
    pub icon_url_medium: Option<String>,
    /// High-quality outcome artwork, cross-filled from another quality when available.
    pub icon_url_high: Option<String>,
    pub name: String,
    pub name_long: Option<String>,
}

/// Retained outcome conversion errors for source compatibility.
///
/// Missing artwork is valid and no longer produces these errors.
#[derive(Debug)]
pub enum OutcomeValidationError {
    Multiple(String, Vec<OutcomeValidationError>),
    MissingIconUrl(String),
}

impl fmt::Display for OutcomeValidationError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            OutcomeValidationError::Multiple(name, errors) => {
                writeln!(f, "Outcome validation errors ({name}):")?;
                for err in errors {
                    writeln!(f, "  - {}", err)?;
                }
                Ok(())
            }
            OutcomeValidationError::MissingIconUrl(name) => {
                write!(f, "Missing icon URL for outcome: {}", name)
            }
        }
    }
}

impl std::error::Error for OutcomeValidationError {}

impl TryFrom<OutcomeResponse> for Outcome {
    type Error = OutcomeValidationError;

    fn try_from(source: OutcomeResponse) -> Result<Self, Self::Error> {
        let resolved = resolve_icon_urls(
            non_blank(source.icon_url_low),
            non_blank(source.icon_url_medium),
            non_blank(source.icon_url_high),
        );
        let (icon_url_low, icon_url_medium, icon_url_high) = match resolved {
            Some((low, medium, high)) => (Some(low), Some(medium), Some(high)),
            None => (None, None, None),
        };

        Ok(Outcome {
            index: source.index,
            icon_url_low,
            icon_url_medium,
            icon_url_high,
            name: source.name,
            name_long: source.name_long,
        })
    }
}

/// Treats empty and whitespace-only outcome artwork values as absent without rewriting URLs.
fn non_blank(value: Option<String>) -> Option<String> {
    value.filter(|value| !value.trim().is_empty())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_outcome_valid_conversion() {
        let wire = OutcomeResponse {
            index: 0,
            name: "Yes".to_string(),
            name_long: None,
            icon_url_low: Some("https://example.com/yes_low.png".to_string()),
            icon_url_medium: Some("https://example.com/yes_medium.png".to_string()),
            icon_url_high: Some("https://example.com/yes_high.png".to_string()),
        };
        let outcome = Outcome::try_from(wire).unwrap();
        assert_eq!(outcome.index, 0);
        assert_eq!(outcome.name, "Yes");
        assert_eq!(
            outcome.icon_url_low.as_deref(),
            Some("https://example.com/yes_low.png")
        );
        assert_eq!(
            outcome.icon_url_medium.as_deref(),
            Some("https://example.com/yes_medium.png")
        );
        assert_eq!(
            outcome.icon_url_high.as_deref(),
            Some("https://example.com/yes_high.png")
        );
    }

    #[test]
    fn outcome_without_artwork_remains_valid() {
        let wire = OutcomeResponse {
            index: 1,
            name: "No".to_string(),
            name_long: None,
            icon_url_low: None,
            icon_url_medium: None,
            icon_url_high: None,
        };
        let outcome = Outcome::try_from(wire).unwrap();
        assert_eq!(outcome.icon_url_low, None);
        assert_eq!(outcome.icon_url_medium, None);
        assert_eq!(outcome.icon_url_high, None);
    }

    #[test]
    fn omitted_outcome_artwork_deserializes_as_absent() {
        let wire: OutcomeResponse = serde_json::from_value(serde_json::json!({
            "index": 1,
            "name": "No"
        }))
        .unwrap();
        let outcome = Outcome::try_from(wire).unwrap();

        assert_eq!(outcome.icon_url_low, None);
        assert_eq!(outcome.icon_url_medium, None);
        assert_eq!(outcome.icon_url_high, None);
    }

    #[test]
    fn outcome_artwork_normalizes_blanks_and_cross_fills_present_quality() {
        let outcome = Outcome::try_from(OutcomeResponse {
            index: 0,
            name: "Yes".to_string(),
            name_long: None,
            icon_url_low: Some(" ".to_string()),
            icon_url_medium: Some("https://example.com/yes.png".to_string()),
            icon_url_high: Some(String::new()),
        })
        .unwrap();

        assert_eq!(
            outcome.icon_url_low.as_deref(),
            Some("https://example.com/yes.png")
        );
        assert_eq!(
            outcome.icon_url_medium.as_deref(),
            Some("https://example.com/yes.png")
        );
        assert_eq!(
            outcome.icon_url_high.as_deref(),
            Some("https://example.com/yes.png")
        );
    }

    #[test]
    fn retained_validation_error_surface_remains_available() {
        let error = OutcomeValidationError::MissingIconUrl("Yes".to_string());
        assert_eq!(error.to_string(), "Missing icon URL for outcome: Yes");
    }
}
