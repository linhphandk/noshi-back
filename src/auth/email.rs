use async_trait::async_trait;
use aws_sdk_sesv2::types::{Body, Content, Destination, EmailContent, Message};
use aws_sdk_sesv2::Client as SesClient;
use tracing::{error, info, instrument};

use crate::shared::errors::{AppError, AppResult};

#[async_trait]
pub trait EmailProvider: Send + Sync {
    async fn send_password_reset(&self, to: &str, reset_url: &str) -> AppResult<()>;
    async fn send_notification(&self, to: &str, subject: &str, body: &str) -> AppResult<()>;
}

#[derive(Clone)]
pub struct SesEmailProvider {
    client: SesClient,
    from_email: String,
    from_name: String,
}

impl SesEmailProvider {
    pub fn new(client: SesClient, from_email: &str, from_name: &str) -> Self {
        Self {
            client,
            from_email: from_email.to_string(),
            from_name: from_name.to_string(),
        }
    }

    async fn send(&self, to: &str, subject: &str, body_text: &str) -> AppResult<()> {
        let from = format!("{} <{}>", self.from_name, self.from_email);

        let dest = Destination::builder().to_addresses(to).build();

        let subject_content = Content::builder().data(subject).build().map_err(|e| {
            error!(error = %e, "SES subject build failed");
            AppError::Internal
        })?;

        let body_content = Content::builder().data(body_text).build().map_err(|e| {
            error!(error = %e, "SES body build failed");
            AppError::Internal
        })?;

        let msg = Message::builder()
            .subject(subject_content)
            .body(Body::builder().text(body_content).build())
            .build();

        let content = EmailContent::builder().simple(msg).build();

        let result = self
            .client
            .send_email()
            .from_email_address(from)
            .destination(dest)
            .content(content)
            .send()
            .await;

        match result {
            Ok(_) => {
                info!(to = %to, "email sent via SES");
                Ok(())
            }
            Err(e) => {
                error!(to = %to, error = %e, "SES send failed");
                Err(AppError::Internal)
            }
        }
    }
}

#[async_trait]
impl EmailProvider for SesEmailProvider {
    #[instrument(skip(self), fields(to = %to))]
    async fn send_password_reset(&self, to: &str, reset_url: &str) -> AppResult<()> {
        self.send(
            to,
            "Reset your password",
            &format!(
                "Click the link below to reset your password:\n\n{}\n\nThis link expires in 1 hour.",
                reset_url
            ),
        )
        .await
    }

    #[instrument(skip(self), fields(to = %to))]
    async fn send_notification(&self, to: &str, subject: &str, body: &str) -> AppResult<()> {
        self.send(to, subject, body).await
    }
}
