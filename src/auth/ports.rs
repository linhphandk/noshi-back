use async_trait::async_trait;
use chrono::NaiveDateTime;
use uuid::Uuid;

use crate::auth::models::{NewPasswordReset, NewUser, PasswordReset, Tokens, User, UserInfo};
use crate::shared::errors::AppResult;

#[async_trait]
pub trait AuthProvider: Send + Sync {
    async fn register(
        &self,
        email: &str,
        name: &str,
        password: &str,
    ) -> AppResult<(UserInfo, Tokens)>;

    async fn login(
        &self,
        email: &str,
        password: &str,
        password_hash: &str,
    ) -> AppResult<(Tokens, UserInfo)>;

    async fn introspect_token(&self, token: &str) -> AppResult<UserInfo>;

    async fn generate_access_token(&self, user_id: &str, email: &str) -> AppResult<String>;

    fn token_expiry_seconds(&self) -> u64;

    fn jwt_secret(&self) -> &str;
}

#[async_trait]
pub trait UserRepository: Send + Sync {
    async fn create(&self, user: NewUser) -> AppResult<User>;
    async fn find_by_email(&self, email: &str) -> AppResult<Option<User>>;
    async fn find_by_id(&self, id: Uuid) -> AppResult<Option<User>>;
    async fn update_password_hash(&self, id: Uuid, password_hash: String) -> AppResult<()>;
}

#[async_trait]
pub trait SessionRepository: Send + Sync {
    async fn create(
        &self,
        user_id: Uuid,
        refresh_token: String,
        expires_at: NaiveDateTime,
    ) -> AppResult<()>;
    async fn find_by_token(&self, refresh_token: &str) -> AppResult<Option<SessionInfo>>;
    async fn revoke(&self, refresh_token: &str) -> AppResult<()>;
    async fn revoke_all_for_user(&self, user_id: Uuid) -> AppResult<()>;
}

#[derive(Debug, Clone)]
pub struct SessionInfo {
    pub user_id: Uuid,
    pub expires_at: NaiveDateTime,
}

#[async_trait]
pub trait PasswordResetRepository: Send + Sync {
    async fn create(&self, reset: NewPasswordReset) -> AppResult<PasswordReset>;
    async fn find_by_token_hash(&self, token_hash: &str) -> AppResult<Option<PasswordReset>>;
    async fn mark_used(&self, id: Uuid) -> AppResult<()>;
}
