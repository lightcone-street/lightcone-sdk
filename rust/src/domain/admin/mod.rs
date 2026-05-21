#![doc = include_str!("ADMIN.md")]

pub mod client;
pub mod wire;

pub use wire::{
    AddMetadataCategoryRequest, AddMetadataCategoryResponse, AdminConditionalMintRow,
    AdminConditionalTokenMetadataEntry, AdminConditionalTokenMetadataRow,
    AdminDepositTokenMetadataResponse, AdminImageVariants, AdminLogEvent, AdminLogEventsQuery,
    AdminLogEventsResponse, AdminLogMetricBreakdown, AdminLogMetricHistoryQuery,
    AdminLogMetricHistoryResponse, AdminLogMetricPoint, AdminLogMetricSummary,
    AdminLogMetricsQuery, AdminLogMetricsResponse, AdminLoginRequest, AdminLoginResponse,
    AdminMarketDepositAsset, AdminMarketMetadataResponse, AdminMarketMetadataRow, AdminMarketRow,
    AdminMarketStatus, AdminMarketStatusFilter, AdminMarketsQuery, AdminMarketsResponse,
    AdminMetadataMarket, AdminMissingMetadata, AdminNonceResponse, AdminOutcomeMetadataEntry,
    AdminOutcomeMetadataRow, AllocateCodesRequest, AllocateCodesResponse, CodeListEntry,
    ConditionalTokenMetadataPayload, CreateNotificationRequest, CreateNotificationResponse,
    CriticalLogErrors24hCountResponse, DepositTokenMetadataPayload, DepositTokenMetadataResponse,
    DismissNotificationRequest, DismissNotificationResponse, ListCodesRequest, ListCodesResponse,
    MarketDeploymentConditionalToken, MarketDeploymentDepositAsset, MarketDeploymentMarket,
    MarketDeploymentOutcome, MarketMetadataPayload, MarketsToSettleCountResponse,
    MarketsToSettleQuery, MarketsToSettleResponse, MetadataCategoriesResponse,
    MetadataImageTargetType, MetadataImageUpdate, MetadataImageUpdateResponse,
    OutcomeMetadataPayload, ReferralConfig, RevokeRequest, RevokeResponse, TargetSpec,
    UnifiedMetadataRequest, UnifiedMetadataResponse, UnrevokeRequest, UnrevokeResponse,
    UpdateCodeRequest, UpdateCodeResponse, UpdateConditionalTokenImageRequest,
    UpdateConditionalTokenMetadataPayload, UpdateConfigRequest, UpdateDepositTokenImagesRequest,
    UpdateDepositTokenMetadataRequest, UpdateDepositTokenMetadataResponse,
    UpdateMarketImagesRequest, UpdateMarketMetadataPayload, UpdateMarketMetadataRequest,
    UpdateMarketMetadataResponse, UpdateOutcomeImageRequest, UpdateOutcomeMetadataPayload,
    UploadMarketDeploymentAssetsRequest, UploadMarketDeploymentAssetsResponse,
    UploadedConditionalToken, UploadedDepositAssetImages, UploadedMarketImages,
    UploadedOutcomeImages, WhitelistRequest, WhitelistResponse,
};
