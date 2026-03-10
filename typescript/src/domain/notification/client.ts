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

  async dismiss(notificationId: string): Promise<void> {
    const url = `${this.client.http.baseUrl()}/api/notifications/dismiss`;
    await this.client.http.post<unknown, { notification_id: string }>(
      url,
      { notification_id: notificationId },
      RetryPolicy.None
    );
  }
}
