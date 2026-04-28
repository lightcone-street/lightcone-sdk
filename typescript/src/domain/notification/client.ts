import { RetryPolicy, type LightconeHttp } from "../../http";
import type { Notification } from "./index";

interface NotificationsResponse {
  notifications: Notification[];
}

interface ClientContext {
  http: LightconeHttp;
}

export class Notifications {
  constructor(private readonly client: ClientContext) {}

  async fetch(): Promise<Notification[]> {
    const url = `${this.client.http.baseUrl()}/api/notifications`;
    const resp = await this.client.http.get<NotificationsResponse>(url, RetryPolicy.Idempotent);
    return resp.notifications;
  }

  /**
   * Same as {@link fetch}, but uses the supplied `authToken` for this call
   * instead of the SDK's process-wide cookie store. For server-side cookie
   * forwarding (SSR / route handlers).
   */
  async fetchWithAuthOverride(authToken: string): Promise<Notification[]> {
    const url = `${this.client.http.baseUrl()}/api/notifications`;
    const resp = await this.client.http.getWithAuth<NotificationsResponse>(
      url,
      RetryPolicy.Idempotent,
      authToken,
    );
    return resp.notifications;
  }

  async dismiss(notificationId: string): Promise<void> {
    const url = `${this.client.http.baseUrl()}/api/notifications/dismiss`;
    await this.client.http.post<{ status: string }, { notification_id: string }>(
      url,
      { notification_id: notificationId },
      RetryPolicy.None
    );
  }
}
