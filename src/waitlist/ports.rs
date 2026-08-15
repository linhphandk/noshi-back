use crate::shared::errors::AppResult;
use crate::waitlist::models::{NewWaitlistEntry, WaitlistEntry};
use async_trait::async_trait;

#[async_trait]
pub trait WaitlistRepository: Send + Sync {
    async fn create(&self, entry: NewWaitlistEntry) -> AppResult<WaitlistEntry>;
    async fn find_by_email(&self, email: &str) -> AppResult<Option<WaitlistEntry>>;
}
