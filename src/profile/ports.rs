use async_trait::async_trait;
use uuid::Uuid;

use crate::profile::models::{
    ManualPlatform, NewManualPlatform, NewProfile, Profile, UpdateManualPlatform, UpdateProfile,
};
use crate::shared::errors::AppResult;

#[async_trait]
pub trait ProfileRepository: Send + Sync {
    async fn create(&self, profile: NewProfile) -> AppResult<Profile>;
    async fn find_by_id(&self, id: Uuid) -> AppResult<Option<Profile>>;
    async fn find_by_user_id(&self, user_id: Uuid) -> AppResult<Option<Profile>>;
    async fn find_by_slug(&self, slug: &str) -> AppResult<Option<Profile>>;
    async fn update(&self, id: Uuid, profile: UpdateProfile) -> AppResult<Profile>;
    async fn delete(&self, id: Uuid) -> AppResult<()>;
}

#[async_trait]
pub trait ManualPlatformRepository: Send + Sync {
    async fn create(&self, platform: NewManualPlatform) -> AppResult<ManualPlatform>;
    async fn find_by_id(&self, id: Uuid) -> AppResult<Option<ManualPlatform>>;
    async fn find_by_user_id(&self, user_id: Uuid) -> AppResult<Vec<ManualPlatform>>;
    async fn update(&self, id: Uuid, platform: UpdateManualPlatform) -> AppResult<ManualPlatform>;
    async fn delete(&self, id: Uuid) -> AppResult<()>;
}
