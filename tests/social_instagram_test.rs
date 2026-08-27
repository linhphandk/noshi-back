use axum::extract::{Form, Query};
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use axum::routing::{get, post};
use axum::Router;
use noshi_back::shared::crypto;
use noshi_back::shared::errors::AppError;
use noshi_back::social::instagram::InstagramProvider;
use noshi_back::social::ports::SocialProvider;
use reqwest::Client;
use serde::Deserialize;
use serial_test::serial;
use std::time::Duration as StdDuration;

const TEST_USER_ID: &str = "17841405822304914";
const TEST_USERNAME: &str = "test_influencer";

struct TestEnv {
    provider: InstagramProvider,
    client: Client,
    encryption_key: [u8; 32],
    base_url: String,
}

fn init_tracing() {
    let _ = tracing_subscriber::fmt()
        .with_env_filter("debug")
        .try_init();
}

async fn start_mock_server() -> String {
    let app = Router::new()
        .route("/oauth/authorize", get(mock_authorize))
        .route("/oauth/access_token", post(mock_short_token))
        .route("/access_token", get(mock_long_token))
        .route("/refresh_access_token", get(mock_refresh_token))
        .route("/me", get(mock_profile))
        .route("/{user_id}/insights", get(mock_insights))
        .route("/{user_id}/media", get(mock_media_list))
        .route("/media/{media_id}/insights", get(mock_media_insights));

    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();
    tokio::spawn(async move {
        axum::serve(listener, app).await.unwrap();
    });
    tokio::time::sleep(StdDuration::from_millis(50)).await;
    format!("http://{}", addr)
}

async fn setup_test_env() -> TestEnv {
    init_tracing();
    let base_url = start_mock_server().await;
    let encryption_key =
        crypto::decode_key("JDbUt2tSDFf6lHd3tYgqHWLyySvjrcxO2USr9Ozyh5k=").unwrap();

    let provider = InstagramProvider::with_custom_urls(
        "test_client_id".to_string(),
        "test_client_secret".to_string(),
        "http://localhost:5173/oauth/instagram/callback".to_string(),
        format!("{}/oauth/authorize", base_url),
        format!("{}/oauth/access_token", base_url),
        format!("{}/access_token", base_url),
        format!("{}/refresh_access_token", base_url),
        base_url.clone(),
    );

    let client = Client::builder()
        .timeout(StdDuration::from_secs(10))
        .redirect(reqwest::redirect::Policy::none())
        .build()
        .unwrap();

    TestEnv {
        provider,
        client,
        encryption_key,
        base_url,
    }
}

#[derive(Deserialize)]
struct AuthorizeParams {
    state: String,
}

async fn mock_authorize(Query(params): Query<AuthorizeParams>) -> Response {
    let redirect = format!(
        "http://localhost:5173/oauth/instagram/callback?code=test_code_{}&state={}#_",
        params.state, params.state
    );
    (StatusCode::SEE_OTHER, [("location", redirect)]).into_response()
}

#[derive(Deserialize)]
struct ShortTokenParams {
    code: String,
}

async fn mock_short_token(Form(params): Form<ShortTokenParams>) -> Response {
    if !params.code.starts_with("test_code_") {
        return (StatusCode::BAD_REQUEST, "Invalid code").into_response();
    }
    axum::Json(serde_json::json!({
        "access_token": "short_test_token",
        "user_id": TEST_USER_ID,
    }))
    .into_response()
}

#[derive(Deserialize)]
struct LongTokenParams {
    access_token: String,
}

async fn mock_long_token(Query(params): Query<LongTokenParams>) -> Response {
    if params.access_token != "short_test_token" {
        return (StatusCode::BAD_REQUEST, "Invalid token").into_response();
    }
    axum::Json(serde_json::json!({
        "access_token": "long_test_token",
        "token_type": "bearer",
        "expires_in": 5184000,
    }))
    .into_response()
}

async fn mock_refresh_token(Query(params): Query<LongTokenParams>) -> Response {
    if !params.access_token.starts_with("long_") && !params.access_token.starts_with("refreshed_") {
        return (StatusCode::BAD_REQUEST, "Invalid token").into_response();
    }
    axum::Json(serde_json::json!({
        "access_token": "refreshed_test_token",
        "token_type": "bearer",
        "expires_in": 5184000,
    }))
    .into_response()
}

