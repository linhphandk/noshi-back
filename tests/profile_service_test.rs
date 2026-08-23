use chrono::Utc;
use mockall::mock;
use uuid::Uuid;

use noshi_back::profile::models::{
    CreateManualPlatformRequest, CreateProfileRequest, ManualPlatform, NewManualPlatform,
    NewProfile, Profile, UpdateManualPlatform, UpdateProfile,
};
use noshi_back::profile::ports::{ManualPlatformRepository, ProfileRepository};
use noshi_back::profile::service::ProfileService;
use noshi_back::shared::errors::{AppError, AppResult};

mock! {
    pub TestProfileRepository {}
    #[async_trait::async_trait]
    impl ProfileRepository for TestProfileRepository {
        async fn create(&self, profile: NewProfile) -> AppResult<Profile>;
        async fn find_by_id(&self, id: Uuid) -> AppResult<Option<Profile>>;
        async fn find_by_user_id(&self, user_id: Uuid) -> AppResult<Option<Profile>>;
        async fn find_by_slug(&self, slug: &str) -> AppResult<Option<Profile>>;
        async fn update(&self, id: Uuid, profile: UpdateProfile) -> AppResult<Profile>;
        async fn delete(&self, id: Uuid) -> AppResult<()>;
    }
}

mock! {
    pub TestManualPlatformRepository {}
    #[async_trait::async_trait]
    impl ManualPlatformRepository for TestManualPlatformRepository {
        async fn create(&self, platform: NewManualPlatform) -> AppResult<ManualPlatform>;
        async fn find_by_id(&self, id: Uuid) -> AppResult<Option<ManualPlatform>>;
        async fn find_by_user_id(&self, user_id: Uuid) -> AppResult<Vec<ManualPlatform>>;
        async fn find_by_user_and_platform(
            &self,
            user_id: Uuid,
            platform: &str,
        ) -> AppResult<Option<ManualPlatform>>;
        async fn update(&self, id: Uuid, platform: UpdateManualPlatform) -> AppResult<ManualPlatform>;
        async fn delete(&self, id: Uuid) -> AppResult<()>;
    }
}

fn make_profile(user_id: Uuid) -> Profile {
    Profile {
        id: Uuid::now_v7(),
        user_id,
        slug: "test-user".to_string(),
        niches: vec![Some("tech".to_string())],
        headline: "Tech Influencer".to_string(),
        is_published: false,
        completion_score: 50,
        created_at: Utc::now().naive_utc(),
        updated_at: Utc::now().naive_utc(),
    }
}

fn make_platform(user_id: Uuid) -> ManualPlatform {
    ManualPlatform {
        id: Uuid::now_v7(),
        user_id,
        platform: "instagram".to_string(),
        handle: "@testuser".to_string(),
        follower_count: 10000,
        created_at: Utc::now().naive_utc(),
        updated_at: Utc::now().naive_utc(),
    }
}

#[tokio::test]
async fn test_create_profile() {
    let mut profile_repo = MockTestProfileRepository::default();
    let manual_platform_repo = MockTestManualPlatformRepository::default();
    let user_id = Uuid::now_v7();
    let uid = user_id;
    profile_repo
        .expect_find_by_user_id()
        .returning(move |_| Ok(None));
    profile_repo.expect_find_by_slug().returning(|_| Ok(None));
    profile_repo
        .expect_create()
        .returning(move |_| Ok(make_profile(uid)));
    let service = ProfileService::new(profile_repo, manual_platform_repo);
    let result = service
        .create_profile(
            user_id,
            CreateProfileRequest {
                slug: "test-user".to_string(),
                niches: vec!["tech".to_string()],
                headline: "Tech Influencer".to_string(),
                is_published: None,
            },
        )
        .await;
    assert!(result.is_ok());
}

#[tokio::test]
async fn test_create_profile_duplicate() {
    let mut profile_repo = MockTestProfileRepository::default();
    let manual_platform_repo = MockTestManualPlatformRepository::default();
    let user_id = Uuid::now_v7();
    profile_repo
        .expect_find_by_user_id()
        .returning(move |_| Ok(Some(make_profile(user_id))));
    let service = ProfileService::new(profile_repo, manual_platform_repo);
    let result = service
        .create_profile(
            user_id,
            CreateProfileRequest {
                slug: "test-user".to_string(),
                niches: vec![],
                headline: "Hi".to_string(),
                is_published: None,
            },
        )
        .await;
    assert!(matches!(result, Err(AppError::Conflict(_))));
}

