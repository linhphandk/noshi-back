use crate::shared::errors::{AppError, AppResult};
use crate::waitlist::models::{NewWaitlistEntry, WaitlistResponse};
use crate::waitlist::ports::WaitlistRepository;
use tracing::instrument;

#[derive(Clone)]
pub struct WaitlistService<R: WaitlistRepository> {
    repo: R,
}

impl<R: WaitlistRepository> WaitlistService<R> {
    pub fn new(repo: R) -> Self {
        Self { repo }
    }

    #[instrument(skip(self))]
    pub async fn join(&self, email: String) -> AppResult<WaitlistResponse> {
        if email.is_empty() || !email.contains('@') {
            return Err(AppError::BadRequest("Invalid email".to_string()));
        }

        if let Some(existing) = self.repo.find_by_email(&email).await? {
            return Ok(WaitlistResponse {
                position: existing.position,
                message: "You're already on the waitlist".to_string(),
            });
        }

        let entry = self.repo.create(NewWaitlistEntry { email }).await?;

        Ok(WaitlistResponse {
            position: entry.position,
            message: format!("You're #{} on the waitlist", entry.position),
        })
    }
}