#[derive(Deserialize)]
struct ProfileParams {
    access_token: String,
}

async fn mock_profile(Query(params): Query<ProfileParams>) -> Response {
    if !params.access_token.starts_with("long_") && !params.access_token.starts_with("refreshed_") {
        return (StatusCode::UNAUTHORIZED, "Invalid token").into_response();
    }
    axum::Json(serde_json::json!({
        "username": TEST_USERNAME,
        "followers_count": 15000,
        "biography": "Test bio",
        "profile_picture_url": "https://example.com/pic.jpg",
        "media_count": 42,
    }))
    .into_response()
}

#[derive(Deserialize)]
struct InsightsParams {
    metric: String,
    access_token: String,
}

async fn mock_insights(Query(params): Query<InsightsParams>) -> Response {
    if !params.access_token.starts_with("long_") && !params.access_token.starts_with("refreshed_") {
        return (StatusCode::UNAUTHORIZED, "Invalid token").into_response();
    }

    if params.metric.contains("follower_demographics") {
        axum::Json(serde_json::json!({
            "data": [{
                "total_value": {
                    "value": 15000,
                    "breakdowns": [
                        {
                            "dimension_keys": ["age"],
                            "results": [
                                {"dimension_values": ["18-24"], "value": 3000},
                                {"dimension_values": ["25-34"], "value": 7000},
                            ]
                        },
                        {
                            "dimension_keys": ["gender"],
                            "results": [
                                {"dimension_values": ["male"], "value": 8000},
                                {"dimension_values": ["female"], "value": 7000},
                            ]
                        },
                        {
                            "dimension_keys": ["country"],
                            "results": [
                                {"dimension_values": ["US"], "value": 9000},
                                {"dimension_values": ["UK"], "value": 3000},
                            ]
                        }
                    ]
                }
            }]
        }))
        .into_response()
    } else {
        axum::Json(serde_json::json!({
            "data": [
                {"name": "reach", "total_value": {"value": 50000}},
                {"name": "accounts_engaged", "total_value": {"value": 3500}},
                {"name": "total_interactions", "total_value": {"value": 4200}},
            ]
        }))
        .into_response()
    }
}

async fn mock_media_list() -> impl IntoResponse {
    let media: Vec<serde_json::Value> = (0..5)
        .map(|i| {
            serde_json::json!({
                "id": format!("media_{}", i),
                "media_type": "IMAGE",
                "media_url": format!("https://example.com/media_{}.jpg", i),
            })
        })
        .collect();

    (
        StatusCode::OK,
        axum::Json(serde_json::json!({"data": media})),
    )
}

async fn mock_media_insights() -> impl IntoResponse {
    (
        StatusCode::OK,
        axum::Json(serde_json::json!({
            "data": [
                {"name": "total_interactions", "period": "lifetime", "values": [{"value": 500}]},
                {"name": "reach", "period": "lifetime", "values": [{"value": 2000}]},
                {"name": "views", "period": "lifetime", "values": [{"value": 1500}]},
            ]
        })),
    )
}

#[tokio::test]
#[serial]
async fn test_instagram_oauth_flow() {
    let env = setup_test_env().await;

    let state = "test_state_123";
    let authorize_url = env.provider.authorize_url(state);
    assert!(authorize_url.contains("/oauth/authorize"));
    assert!(authorize_url.contains("client_id=test_client_id"));
    assert!(authorize_url
        .contains("redirect_uri=http%3A%2F%2Flocalhost%3A5173%2Foauth%2Finstagram%2Fcallback"));
    assert!(authorize_url
        .contains("scope=instagram_business_basic%2Cinstagram_business_manage_insights"));
    assert!(authorize_url.contains(&format!("state={}", state)));

    let auth_resp = env
        .client
        .get(format!("{}/oauth/authorize", env.base_url))
        .query(&[
            ("client_id", "test_client_id"),
            (
                "redirect_uri",
                "http://localhost:5173/oauth/instagram/callback",
            ),
            ("response_type", "code"),
            (
                "scope",
                "instagram_business_basic,instagram_business_manage_insights",
            ),
            ("state", state),
        ])
        .send()
        .await
        .unwrap();

    assert_eq!(auth_resp.status(), 303);
    let location = auth_resp
        .headers()
        .get("location")
        .unwrap()
        .to_str()
        .unwrap();
    assert!(location.contains("code="));
    assert!(location.contains("state=test_state_123"));
    assert!(location.ends_with("#_"));

    let tokens = env.provider.exchange_code("test_code_123").await.unwrap();
    assert!(tokens.access_token.starts_with("long_"));
    assert_eq!(tokens.platform_user_id, TEST_USER_ID);
    assert!(tokens.refresh_token.is_some());
    assert_eq!(tokens.refresh_token.unwrap(), tokens.access_token);

    let profile = env
        .provider
        .fetch_profile(&tokens.access_token)
        .await
        .unwrap();
    assert_eq!(profile.handle, TEST_USERNAME);
    assert_eq!(profile.follower_count, 15000);
    assert!(profile.profile_picture_url.is_some());

    let insights = env
        .provider
        .fetch_insights(&tokens.access_token, TEST_USER_ID)
        .await
        .unwrap();
    assert!(insights.engagement_rate.is_some());
    assert!(insights.audience_demographics.is_some());
}

