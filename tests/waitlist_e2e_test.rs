use axum::body::{to_bytes, Body};
use axum::http::{Request, StatusCode};
use noshi_back::router::create_router;
use noshi_back::shared::config::Config;
use noshi_back::shared::db::{establish_pool, run_migrations};
use noshi_back::state::AppState;
use serial_test::serial;
use std::sync::OnceLock;
use testcontainers::clients::Cli;
use testcontainers_modules::postgres::Postgres;
use tower::ServiceExt;

fn docker() -> &'static Cli {
    static DOCKER: OnceLock<Cli> = OnceLock::new();
    DOCKER.get_or_init(Cli::default)
}

struct TestApp {
    _container: testcontainers::Container<'static, Postgres>,
    router: axum::Router,
}

async fn setup() -> TestApp {
    let _ = tracing_subscriber::fmt()
        .with_env_filter("debug")
        .try_init();
    let container = docker().run(Postgres::default());
    let port = container.get_host_port_ipv4(5432);
    let db_url = format!("postgres://postgres:postgres@localhost:{}/postgres", port);
    let pool = establish_pool(&db_url, 4);
    run_migrations(&pool);

    let config = Config {
        server_host: "127.0.0.1".to_string(),
        server_port: 0,
        database_url: db_url,
        database_pool_size: 4,
        jwt_secret: "test-secret".to_string(),
        jwt_expiry_minutes: 15,
        s3_bucket: None,
        s3_region: None,
        s3_endpoint: None,
        s3_public_url: None,
        smtp_host: None,
        smtp_port: None,
        smtp_user: None,
        smtp_password: None,
        smtp_from_email: None,
        smtp_from_name: None,
        frontend_url: Some("http://localhost:5173".to_string()),
        instagram_client_id: None,
        instagram_client_secret: None,
        instagram_redirect_uri: None,
        token_encryption_key: Some("JDbUt2tSDFf6lHd3tYgqHWLyySvjrcxO2USr9Ozyh5k=".to_string()),
        social_oauth_state_secret: None,
    };

    let state = AppState::new(
        pool,
        "test-secret".to_string(),
        15,
        "http://localhost:5173".to_string(),
        &config,
    )
    .await
    .unwrap();

    TestApp {
        _container: container,
        router: create_router(state),
    }
}

async fn post_waitlist(app: &TestApp, body: &str) -> (StatusCode, String) {
    let resp = app
        .router
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/waitlist")
                .header("content-type", "application/json")
                .body(Body::from(body.to_string()))
                .unwrap(),
        )
        .await
        .unwrap();

    let status = resp.status();
    let bytes = to_bytes(resp.into_body(), 1024).await.unwrap();
    (status, String::from_utf8_lossy(&bytes).to_string())
}

#[tokio::test]
#[serial]
async fn test_e2e_waitlist_success() {
    let app = setup().await;
    let email = format!("e2e-{}@test.com", uuid::Uuid::now_v7());

    let (status, body) = post_waitlist(&app, &format!(r#"{{"email":"{}"}}"#, email)).await;
    assert_eq!(status, StatusCode::CREATED);
    assert!(body.contains("waitlist"));
}

#[tokio::test]
#[serial]
async fn test_e2e_waitlist_duplicate() {
    let app = setup().await;
    let email = format!("dup-{}@test.com", uuid::Uuid::now_v7());

    let (status, _) = post_waitlist(&app, &format!(r#"{{"email":"{}"}}"#, email)).await;
    assert_eq!(status, StatusCode::CREATED);

    let (status, body) = post_waitlist(&app, &format!(r#"{{"email":"{}"}}"#, email)).await;
    assert_eq!(status, StatusCode::CONFLICT);
    assert!(body.contains("already"));
}

#[tokio::test]
#[serial]
async fn test_e2e_waitlist_invalid_email() {
    let app = setup().await;

    let (status, body) = post_waitlist(&app, r#"{"email":"no-at-sign"}"#).await;
    assert_eq!(status, StatusCode::BAD_REQUEST);
    assert!(body.contains("Invalid"));
}

#[tokio::test]
#[serial]
async fn test_e2e_waitlist_empty_email() {
    let app = setup().await;

    let (status, _) = post_waitlist(&app, r#"{"email":""}"#).await;
    assert_eq!(status, StatusCode::BAD_REQUEST);
}
