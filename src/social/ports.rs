use async_trait::async_trait;
use uuid::Uuid;

use crate::shared::errors::AppResult;
use crate::social::models::{
    NewSocialConnection, SocialConnection, SocialInsights, SocialProfile, SocialTokens,
    UpdateSocialConnection,
};

#[async_trait]
pub trait SocialConnectionRepository: Send + Sync {
    async fn create(&self, connection: NewSocialConnection) -> AppResult<SocialConnection>;
    async fn find_by_id(&self, id: Uuid) -> AppResult<Option<SocialConnection>>;
    async fn find_by_user_id(&self, user_id: Uuid) -> AppResult<Vec<SocialConnection>>;
    async fn find_by_user_and_platform(
        &self,
        user_id: Uuid,
        platform: &str,
    ) -> AppResult<Option<SocialConnection>>;
    async fn update(
        &self,
        id: Uuid,
        connection: UpdateSocialConnection,
    ) -> AppResult<SocialConnection>;
    async fn update_demographics(
        &self,
        id: Uuid,
        demographics: Option<serde_json::Value>,
    ) -> AppResult<SocialConnection>;
    async fn delete(&self, id: Uuid) -> AppResult<()>;
}

#[async_trait]
pub trait SocialProvider: Send + Sync {
    fn platform(&self) -> &str;
    fn authorize_url(&self, state: &str) -> String;
    async fn exchange_code(&self, code: &str) -> AppResult<SocialTokens>;
    async fn refresh_token(&self, token: &str) -> AppResult<SocialTokens>;
    async fn fetch_profile(&self, token: &str) -> AppResult<SocialProfile>;
    async fn fetch_insights(
        &self,
        token: &str,
        platform_user_id: &str,
    ) -> AppResult<SocialInsights>;
}
