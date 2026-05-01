//! Outcome — market outcome definitions (sub-entity of market).

use super::wire::OutcomeResponse;
use serde::{Deserialize, Serialize};
use std::fmt;

/// A validated market outcome.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Outcome {
    pub index: i16,
    pub icon_url_low: String,
    pub icon_url_medium: String,
    pub icon_url_high: String,
    pub name: String,
}

/// Errors when validating an outcome response.
#[derive(Debug)]
pub enum OutcomeValidationError {
    Multiple(String, Vec<OutcomeValidationError>),
    MissingThumbnailUrl(String),
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
            OutcomeValidationError::MissingThumbnailUrl(name) => {
                write!(f, "Missing thumbnail URL for outcome: {}", name)
            }
        }
    }
}

impl std::error::Error for OutcomeValidationError {}

impl TryFrom<OutcomeResponse> for Outcome {
    type Error = OutcomeValidationError;

    fn try_from(source: OutcomeResponse) -> Result<Self, Self::Error> {
        let mut errors: Vec<OutcomeValidationError> = Vec::new();

        let icon_url_low = source.icon_url_low.unwrap_or_else(|| {
            errors.push(OutcomeValidationError::MissingThumbnailUrl(
                source.name.clone(),
            ));
            String::new()
        });
        let icon_url_medium = source.icon_url_medium.unwrap_or_default();
        let icon_url_high = source.icon_url_high.unwrap_or_default();

        if !errors.is_empty() {
            return Err(OutcomeValidationError::Multiple(
                source.name.clone(),
                errors,
            ));
        }

        Ok(Outcome {
            index: source.index,
            icon_url_low,
            icon_url_medium,
            icon_url_high,
            name: source.name,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_outcome_valid_conversion() {
        let wire = OutcomeResponse {
            index: 0,
            name: "Yes".to_string(),
            icon_url_low: Some("https://example.com/yes_low.png".to_string()),
            icon_url_medium: Some("https://example.com/yes_medium.png".to_string()),
            icon_url_high: Some("https://example.com/yes_high.png".to_string()),
        };
        let outcome = Outcome::try_from(wire).unwrap();
        assert_eq!(outcome.index, 0);
        assert_eq!(outcome.name, "Yes");
        assert_eq!(outcome.icon_url_low, "https://example.com/yes_low.png");
        assert_eq!(
            outcome.icon_url_medium,
            "https://example.com/yes_medium.png"
        );
        assert_eq!(outcome.icon_url_high, "https://example.com/yes_high.png");
    }

    #[test]
    fn test_outcome_missing_icon_url_fails() {
        let wire = OutcomeResponse {
            index: 1,
            name: "No".to_string(),
            icon_url_low: None,
            icon_url_medium: None,
            icon_url_high: None,
        };
        let err = Outcome::try_from(wire).unwrap_err();
        assert!(format!("{err}").contains("Missing thumbnail"));
    }
}
