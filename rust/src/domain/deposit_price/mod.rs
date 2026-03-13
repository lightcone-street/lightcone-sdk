//! Deposit-price websocket types and state.

pub mod state;
pub mod wire;

use crate::shared::{PubkeyStr, Resolution};
use serde::{Deserialize, Serialize};

pub use state::{DepositPriceState, LatestDepositPrice};
pub use wire::{
    DepositPrice, DepositPriceCandleUpdate, DepositPriceSnapshot, DepositPriceTick,
    DepositTokenCandle,
};

/// Key for deposit-price lookups.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct DepositPriceKey {
    pub deposit_asset: PubkeyStr,
    pub resolution: Resolution,
}
