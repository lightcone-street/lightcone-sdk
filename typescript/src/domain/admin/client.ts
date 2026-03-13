import { RetryPolicy, type LightconeHttp } from "../../http";
import type { AdminEnvelope } from "./index";
import type {
  AllocateCodesRequest,
  AllocateCodesResponse,
  CreateNotificationRequest,
  CreateNotificationResponse,
  DismissNotificationRequest,
  DismissNotificationResponse,
  RevokeRequest,
  RevokeResponse,
  UnifiedMetadataRequest,
  UnifiedMetadataResponse,
  UnrevokeRequest,
  UnrevokeResponse,
  WhitelistRequest,
  WhitelistResponse,
} from "./wire";

interface ClientContext {
  http: LightconeHttp;
}

export class Admin {
  constructor(private readonly client: ClientContext) {}

  async upsertMetadata(
    envelope: AdminEnvelope<UnifiedMetadataRequest>
  ): Promise<UnifiedMetadataResponse> {
    const url = `${this.client.http.baseUrl()}/api/admin/metadata`;
    return this.client.http.post<UnifiedMetadataResponse, AdminEnvelope<UnifiedMetadataRequest>>(
      url,
      envelope,
      RetryPolicy.None
    );
  }

  async allocateCodes(
    envelope: AdminEnvelope<AllocateCodesRequest>
  ): Promise<AllocateCodesResponse> {
    const url = `${this.client.http.baseUrl()}/api/admin/referral/allocate`;
    return this.client.http.post<AllocateCodesResponse, AdminEnvelope<AllocateCodesRequest>>(
      url,
      envelope,
      RetryPolicy.None
    );
  }

  async whitelist(envelope: AdminEnvelope<WhitelistRequest>): Promise<WhitelistResponse> {
    const url = `${this.client.http.baseUrl()}/api/admin/referral/whitelist`;
    return this.client.http.post<WhitelistResponse, AdminEnvelope<WhitelistRequest>>(
      url,
      envelope,
      RetryPolicy.None
    );
  }

  async revoke(envelope: AdminEnvelope<RevokeRequest>): Promise<RevokeResponse> {
    const url = `${this.client.http.baseUrl()}/api/admin/referral/revoke`;
    return this.client.http.post<RevokeResponse, AdminEnvelope<RevokeRequest>>(
      url,
      envelope,
      RetryPolicy.None
    );
  }

  async unrevoke(envelope: AdminEnvelope<UnrevokeRequest>): Promise<UnrevokeResponse> {
    const url = `${this.client.http.baseUrl()}/api/admin/referral/unrevoke`;
    return this.client.http.post<UnrevokeResponse, AdminEnvelope<UnrevokeRequest>>(
      url,
      envelope,
      RetryPolicy.None
    );
  }

  async createNotification(
    envelope: AdminEnvelope<CreateNotificationRequest>
  ): Promise<CreateNotificationResponse> {
    const url = `${this.client.http.baseUrl()}/api/admin/notifications`;
    return this.client.http.post<CreateNotificationResponse, AdminEnvelope<CreateNotificationRequest>>(
      url,
      envelope,
      RetryPolicy.None
    );
  }

  async dismissNotification(
    envelope: AdminEnvelope<DismissNotificationRequest>
  ): Promise<DismissNotificationResponse> {
    const url = `${this.client.http.baseUrl()}/api/admin/notifications/dismiss`;
    return this.client.http.post<DismissNotificationResponse, AdminEnvelope<DismissNotificationRequest>>(
      url,
      envelope,
      RetryPolicy.None
    );
  }
}
