//! Notification sub-client — fetch and dismiss notifications.

use crate::client::LightconeClient;
use crate::domain::notification::Notification;
use crate::error::SdkError;
use crate::http::RetryPolicy;
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
}
