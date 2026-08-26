use crate::shared::email_client::EmailClient;
use crate::shared::errors::{AppError, AppResult};
use crate::waitlist::models::{NewWaitlistEntry, WaitlistResponse};
use crate::waitlist::ports::WaitlistRepository;
use std::sync::Arc;
use tracing::{debug, error, instrument};

#[derive(Clone)]
pub struct WaitlistService<R: WaitlistRepository> {
    repo: R,
    email_client: Option<Arc<EmailClient>>,
}

impl<R: WaitlistRepository> WaitlistService<R> {
    pub fn new(repo: R) -> Self {
        Self {
            repo,
            email_client: None,
        }
    }

    pub fn with_email_client(mut self, client: Option<Arc<EmailClient>>) -> Self {
        self.email_client = client;
        self
    }

    #[instrument(skip(self), fields(email = %email))]
    pub async fn join(&self, email: String) -> AppResult<WaitlistResponse> {
        debug!("validating email");
        if email.is_empty() || !email.contains('@') {
            return Err(AppError::BadRequest("Invalid email".to_string()));
        }

        debug!("checking for existing entry");
        if self.repo.find_by_email(&email).await?.is_some() {
            return Err(AppError::Conflict("Email already on waitlist".to_string()));
        }

        debug!("inserting waitlist entry");
        self.repo
            .create(NewWaitlistEntry {
                email: email.clone(),
            })
            .await?;

        if let Some(client) = &self.email_client {
            if let Err(e) = client.send_waitlist_confirmation(&email).await {
                error!(email = %email, error = %e, "failed to send waitlist confirmation email");
            }
        }

        Ok(WaitlistResponse {
            message: "You're on the waitlist".to_string(),
        })
    }
}
