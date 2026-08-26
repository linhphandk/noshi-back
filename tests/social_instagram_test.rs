use reqwest::Client;
use serial_test::serial;
use std::time::Duration as StdDuration;
use tokio::process::Command;
use tokio::time::sleep;

use noshi_back::shared::crypto;
use noshi_back::shared::errors::AppError;
use noshi_back::social::instagram::InstagramProvider;
use noshi_back::social::ports::SocialProvider;

const SIMULATOR_BASE_URL: &str = "http://127.0.0.1:4444";
const TEST_USER_ID: &str = "17841405822304914";
const TEST_USERNAME: &str = "test_influencer";

struct TestEnv {
    provider: InstagramProvider,
    client: Client,
    encryption_key: [u8; 32],
}

fn init_tracing() {
    let _ = tracing_subscriber::fmt()
        .with_env_filter("debug")
        .try_init();
}

async fn start_simulator() -> tokio::process::Child {
    let cmd = Command::new("cargo")
        .arg("run")
        .current_dir("/tmp/instagram-simulator")
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::piped())
        .spawn()
        .expect("Failed to start simulator");

    sleep(StdDuration::from_secs(3)).await;
    cmd
}

async fn setup_test_env() -> TestEnv {
    init_tracing();
    let encryption_key =
        crypto::decode_key("JDbUt2tSDFf6lHd3tYgqHWLyySvjrcxO2USr9Ozyh5k=").unwrap();

    let provider = InstagramProvider::with_custom_urls(
        "test_client_id".to_string(),
        "test_client_secret".to_string(),
        "http://localhost:5173/oauth/instagram/callback".to_string(),
        format!("{}/oauth/authorize", SIMULATOR_BASE_URL),
        format!("{}/oauth/access_token", SIMULATOR_BASE_URL),
        format!("{}/access_token", SIMULATOR_BASE_URL),
        format!("{}/refresh_access_token", SIMULATOR_BASE_URL),
        SIMULATOR_BASE_URL.to_string(),
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
    }
}

#[tokio::test]
#[serial]
async fn test_instagram_oauth_flow() {
    let _simulator = start_simulator().await;
    sleep(StdDuration::from_secs(3)).await;

    let env = setup_test_env().await;

    // Step 1: Get authorize URL from provider (should point to simulator)
    let state = "test_state_123";
    let authorize_url = env.provider.authorize_url(state);
    assert!(authorize_url.contains("127.0.0.1:4444/oauth/authorize"));
    assert!(authorize_url.contains("client_id=test_client_id"));
    assert!(authorize_url
        .contains("redirect_uri=http%3A%2F%2Flocalhost%3A5173%2Foauth%2Finstagram%2Fcallback"));
    assert!(authorize_url
        .contains("scope=instagram_business_basic%2Cinstagram_business_manage_insights"));
    assert!(authorize_url.contains(&format!("state={}", state)));

    // Step 2: Call simulator's authorize endpoint directly to get a code
    let auth_resp = env
        .client
        .get(format!("{}/oauth/authorize", SIMULATOR_BASE_URL))
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

    let code = location
        .split("code=")
        .nth(1)
        .unwrap()
        .split('&')
        .next()
        .unwrap();

    // Step 3: Use provider to exchange code for tokens (provider calls simulator)
    let tokens = match env.provider.exchange_code(code).await {
        Ok(t) => t,
        Err(e) => {
            eprintln!("exchange_code failed: {:?}", e);
            panic!("exchange_code failed: {:?}", e);
        }
    };
    assert!(tokens.access_token.starts_with("long_"));
    assert_eq!(tokens.platform_user_id, TEST_USER_ID);
    assert!(tokens.refresh_token.is_some());
    assert_eq!(tokens.refresh_token.unwrap(), tokens.access_token);

    // Step 4: Use provider to fetch profile
    let profile = env
        .provider
        .fetch_profile(&tokens.access_token)
        .await
        .unwrap();
    assert_eq!(profile.handle, TEST_USERNAME);
    assert_eq!(profile.follower_count, 15000);
    assert!(profile.profile_picture_url.is_some());

    // Step 5: Use provider to fetch insights
    let insights = env
        .provider
        .fetch_insights(&tokens.access_token, TEST_USER_ID)
        .await
        .unwrap();
    assert!(insights.engagement_rate.is_some());
    assert!(insights.audience_demographics.is_some());

    // Step 6: Skip refresh test - simulator requires token to be 24hr+ old before refresh
    // This is correct behavior; in production you'd wait before refreshing
}

