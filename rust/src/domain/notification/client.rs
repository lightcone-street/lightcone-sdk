//! Notification sub-client — fetch and dismiss notifications.

use crate::client::LightconeClient;
use crate::domain::notification::Notification;
use crate::error::SdkError;
use crate::http::RetryPolicy;
#[cfg(not(target_arch = "wasm32"))]
use crate::http::{RelayContext, Relayed};
use serde::{Deserialize, Serialize};

pub struct Notifications<'a> {
    pub(crate) client: &'a LightconeClient,
}

#[derive(Deserialize)]
struct NotificationsResponse {
    notifications: Vec<Notification>,
}

#[derive(Serialize)]
struct DismissRequest {
    notification_id: String,
}

impl<'a> Notifications<'a> {
    pub async fn fetch(&self) -> Result<Vec<Notification>, SdkError> {
        let url = format!("{}/api/notifications", self.client.http.base_url());
        let body: NotificationsResponse =
            self.client.http.get(&url, RetryPolicy::Idempotent).await?;
        Ok(body.notifications)
    }

    /// Same as [`Self::fetch`], but forwards the supplied raw `Cookie` header (`privy-token` and/or `lightcone-token`) for this
    /// call instead of the SDK's process-wide token store. For server-side
    /// cookie forwarding (SSR / server functions).
    pub async fn fetch_with_cookies(
        &self,
        cookie_header: &str,
    ) -> Result<Vec<Notification>, SdkError> {
        let url = format!("{}/api/notifications", self.client.http.base_url());
        let body: NotificationsResponse = self
            .client
            .http
            .get_with_cookies(&url, RetryPolicy::Idempotent, cookie_header)
            .await?;
        Ok(body.notifications)
    }

    pub async fn dismiss(&self, notification_id: &str) -> Result<(), SdkError> {
        let url = format!("{}/api/notifications/dismiss", self.client.http.base_url());
        let body = DismissRequest {
            notification_id: notification_id.to_string(),
        };
        let _: serde_json::Value = self
            .client
            .http
            .post(&url, &body, RetryPolicy::None)
            .await?;
        Ok(())
    }

    /// Same as [`Self::dismiss`], but forwards the supplied raw `Cookie` header (`privy-token` and/or `lightcone-token`) for this
    /// call instead of the SDK's process-wide token store. For server-side
    /// cookie forwarding (SSR / server functions).
    pub async fn dismiss_with_cookies(
        &self,
        notification_id: &str,
        cookie_header: &str,
    ) -> Result<(), SdkError> {
        let url = format!("{}/api/notifications/dismiss", self.client.http.base_url());
        let body = DismissRequest {
            notification_id: notification_id.to_string(),
        };
        let _: serde_json::Value = self
            .client
            .http
            .post_with_cookies(&url, &body, RetryPolicy::None, cookie_header)
            .await?;
        Ok(())
    }

    /// Dismiss on behalf of one visitor, preserving its cookies and verified
    /// country and returning any backend Set-Cookie headers to the caller.
    #[cfg(not(target_arch = "wasm32"))]
    pub async fn dismiss_relayed(
        &self,
        notification_id: &str,
        context: &RelayContext,
    ) -> Result<Relayed<()>, SdkError> {
        let url = format!("{}/api/notifications/dismiss", self.client.http.base_url());
        let body = DismissRequest {
            notification_id: notification_id.to_string(),
        };
        let relayed: Relayed<serde_json::Value> = self
            .client
            .http
            .post_relayed(&url, &body, RetryPolicy::None, context)
            .await?;
        Ok(Relayed {
            body: (),
            set_cookie: relayed.set_cookie,
        })
    }
}
