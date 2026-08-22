use chrono::Utc;
use mockall::mock;
use uuid::Uuid;

use noshi_back::auth::email::EmailProvider;
use noshi_back::auth::models::{NewPasswordReset, NewUser, PasswordReset, Tokens, User, UserInfo};
use noshi_back::auth::ports::{
    AuthProvider, PasswordResetRepository, SessionInfo, SessionRepository, UserRepository,
};
use noshi_back::auth::service::AuthService;
use noshi_back::shared::errors::{AppError, AppResult};

mock! {
    pub TestAuthProvider {}

    #[async_trait::async_trait]
    impl AuthProvider for TestAuthProvider {
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
    }
}

mock! {
    pub TestUserRepository {}

    #[async_trait::async_trait]
    impl UserRepository for TestUserRepository {
        async fn create(&self, user: NewUser) -> AppResult<User>;
        async fn find_by_email(&self, email: &str) -> AppResult<Option<User>>;
        async fn find_by_id(&self, id: Uuid) -> AppResult<Option<User>>;
        async fn update_password_hash(&self, id: Uuid, password_hash: String) -> AppResult<()>;
    }
}

mock! {
    pub TestSessionRepository {}

    #[async_trait::async_trait]
    impl SessionRepository for TestSessionRepository {
        async fn create(
            &self,
            user_id: Uuid,
            refresh_token: String,
            expires_at: chrono::NaiveDateTime,
        ) -> AppResult<()>;

        async fn find_by_token(&self, refresh_token: &str) -> AppResult<Option<SessionInfo>>;

        async fn revoke(&self, refresh_token: &str) -> AppResult<()>;

        async fn revoke_all_for_user(&self, user_id: Uuid) -> AppResult<()>;
    }
}

mock! {
    pub TestPasswordResetRepository {}

    #[async_trait::async_trait]
    impl PasswordResetRepository for TestPasswordResetRepository {
        async fn create(&self, reset: NewPasswordReset) -> AppResult<PasswordReset>;
        async fn find_by_token_hash(&self, token_hash: &str) -> AppResult<Option<PasswordReset>>;
        async fn mark_used(&self, id: Uuid) -> AppResult<()>;
    }
}

mock! {
    pub TestEmailProvider {}

    #[async_trait::async_trait]
    impl EmailProvider for TestEmailProvider {
        async fn send_password_reset(&self, to: &str, reset_url: &str) -> AppResult<()>;
    }
}

fn make_user() -> User {
    User {
        id: Uuid::now_v7(),
        email: "test@test.com".to_string(),
        name: "Test User".to_string(),
        password_hash: "hash".to_string(),
        created_at: Utc::now().naive_utc(),
        updated_at: Utc::now().naive_utc(),
    }
}

#[tokio::test]
async fn test_register_success() {
    let mut auth_provider = MockTestAuthProvider::default();
    let mut user_repo = MockTestUserRepository::default();
    let mut session_repo = MockTestSessionRepository::default();
    let password_reset_repo = MockTestPasswordResetRepository::default();
    let email_provider = MockTestEmailProvider::default();

    auth_provider.expect_register().returning(|_, _, _| {
        Ok((
            UserInfo {
                sub: "1".to_string(),
                email: "test@test.com".to_string(),
                name: "Test User".to_string(),
                password_hash: "hash".to_string(),
            },
            Tokens {
                access_token: "token".to_string(),
                expires_in: 900,
                refresh_token: None,
            },
        ))
    });

    user_repo.expect_find_by_email().returning(|_| Ok(None));
    user_repo
        .expect_create()
        .returning(move |_| Ok(make_user()));
    session_repo.expect_create().returning(|_, _, _| Ok(()));

    let service = AuthService::new(
        auth_provider,
        user_repo,
        session_repo,
        password_reset_repo,
        email_provider,
        7,
        "http://localhost:5173".to_string(),
    );

    let result = service
        .register(
            "test@test.com".to_string(),
            "Test User".to_string(),
            "password123".to_string(),
        )
        .await;

    assert!(result.is_ok());
}

#[tokio::test]
async fn test_register_duplicate_email() {
    let auth_provider = MockTestAuthProvider::default();
    let mut user_repo = MockTestUserRepository::default();
    let session_repo = MockTestSessionRepository::default();
    let password_reset_repo = MockTestPasswordResetRepository::default();
    let email_provider = MockTestEmailProvider::default();

    user_repo
        .expect_find_by_email()
        .returning(move |_| Ok(Some(make_user())));

    let service = AuthService::new(
        auth_provider,
        user_repo,
        session_repo,
        password_reset_repo,
        email_provider,
        7,
        "http://localhost:5173".to_string(),
    );

    let result = service
        .register(
            "test@test.com".to_string(),
            "Test User".to_string(),
            "password123".to_string(),
        )
        .await;

    assert!(matches!(result, Err(AppError::Conflict(_))));
}

#[tokio::test]
async fn test_register_short_password() {
    let auth_provider = MockTestAuthProvider::default();
    let user_repo = MockTestUserRepository::default();
    let session_repo = MockTestSessionRepository::default();
    let password_reset_repo = MockTestPasswordResetRepository::default();
    let email_provider = MockTestEmailProvider::default();

    let service = AuthService::new(
        auth_provider,
        user_repo,
        session_repo,
        password_reset_repo,
        email_provider,
        7,
        "http://localhost:5173".to_string(),
    );

    let result = service
        .register(
            "test@test.com".to_string(),
            "Test User".to_string(),
            "short".to_string(),
        )
        .await;

    assert!(matches!(result, Err(AppError::BadRequest(_))));
}

