use crate::shared::errors::{AppError, AppResult};
use reqwest::Client;
use serde::Serialize;
use tracing::{debug, error, instrument};

#[derive(Serialize)]
struct EmailRequest {
    subject: String,
    message: String,
    from: String,
}

pub struct EmailClient {
    client: Client,
    endpoint: String,
}

impl EmailClient {
    pub fn new(endpoint: String) -> Self {
        Self {
            client: Client::new(),
            endpoint,
        }
    }

    #[instrument(skip(self))]
    pub async fn send_waitlist_confirmation(&self, to_email: &str) -> AppResult<()> {
        let subject = "Welcome to Noshi Waitlist!".to_string();
        let message = "Thanks for joining the Noshi waitlist!\n\nWe'll notify you when we launch.\n\n— The Noshi Team".to_string();
        let from = "noreply@noshi.com".to_string();

        let req = EmailRequest {
            subject,
            message,
            from,
        };

        debug!(to = %to_email, "sending waitlist confirmation email");

        let resp = self
            .client
            .post(&self.endpoint)
            .json(&req)
            .send()
            .await
            .map_err(|e| {
                error!("Email function request failed: {:?}", e);
                AppError::Internal
            })?;

        if !resp.status().is_success() {
            let err = resp.text().await.unwrap_or_default();
            error!("Email function returned error: {}", err);
            return Err(AppError::Internal);
        }

        debug!(to = %to_email, "waitlist confirmation email sent");
        Ok(())
    }
}
