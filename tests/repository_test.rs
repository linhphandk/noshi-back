use chrono::{Duration, Utc};
use noshi_back::auth::models::{NewPasswordReset, NewUser};
use noshi_back::auth::ports::{PasswordResetRepository, SessionRepository, UserRepository};
use noshi_back::auth::repository::{
    DieselPasswordResetRepository, DieselSessionRepository, DieselUserRepository,
};
use noshi_back::shared::db::{establish_pool, run_migrations};
use std::sync::OnceLock;
use testcontainers::clients::Cli;
use testcontainers_modules::postgres::Postgres;
use uuid::Uuid;

fn docker() -> &'static Cli {
    static DOCKER: OnceLock<Cli> = OnceLock::new();
    DOCKER.get_or_init(Cli::default)
}

struct TestDb {
    _container: testcontainers::Container<'static, Postgres>,
    user_repo: DieselUserRepository,
    session_repo: DieselSessionRepository,
    password_reset_repo: DieselPasswordResetRepository,
}

fn setup() -> TestDb {
    let container = docker().run(Postgres::default());
    let port = container.get_host_port_ipv4(5432);
    let db_url = format!("postgres://postgres:postgres@localhost:{}/postgres", port);
    let pool = establish_pool(&db_url, 4);
    run_migrations(&pool);
    TestDb {
        _container: container,
        user_repo: DieselUserRepository::new(pool.clone()),
        session_repo: DieselSessionRepository::new(pool.clone()),
        password_reset_repo: DieselPasswordResetRepository::new(pool),
    }
}

async fn create_test_user(db: &TestDb) -> Uuid {
    let user_id = Uuid::now_v7();
    db.user_repo
        .create(NewUser {
            id: user_id,
            email: format!("user-{}@example.com", user_id),
            name: "Test User".to_string(),
            password_hash: "hashed_password".to_string(),
        })
        .await
        .unwrap();
    user_id
}

#[tokio::test]
async fn test_user_create_and_find() {
    let db = setup();
    let user_id = Uuid::now_v7();
    let new_user = NewUser {
        id: user_id,
        email: "test@example.com".to_string(),
        name: "Test User".to_string(),
        password_hash: "hashed_password".to_string(),
    };

    let user = db.user_repo.create(new_user).await.unwrap();
    assert_eq!(user.email, "test@example.com");
    assert_eq!(user.name, "Test User");

    let found = db
        .user_repo
        .find_by_email("test@example.com")
        .await
        .unwrap();
    assert!(found.is_some());
    assert_eq!(found.unwrap().id, user_id);

    let found_by_id = db.user_repo.find_by_id(user_id).await.unwrap();
    assert!(found_by_id.is_some());
}

#[tokio::test]
async fn test_user_find_nonexistent() {
    let db = setup();
    let found = db
        .user_repo
        .find_by_email("nonexistent@example.com")
        .await
        .unwrap();
    assert!(found.is_none());
}

#[tokio::test]
async fn test_user_update_password_hash() {
    let db = setup();
    let user_id = create_test_user(&db).await;
    db.user_repo
        .update_password_hash(user_id, "new_hash".to_string())
        .await
        .unwrap();

    let user = db.user_repo.find_by_id(user_id).await.unwrap().unwrap();
    assert_eq!(user.password_hash, "new_hash");
}

#[tokio::test]
async fn test_user_update_password_nonexistent() {
    let db = setup();
    let result = db
        .user_repo
        .update_password_hash(Uuid::now_v7(), "hash".to_string())
        .await;
    assert!(result.is_err());
}

#[tokio::test]
async fn test_session_create_and_find() {
    let db = setup();
    let user_id = create_test_user(&db).await;
    let refresh_token = "test-refresh-token".to_string();
    let expires_at = Utc::now()
        .checked_add_signed(Duration::days(7))
        .unwrap()
        .naive_utc();

    db.session_repo
        .create(user_id, refresh_token.clone(), expires_at)
        .await
        .unwrap();

    let found = db.session_repo.find_by_token(&refresh_token).await.unwrap();
    assert!(found.is_some());
    assert_eq!(found.unwrap().user_id, user_id);
}

#[tokio::test]
async fn test_session_revoke() {
    let db = setup();
    let user_id = create_test_user(&db).await;
    let refresh_token = "test-refresh-token".to_string();
    let expires_at = Utc::now()
        .checked_add_signed(Duration::days(7))
        .unwrap()
        .naive_utc();

    db.session_repo
        .create(user_id, refresh_token.clone(), expires_at)
        .await
        .unwrap();
    db.session_repo.revoke(&refresh_token).await.unwrap();

    let found = db.session_repo.find_by_token(&refresh_token).await.unwrap();
    assert!(found.is_none());
}

#[tokio::test]
async fn test_session_revoke_all_for_user() {
    let db = setup();
    let user_id = create_test_user(&db).await;
    let expires_at = Utc::now()
        .checked_add_signed(Duration::days(7))
        .unwrap()
        .naive_utc();

    db.session_repo
        .create(user_id, "token1".to_string(), expires_at)
        .await
        .unwrap();
    db.session_repo
        .create(user_id, "token2".to_string(), expires_at)
        .await
        .unwrap();

    db.session_repo.revoke_all_for_user(user_id).await.unwrap();

    assert!(db
        .session_repo
        .find_by_token("token1")
        .await
        .unwrap()
        .is_none());
    assert!(db
        .session_repo
        .find_by_token("token2")
        .await
        .unwrap()
        .is_none());
}

#[tokio::test]
async fn test_password_reset_create_and_find() {
    let db = setup();
    let user_id = create_test_user(&db).await;
    let token_hash = "test-token-hash".to_string();
    let expires_at = Utc::now()
        .checked_add_signed(Duration::hours(1))
        .unwrap()
        .naive_utc();

    let reset = db
        .password_reset_repo
        .create(NewPasswordReset {
            user_id,
            token_hash: token_hash.clone(),
            expires_at,
        })
        .await
        .unwrap();

    assert_eq!(reset.user_id, user_id);
    assert_eq!(reset.token_hash, token_hash);
    assert!(reset.used_at.is_none());

    let found = db
        .password_reset_repo
        .find_by_token_hash(&token_hash)
        .await
        .unwrap();
    assert!(found.is_some());
}

#[tokio::test]
async fn test_password_reset_mark_used() {
    let db = setup();
    let user_id = create_test_user(&db).await;
    let token_hash = "test-token-hash".to_string();
    let expires_at = Utc::now()
        .checked_add_signed(Duration::hours(1))
        .unwrap()
        .naive_utc();

    let reset = db
        .password_reset_repo
        .create(NewPasswordReset {
            user_id,
            token_hash,
            expires_at,
        })
        .await
        .unwrap();

    db.password_reset_repo.mark_used(reset.id).await.unwrap();

    let found = db
        .password_reset_repo
        .find_by_token_hash(&reset.token_hash)
        .await
        .unwrap()
        .unwrap();
    assert!(found.used_at.is_some());
}

#[tokio::test]
async fn test_password_reset_mark_used_nonexistent() {
    let db = setup();
    let result = db.password_reset_repo.mark_used(Uuid::now_v7()).await;
    assert!(result.is_err());
}
