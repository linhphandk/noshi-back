use crate::auth::email::EmailProvider;
use crate::shared::errors::{AppError, AppResult};
use crate::waitlist::models::{NewWaitlistEntry, WaitlistResponse};
use crate::waitlist::ports::WaitlistRepository;
use std::sync::Arc;
use tracing::{debug, error, info, instrument};

#[derive(Clone)]
pub struct WaitlistService<R: WaitlistRepository, E: EmailProvider> {
    repo: R,
    email: Arc<E>,
    notification_email: String,
}

impl<R: WaitlistRepository, E: EmailProvider> WaitlistService<R, E> {
    pub fn new(repo: R, email: Arc<E>, notification_email: String) -> Self {
        Self {
            repo,
            email,
            notification_email,
        }
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

        debug!("sending notification email");
        if let Err(e) = self
            .email
            .send_notification(
                &self.notification_email,
                "New waitlist signup",
                &format!("Someone just joined the waitlist: {}", email),
            )
            .await
        {
            error!(error = ?e, "failed to send waitlist notification email");
        }

        info!(email = %email, "waitlist join complete");
        Ok(WaitlistResponse {
            message: "You're on the waitlist".to_string(),
        })
    }
}
