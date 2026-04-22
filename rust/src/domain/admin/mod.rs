#![doc = include_str!("ADMIN.md")]

pub mod client;
pub mod wire;

pub use wire::{
    AdminLogEvent, AdminLogEventsQuery, AdminLogEventsResponse, AdminLogMetricBreakdown,
    AdminLogMetricHistoryQuery, AdminLogMetricHistoryResponse, AdminLogMetricPoint,
    AdminLogMetricSummary, AdminLogMetricsQuery, AdminLogMetricsResponse, AdminLoginRequest,
    AdminLoginResponse, AdminNonceResponse, AllocateCodesRequest, AllocateCodesResponse,
    CodeListEntry, CreateNotificationRequest, CreateNotificationResponse,
    DismissNotificationRequest, DismissNotificationResponse, ListCodesRequest, ListCodesResponse,
    ReferralConfig, RevokeRequest, RevokeResponse, TargetSpec, UnifiedMetadataRequest,
    UnifiedMetadataResponse, UnrevokeRequest, UnrevokeResponse, UpdateCodeRequest,
    UpdateCodeResponse, UpdateConfigRequest, WhitelistRequest, WhitelistResponse,
};