#[tokio::test]
#[serial]
async fn test_instagram_provider_with_encrypted_storage() {
    let env = setup_test_env().await;

    let tokens = env.provider.exchange_code("test_code_456").await.unwrap();
    let long_token = tokens.access_token;

    let encrypted = crypto::encrypt(&env.encryption_key, &long_token).unwrap();
    let decrypted = crypto::decrypt(&env.encryption_key, &encrypted).unwrap();
    assert_eq!(decrypted, long_token);

    let profile = env.provider.fetch_profile(&long_token).await.unwrap();
    assert_eq!(profile.handle, TEST_USERNAME);
}

#[tokio::test]
#[serial]
async fn test_instagram_media_and_insights_sync() {
    let env = setup_test_env().await;

    let tokens = env.provider.exchange_code("test_code_789").await.unwrap();
    let long_token = tokens.access_token;

    let media_resp = env
        .client
        .get(format!("{}/{}/media", env.base_url, TEST_USER_ID))
        .query(&[
            ("access_token", long_token.clone()),
            ("limit", "5".to_string()),
        ])
        .send()
        .await
        .unwrap();

    assert_eq!(media_resp.status(), 200);
    let media_data: serde_json::Value = media_resp.json().await.unwrap();
    let media = media_data["data"].as_array().unwrap();
    assert_eq!(media.len(), 5);
    assert!(media[0]["id"].as_str().unwrap().starts_with("media_"));
    assert!(media[0]["media_type"].is_string());

    let first_media_id = media[0]["id"].as_str().unwrap();
    let media_insights_resp = env
        .client
        .get(format!(
            "{}/media/{}/insights",
            env.base_url, first_media_id
        ))
        .query(&[
            ("access_token", long_token.clone()),
            ("metric", "total_interactions,reach,views".to_string()),
        ])
        .send()
        .await
        .unwrap();

    assert_eq!(media_insights_resp.status(), 200);
    let mi_data: serde_json::Value = media_insights_resp.json().await.unwrap();
    let mi_data_arr = mi_data["data"].as_array().unwrap();
    assert_eq!(mi_data_arr.len(), 3);

    for mi in mi_data_arr {
        assert!(mi["name"].is_string());
        assert_eq!(mi["period"], "lifetime");
        assert!(mi["values"].is_array());
    }

    let insights = env
        .provider
        .fetch_insights(&long_token, TEST_USER_ID)
        .await
        .unwrap();
    assert!(insights.engagement_rate.is_some());
    assert!(insights.audience_demographics.is_some());
}

#[tokio::test]
#[serial]
async fn test_instagram_error_handling() {
    let env = setup_test_env().await;

    let result = env.provider.exchange_code("invalid_code").await;
    assert!(result.is_err());
    assert!(matches!(result.unwrap_err(), AppError::Internal));

    let refresh_result = env.provider.refresh_token("invalid_long_token").await;
    assert!(refresh_result.is_err());
    assert!(matches!(refresh_result.unwrap_err(), AppError::Internal));

    let profile_result = env.provider.fetch_profile("invalid_token").await;
    assert!(profile_result.is_err());
    assert!(matches!(profile_result.unwrap_err(), AppError::Internal));
}
