//! Hyperliquid-style book aggregation parameters.

/// Book aggregation parameters for orderbook subscriptions and REST depth.
///
/// `(None, None)` is full precision. `n_sig_figs` must be 2, 3, 4, or 5;
/// `mantissa` must be 1, 2, or 5 and is only valid with `n_sig_figs = 5`.
/// `(Some(5), None)` normalizes to `(Some(5), Some(1))` — the backend treats
/// them as the same subscription, so all key/matching logic in the SDK
/// compares normalized values.
///
/// Fields are snake_case like the rest of the SDK; the camelCase `nSigFigs`
/// spelling exists only at the wire boundary (subscribe params and REST query
/// keys). Incoming `book_update` frames tag their view with snake_case
/// `n_sig_figs`/`mantissa`, omitted for full precision.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub struct BookAggregation {
    pub n_sig_figs: Option<u32>,
    pub mantissa: Option<u32>,
}

impl BookAggregation {
    /// Full precision (no aggregation).
    pub const FULL: Self = Self {
        n_sig_figs: None,
        mantissa: None,
    };

    /// Validate aggregation params against the backend's contract, returning
    /// the normalized form. Invalid combinations are rejected server-side
    /// (`INVALID_ORDERBOOK_SUBSCRIPTION` / HTTP 400), so validate before
    /// sending.
    pub fn validate(n_sig_figs: Option<u32>, mantissa: Option<u32>) -> Result<Self, &'static str> {
        match (n_sig_figs, mantissa) {
            (None, None) => Ok(Self::FULL),
            (None, Some(_)) => Err("mantissa is only valid when nSigFigs is 5"),
            (Some(2..=4), None) => Ok(Self {
                n_sig_figs,
                mantissa: None,
            }),
            (Some(2..=4), Some(_)) => Err("mantissa is only valid when nSigFigs is 5"),
            (Some(5), None) => Ok(Self {
                n_sig_figs: Some(5),
                mantissa: Some(1),
            }),
            (Some(5), Some(1 | 2 | 5)) => Ok(Self {
                n_sig_figs: Some(5),
                mantissa,
            }),
            (Some(5), Some(_)) => Err("mantissa must be 1, 2, or 5"),
            _ => Err("nSigFigs must be 2, 3, 4, 5, or omitted"),
        }
    }

    /// Normalized form: `(5, None)` becomes `(5, Some(1))`; everything else is
    /// unchanged. Lenient — never errors. Use [`Self::validate`] to reject
    /// invalid combinations before sending to the server.
    pub fn normalized(self) -> Self {
        match (self.n_sig_figs, self.mantissa) {
            (Some(5), None) => Self {
                n_sig_figs: Some(5),
                mantissa: Some(1),
            },
            _ => self,
        }
    }

    /// Aggregation identified by an incoming frame's tags. Untagged frames
    /// (both fields absent) are full precision. Lenient — never errors.
    pub fn from_frame(n_sig_figs: Option<u32>, mantissa: Option<u32>) -> Self {
        Self {
            n_sig_figs,
            mantissa,
        }
        .normalized()
    }

    /// Whether this is the full-precision (no aggregation) view.
    pub fn is_full(self) -> bool {
        self.normalized() == Self::FULL
    }

    /// Stable suffix for subscription keys: `"full"`, `"sig2"`..`"sig4"`, or
    /// `"sig5m1"`/`"sig5m2"`/`"sig5m5"`. Matches the backend's subscription-key
    /// vocabulary so keys are comparable across normalized spellings.
    pub fn key_suffix(self) -> String {
        let normalized = self.normalized();
        match (normalized.n_sig_figs, normalized.mantissa) {
            (None, None) => "full".to_string(),
            (Some(n_sig_figs), None) => format!("sig{n_sig_figs}"),
            (Some(n_sig_figs), Some(mantissa)) => format!("sig{n_sig_figs}m{mantissa}"),
            (None, Some(_)) => "invalid".to_string(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validate_matches_backend_contract() {
        assert!(BookAggregation::validate(None, None).is_ok());
        assert!(BookAggregation::validate(Some(2), None).is_ok());
        assert!(BookAggregation::validate(Some(3), None).is_ok());
        assert!(BookAggregation::validate(Some(4), None).is_ok());
        assert!(BookAggregation::validate(Some(5), None).is_ok());
        assert!(BookAggregation::validate(Some(5), Some(1)).is_ok());
        assert!(BookAggregation::validate(Some(5), Some(2)).is_ok());
        assert!(BookAggregation::validate(Some(5), Some(5)).is_ok());

        assert!(BookAggregation::validate(Some(0), None).is_err());
        assert!(BookAggregation::validate(Some(1), None).is_err());
        assert!(BookAggregation::validate(Some(6), None).is_err());
        assert!(BookAggregation::validate(Some(4), Some(2)).is_err());
        assert!(BookAggregation::validate(None, Some(2)).is_err());
        assert!(BookAggregation::validate(Some(5), Some(0)).is_err());
        assert!(BookAggregation::validate(Some(5), Some(3)).is_err());
    }

    #[test]
    fn test_sig_figs_five_normalizes_to_mantissa_one() {
        let validated = BookAggregation::validate(Some(5), None);
        assert_eq!(
            validated,
            Ok(BookAggregation {
                n_sig_figs: Some(5),
                mantissa: Some(1),
            })
        );

        let normalized = BookAggregation {
            n_sig_figs: Some(5),
            mantissa: None,
        }
        .normalized();
        assert_eq!(normalized.mantissa, Some(1));
        assert_eq!(
            normalized,
            BookAggregation::from_frame(Some(5), Some(1)),
            "(5, none) and (5, 1) are the same subscription"
        );
    }

    #[test]
    fn test_from_frame_untagged_is_full_precision() {
        assert_eq!(
            BookAggregation::from_frame(None, None),
            BookAggregation::FULL
        );
        assert!(BookAggregation::from_frame(None, None).is_full());
        assert!(!BookAggregation::from_frame(Some(4), None).is_full());
    }

    #[test]
    fn test_key_suffix_vocabulary() {
        assert_eq!(BookAggregation::FULL.key_suffix(), "full");
        assert_eq!(
            BookAggregation::from_frame(Some(2), None).key_suffix(),
            "sig2"
        );
        assert_eq!(
            BookAggregation::from_frame(Some(5), None).key_suffix(),
            "sig5m1"
        );
        assert_eq!(
            BookAggregation::from_frame(Some(5), Some(2)).key_suffix(),
            "sig5m2"
        );
    }
}
