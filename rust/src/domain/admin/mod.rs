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
    MarketDeploymentConditionalToken, MarketDeploymentDepositAsset, MarketDeploymentMarket,
    MarketDeploymentOutcome, ReferralConfig, RevokeRequest, RevokeResponse, TargetSpec,
    UnifiedMetadataRequest, UnifiedMetadataResponse, UnrevokeRequest, UnrevokeResponse,
    UpdateCodeRequest, UpdateCodeResponse, UpdateConfigRequest,
    UploadMarketDeploymentAssetsRequest, UploadMarketDeploymentAssetsResponse,
    UploadedConditionalToken, UploadedMarketImages, UploadedOutcomeImages, WhitelistRequest,
    WhitelistResponse,
};
