//! Retry policies for HTTP requests.

use std::time::Duration;

/// Retry policy for an HTTP request.
#[derive(Debug, Clone)]
pub enum RetryPolicy {
    /// No retries — used for non-idempotent POST endpoints by default.
    None,
    /// Retry on transport failures + 502/503/504, with backoff on 429.
    /// Default for GET endpoints.
    Idempotent,
    /// User-provided retry logic.
    Custom(RetryConfig),
}

impl Default for RetryPolicy {
    fn default() -> Self {
        RetryPolicy::None
    }
}

/// Configuration for retry behavior.
#[derive(Debug, Clone)]
pub struct RetryConfig {
    /// Maximum number of retry attempts (not counting the initial request).
    pub max_retries: u32,
    /// Initial delay before the first retry.
    pub initial_delay: Duration,
    /// Maximum delay between retries.
    pub max_delay: Duration,
    /// Multiplier applied to the delay after each retry.
    pub backoff_factor: f64,
    /// Whether to add jitter to the delay.
    pub jitter: bool,
    /// HTTP status codes that trigger a retry.
    pub retryable_statuses: Vec<u16>,
}

impl Default for RetryConfig {
    fn default() -> Self {
        Self {
            max_retries: 3,
            initial_delay: Duration::from_millis(200),
            max_delay: Duration::from_secs(10),
            backoff_factor: 2.0,
            jitter: true,
            retryable_statuses: vec![502, 503, 504],
        }
    }
}

impl RetryConfig {
    /// The default config for idempotent (GET) requests.
    pub fn idempotent() -> Self {
        Self {
            max_retries: 3,
            initial_delay: Duration::from_millis(200),
            max_delay: Duration::from_secs(10),
            backoff_factor: 2.0,
            jitter: true,
            retryable_statuses: vec![429, 502, 503, 504],
        }
    }

    /// Calculate delay for a given attempt (0-indexed).
    pub fn delay_for_attempt(&self, attempt: u32) -> Duration {
        let base = self.initial_delay.as_millis() as f64
            * self.backoff_factor.powi(attempt as i32);
        let capped = base.min(self.max_delay.as_millis() as f64);

        let final_ms = if self.jitter {
            let jitter_range = capped * 0.25;
            let jitter = (rand::random::<f64>() - 0.5) * 2.0 * jitter_range;
            (capped + jitter).max(0.0)
        } else {
            capped
        };

        Duration::from_millis(final_ms as u64)
    }
}
