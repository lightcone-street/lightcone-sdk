//! HTTP client layer — `LightconeHttp` with per-endpoint retry policies.

pub mod client;
pub mod retry;

pub use client::{CookieSession, LightconeHttp};
pub use retry::{RetryConfig, RetryPolicy};
