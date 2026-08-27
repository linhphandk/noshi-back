use mockall::mock;
use noshi_back::shared::errors::{AppError, AppResult};
use noshi_back::waitlist::models::{NewWaitlistEntry, WaitlistEntry};
use noshi_back::waitlist::ports::WaitlistRepository;
use noshi_back::waitlist::service::WaitlistService;
use uuid::Uuid;

mock! {
    pub TestWaitlistRepository {}

    #[async_trait::async_trait]
    impl WaitlistRepository for TestWaitlistRepository {
        async fn create(&self, entry: NewWaitlistEntry) -> AppResult<WaitlistEntry>;
        async fn find_by_email(&self, email: &str) -> AppResult<Option<WaitlistEntry>>;
    }
}

fn make_entry(email: &str) -> WaitlistEntry {
    WaitlistEntry {
        id: Uuid::now_v7(),
        email: email.to_string(),
        signed_up_at: chrono::Utc::now().naive_utc(),
    }
}

#[tokio::test]
async fn test_join_success() {
    let mut repo = MockTestWaitlistRepository::default();
    repo.expect_find_by_email().returning(|_| Ok(None));
    repo.expect_create().returning(|e| Ok(make_entry(&e.email)));

    let service = WaitlistService::new(repo);
    let result = service.join("user@test.com".to_string()).await;

    assert!(result.is_ok());
    assert_eq!(result.unwrap().message, "You're on the waitlist");
}

#[tokio::test]
async fn test_join_duplicate_returns_conflict() {
    let mut repo = MockTestWaitlistRepository::default();
    repo.expect_find_by_email()
        .returning(|_| Ok(Some(make_entry("dup@test.com"))));

    let service = WaitlistService::new(repo);
    let result = service.join("dup@test.com".to_string()).await;

    assert!(matches!(result, Err(AppError::Conflict(_))));
}

#[tokio::test]
async fn test_join_empty_email_returns_bad_request() {
    let repo = MockTestWaitlistRepository::default();
    let service = WaitlistService::new(repo);
    let result = service.join("".to_string()).await;

    assert!(matches!(result, Err(AppError::BadRequest(_))));
}

#[tokio::test]
async fn test_join_no_at_symbol_returns_bad_request() {
    let repo = MockTestWaitlistRepository::default();
    let service = WaitlistService::new(repo);
    let result = service.join("notanemail".to_string()).await;

    assert!(matches!(result, Err(AppError::BadRequest(_))));
}

#[tokio::test]
async fn test_join_repo_error_propagates() {
    let mut repo = MockTestWaitlistRepository::default();
    repo.expect_find_by_email()
        .returning(|_| Err(AppError::Internal));

    let service = WaitlistService::new(repo);
    let result = service.join("user@test.com".to_string()).await;

    assert!(matches!(result, Err(AppError::Internal)));
}
