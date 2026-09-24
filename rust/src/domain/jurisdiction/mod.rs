//! Jurisdiction classification (`GET /api/geoblock`).
//!
//! The backend answers from the country and tier that Cloudflare stamped on the
//! request. A server that relays a visitor's request forwards the visitor's
//! country with the relay secret; the response then reports `relayed: true`.

#[cfg(feature = "http")]
pub mod client;
pub mod wire;

#[cfg(feature = "http")]
pub use client::Jurisdiction;
pub use wire::*;