#[tokio::test]
async fn test_get_profile_not_found() {
    let mut profile_repo = MockTestProfileRepository::default();
    let manual_platform_repo = MockTestManualPlatformRepository::default();
    let user_id = Uuid::now_v7();
    profile_repo
        .expect_find_by_user_id()
        .returning(|_| Ok(None));
    let service = ProfileService::new(profile_repo, manual_platform_repo);
    let result = service.get_profile(user_id).await;
    assert!(matches!(result, Err(AppError::NotFound(_))));
}

#[tokio::test]
async fn test_get_profile() {
    let mut profile_repo = MockTestProfileRepository::default();
    let manual_platform_repo = MockTestManualPlatformRepository::default();
    let user_id = Uuid::now_v7();
    profile_repo
        .expect_find_by_user_id()
        .returning(move |_| Ok(Some(make_profile(user_id))));
    let service = ProfileService::new(profile_repo, manual_platform_repo);
    assert!(service.get_profile(user_id).await.is_ok());
}

#[tokio::test]
async fn test_delete_profile() {
    let mut profile_repo = MockTestProfileRepository::default();
    let manual_platform_repo = MockTestManualPlatformRepository::default();
    let user_id = Uuid::now_v7();
    profile_repo
        .expect_find_by_user_id()
        .returning(move |_| Ok(Some(make_profile(user_id))));
    profile_repo.expect_delete().returning(|_| Ok(()));
    let service = ProfileService::new(profile_repo, manual_platform_repo);
    assert!(service.delete_profile(user_id).await.is_ok());
}

#[tokio::test]
async fn test_add_manual_platform() {
    let mut profile_repo = MockTestProfileRepository::default();
    let mut manual_platform_repo = MockTestManualPlatformRepository::default();
    let user_id = Uuid::now_v7();
    profile_repo
        .expect_find_by_user_id()
        .returning(move |_| Ok(Some(make_profile(user_id))));
    manual_platform_repo
        .expect_find_by_user_and_platform()
        .returning(|_, _| Ok(None));
    manual_platform_repo
        .expect_create()
        .returning(move |_| Ok(make_platform(user_id)));
    let service = ProfileService::new(profile_repo, manual_platform_repo);
    let result = service
        .add_manual_platform(
            user_id,
            CreateManualPlatformRequest {
                platform: "instagram".to_string(),
                handle: "@test".to_string(),
                follower_count: 1000,
            },
        )
        .await;
    assert!(result.is_ok());
}

#[tokio::test]
async fn test_add_manual_platform_duplicate() {
    let mut profile_repo = MockTestProfileRepository::default();
    let mut manual_platform_repo = MockTestManualPlatformRepository::default();
    let user_id = Uuid::now_v7();
    profile_repo
        .expect_find_by_user_id()
        .returning(move |_| Ok(Some(make_profile(user_id))));
    manual_platform_repo
        .expect_find_by_user_and_platform()
        .returning(move |_, _| Ok(Some(make_platform(user_id))));
    let service = ProfileService::new(profile_repo, manual_platform_repo);
    let result = service
        .add_manual_platform(
            user_id,
            CreateManualPlatformRequest {
                platform: "instagram".to_string(),
                handle: "@test".to_string(),
                follower_count: 1000,
            },
        )
        .await;
    assert!(matches!(result, Err(AppError::Conflict(_))));
}

#[tokio::test]
async fn test_delete_manual_platform_wrong_user() {
    let mut profile_repo = MockTestProfileRepository::default();
    let mut manual_platform_repo = MockTestManualPlatformRepository::default();
    let user_id = Uuid::now_v7();
    let other_user = Uuid::now_v7();
    profile_repo
        .expect_find_by_user_id()
        .returning(move |_| Ok(Some(make_profile(user_id))));
    manual_platform_repo
        .expect_find_by_id()
        .returning(move |_| Ok(Some(make_platform(other_user))));
    let service = ProfileService::new(profile_repo, manual_platform_repo);
    let result = service
        .delete_manual_platform(user_id, Uuid::now_v7())
        .await;
    assert!(matches!(result, Err(AppError::Forbidden(_))));
}
