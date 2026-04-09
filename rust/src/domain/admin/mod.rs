#![doc = include_str!("ADMIN.md")]

pub mod client;
pub mod wire;

pub use wire::{
    AdminLoginRequest, AdminLoginResponse, AdminNonceResponse, AllocateCodesRequest,
    AllocateCodesResponse, CreateNotificationRequest, CreateNotificationResponse,
    DismissNotificationRequest, DismissNotificationResponse, RevokeRequest, RevokeResponse,
    TargetSpec, UnifiedMetadataRequest, UnifiedMetadataResponse, UnrevokeRequest, UnrevokeResponse,
    WhitelistRequest, WhitelistResponse,
};
