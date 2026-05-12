#![doc = include_str!("ADMIN.md")]

pub mod client;
pub mod wire;

pub use wire::{
    AddMetadataCategoryRequest, AddMetadataCategoryResponse, AdminLogEvent, AdminLogEventsQuery,
    AdminLogEventsResponse, AdminLogMetricBreakdown, AdminLogMetricHistoryQuery,
    AdminLogMetricHistoryResponse, AdminLogMetricPoint, AdminLogMetricSummary,
    AdminLogMetricsQuery, AdminLogMetricsResponse, AdminLoginRequest, AdminLoginResponse,
    AdminNonceResponse, AllocateCodesRequest, AllocateCodesResponse, CodeListEntry,
    CreateNotificationRequest, CreateNotificationResponse, DepositTokenMetadataPayload,
    DepositTokenMetadataResponse, DismissNotificationRequest, DismissNotificationResponse,
    ListCodesRequest, ListCodesResponse, MarketDeploymentConditionalToken,
    MarketDeploymentDepositAsset, MarketDeploymentMarket, MarketDeploymentOutcome,
    MetadataCategoriesResponse, ReferralConfig, RevokeRequest, RevokeResponse, TargetSpec,
    UnifiedMetadataRequest, UnifiedMetadataResponse, UnrevokeRequest, UnrevokeResponse,
    UpdateCodeRequest, UpdateCodeResponse, UpdateConfigRequest,
    UploadMarketDeploymentAssetsRequest, UploadMarketDeploymentAssetsResponse,
    UploadedConditionalToken, UploadedDepositAssetImages, UploadedMarketImages,
    UploadedOutcomeImages, WhitelistRequest, WhitelistResponse,
};
