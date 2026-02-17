//! Admin domain — metadata management types.

pub mod client;
pub mod wire;

use serde::{Deserialize, Serialize};

pub use wire::{UnifiedMetadataRequest, UnifiedMetadataResponse};

/// Signed admin request envelope.
///
/// All admin requests are wrapped in this envelope. The payload is serialized to
/// canonical JSON and signed with an ED25519 key authorized in the backend.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AdminEnvelope<T: Serialize> {
    pub payload: T,
    /// Base58-encoded ED25519 signature over canonical JSON of the payload.
    pub signature: String,
}
