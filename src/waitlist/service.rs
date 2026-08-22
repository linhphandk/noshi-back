use crate::shared::errors::{AppError, AppResult};
use crate::waitlist::models::{NewWaitlistEntry, WaitlistResponse};
use crate::waitlist::ports::WaitlistRepository;
use tracing::{debug, instrument};

#[derive(Clone)]
pub struct WaitlistService<R: WaitlistRepository> {
    repo: R,
}

impl<R: WaitlistRepository> WaitlistService<R> {
    pub fn new(repo: R) -> Self {
        Self { repo }
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
        self.repo.create(NewWaitlistEntry { email }).await?;

        Ok(WaitlistResponse {
            message: "You're on the waitlist".to_string(),
        })
    }
}