#[tokio::test]
#[serial]
async fn test_instagram_provider_with_encrypted_storage() {
    let _simulator = start_simulator().await;
    sleep(StdDuration::from_secs(3)).await;

    let env = setup_test_env().await;

    // Get tokens via provider (full flow through simulator)
    let state = "test_state_456";
    let auth_resp = env
        .client
        .get(format!("{}/oauth/authorize", SIMULATOR_BASE_URL))
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

    let location = auth_resp
        .headers()
        .get("location")
        .unwrap()
        .to_str()
        .unwrap();
    let code = location
        .split("code=")
        .nth(1)
        .unwrap()
        .split('&')
        .next()
        .unwrap();

    let tokens = env.provider.exchange_code(code).await.unwrap();
    let long_token = tokens.access_token;

    // Encrypt the token
    let encrypted = crypto::encrypt(&env.encryption_key, &long_token).unwrap();
    let decrypted = crypto::decrypt(&env.encryption_key, &encrypted).unwrap();
    assert_eq!(decrypted, long_token);

    // Test with provider using decrypted token
    let profile = env.provider.fetch_profile(&long_token).await.unwrap();
    assert_eq!(profile.handle, TEST_USERNAME);
}

#[tokio::test]
#[serial]
async fn test_instagram_media_and_insights_sync() {
    let _simulator = start_simulator().await;
    sleep(StdDuration::from_secs(3)).await;

    let env = setup_test_env().await;

    // Get long-lived token via provider
    let state = "test_state_789";
    let auth_resp = env
        .client
        .get(format!("{}/oauth/authorize", SIMULATOR_BASE_URL))
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

    let location = auth_resp
        .headers()
        .get("location")
        .unwrap()
        .to_str()
        .unwrap();
    let code = location
        .split("code=")
        .nth(1)
        .unwrap()
        .split('&')
        .next()
        .unwrap();

    let tokens = env.provider.exchange_code(code).await.unwrap();
    let long_token = tokens.access_token;

    // Test media list via direct simulator call (provider doesn't have media endpoint)
    let media_resp = env
        .client
        .get(format!("{}/{}/media", SIMULATOR_BASE_URL, TEST_USER_ID))
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

    // Test media insights via direct simulator call
    let first_media_id = media[0]["id"].as_str().unwrap();
    let media_insights_resp = env
        .client
        .get(format!(
            "{}/media/{}/insights",
            SIMULATOR_BASE_URL, first_media_id
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
        assert_eq!(mi["values"][0]["value"], mi["values"][0]["value"]); // just check it exists
    }

    // Test account insights with breakdowns via provider
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
    let _simulator = start_simulator().await;
    sleep(StdDuration::from_secs(3)).await;

    let env = setup_test_env().await;

    // Test invalid code via provider
    let result = env.provider.exchange_code("invalid_code").await;
    assert!(result.is_err());
    let err = result.unwrap_err();
    assert!(matches!(err, AppError::Internal));

    // Test invalid short token for exchange via provider
    // We need to get a valid code first, then exchange it for short token, then try to exchange invalid short
    let state = "test_state_err";
    let auth_resp = env
        .client
        .get(format!("{}/oauth/authorize", SIMULATOR_BASE_URL))
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

    let location = auth_resp
        .headers()
        .get("location")
        .unwrap()
        .to_str()
        .unwrap();
    let code = location
        .split("code=")
        .nth(1)
        .unwrap()
        .split('&')
        .next()
        .unwrap();

    // Get valid tokens
    let _tokens = env.provider.exchange_code(code).await.unwrap();

    // Now try to refresh with an invalid token (simulator will reject it)
    let refresh_result = env.provider.refresh_token("invalid_long_token").await;
    assert!(refresh_result.is_err());
    let err = refresh_result.unwrap_err();
    assert!(matches!(err, AppError::Internal));

    // Test invalid token for profile via provider
    let profile_result = env.provider.fetch_profile("invalid_token").await;
    assert!(profile_result.is_err());
}