#[tokio::test]
async fn test_login_success() {
    let mut auth_provider = MockTestAuthProvider::default();
    let mut user_repo = MockTestUserRepository::default();
    let mut session_repo = MockTestSessionRepository::default();
    let password_reset_repo = MockTestPasswordResetRepository::default();
    let email_provider = MockTestEmailProvider::default();

    user_repo
        .expect_find_by_email()
        .returning(move |_| Ok(Some(make_user())));

    auth_provider.expect_login().returning(|_, _, _| {
        Ok((
            Tokens {
                access_token: "token".to_string(),
                expires_in: 900,
                refresh_token: None,
            },
            UserInfo {
                sub: "1".to_string(),
                email: "test@test.com".to_string(),
                name: "Test User".to_string(),
                password_hash: "hash".to_string(),
            },
        ))
    });

    session_repo.expect_create().returning(|_, _, _| Ok(()));

    let service = AuthService::new(
        auth_provider,
        user_repo,
        session_repo,
        password_reset_repo,
        email_provider,
        7,
        "http://localhost:5173".to_string(),
    );

    let result = service
        .login("test@test.com".to_string(), "password123".to_string())
        .await;

    assert!(result.is_ok());
}

#[tokio::test]
async fn test_login_wrong_password() {
    let mut auth_provider = MockTestAuthProvider::default();
    let mut user_repo = MockTestUserRepository::default();
    let session_repo = MockTestSessionRepository::default();
    let password_reset_repo = MockTestPasswordResetRepository::default();
    let email_provider = MockTestEmailProvider::default();

    user_repo
        .expect_find_by_email()
        .returning(move |_| Ok(Some(make_user())));

    auth_provider.expect_login().returning(|_, _, _| {
        Err(AppError::BadRequest(
            "Invalid email or password".to_string(),
        ))
    });

    let service = AuthService::new(
        auth_provider,
        user_repo,
        session_repo,
        password_reset_repo,
        email_provider,
        7,
        "http://localhost:5173".to_string(),
    );

    let result = service
        .login("test@test.com".to_string(), "wrongpassword".to_string())
        .await;

    assert!(matches!(result, Err(AppError::BadRequest(_))));
}

#[tokio::test]
async fn test_forgot_password_sends_email() {
    let auth_provider = MockTestAuthProvider::default();
    let mut user_repo = MockTestUserRepository::default();
    let mut password_reset_repo = MockTestPasswordResetRepository::default();
    let session_repo = MockTestSessionRepository::default();
    let mut email_provider = MockTestEmailProvider::default();

    user_repo
        .expect_find_by_email()
        .returning(move |_| Ok(Some(make_user())));

    password_reset_repo.expect_create().returning(|_| {
        Ok(PasswordReset {
            id: Uuid::now_v7(),
            user_id: Uuid::now_v7(),
            token_hash: "hash".to_string(),
            expires_at: Utc::now().naive_utc(),
            used_at: None,
            created_at: Utc::now().naive_utc(),
        })
    });

    email_provider
        .expect_send_password_reset()
        .returning(|_, _| Ok(()));

    let service = AuthService::new(
        auth_provider,
        user_repo,
        session_repo,
        password_reset_repo,
        email_provider,
        7,
        "http://localhost:5173".to_string(),
    );

    let result = service.forgot_password("test@test.com".to_string()).await;

    assert!(result.is_ok());
}

#[tokio::test]
async fn test_forgot_password_unknown_email_returns_ok() {
    let auth_provider = MockTestAuthProvider::default();
    let mut user_repo = MockTestUserRepository::default();
    let session_repo = MockTestSessionRepository::default();
    let password_reset_repo = MockTestPasswordResetRepository::default();
    let email_provider = MockTestEmailProvider::default();

    user_repo.expect_find_by_email().returning(|_| Ok(None));

    let service = AuthService::new(
        auth_provider,
        user_repo,
        session_repo,
        password_reset_repo,
        email_provider,
        7,
        "http://localhost:5173".to_string(),
    );

    let result = service
        .forgot_password("unknown@test.com".to_string())
        .await;

    assert!(result.is_ok());
}

#[tokio::test]
async fn test_get_current_user() {
    let mut auth_provider = MockTestAuthProvider::default();
    let mut user_repo = MockTestUserRepository::default();
    let session_repo = MockTestSessionRepository::default();
    let password_reset_repo = MockTestPasswordResetRepository::default();
    let email_provider = MockTestEmailProvider::default();

    auth_provider.expect_introspect_token().returning(|_| {
        Ok(UserInfo {
            sub: "1".to_string(),
            email: "test@test.com".to_string(),
            name: "Test User".to_string(),
            password_hash: "hash".to_string(),
        })
    });

    user_repo
        .expect_find_by_email()
        .returning(move |_| Ok(Some(make_user())));

    let service = AuthService::new(
        auth_provider,
        user_repo,
        session_repo,
        password_reset_repo,
        email_provider,
        7,
        "http://localhost:5173".to_string(),
    );

    let result = service.get_current_user("valid-token".to_string()).await;

    assert!(result.is_ok());
}

#[tokio::test]
async fn test_get_current_user_invalid_token() {
    let mut auth_provider = MockTestAuthProvider::default();
    let user_repo = MockTestUserRepository::default();
    let session_repo = MockTestSessionRepository::default();
    let password_reset_repo = MockTestPasswordResetRepository::default();
    let email_provider = MockTestEmailProvider::default();

    auth_provider
        .expect_introspect_token()
        .returning(|_| Err(AppError::BadRequest("Invalid token".to_string())));

    let service = AuthService::new(
        auth_provider,
        user_repo,
        session_repo,
        password_reset_repo,
        email_provider,
        7,
        "http://localhost:5173".to_string(),
    );

    let result = service.get_current_user("invalid-token".to_string()).await;

    assert!(matches!(result, Err(AppError::BadRequest(_))));
}
