use noshi_back::auth::ports::AuthProvider;
use noshi_back::auth::provider::LocalAuthProvider;
use noshi_back::shared::errors::AppError;

#[tokio::test]
async fn test_register_hashes_password_and_returns_tokens() {
    let provider = LocalAuthProvider::new("test-secret".to_string(), 15);
    let result = provider
        .register("test@example.com", "Test User", "password123")
        .await;

    assert!(result.is_ok());
    let (info, tokens) = result.unwrap();
    assert!(info.password_hash.starts_with("$2b$"));
    assert_eq!(info.email, "test@example.com");
    assert_eq!(info.name, "Test User");
    assert!(!tokens.access_token.is_empty());
    assert_eq!(tokens.expires_in, 900);
}

#[tokio::test]
async fn test_login_valid_password() {
    let provider = LocalAuthProvider::new("test-secret".to_string(), 15);
    let hash = bcrypt::hash("password123", bcrypt::DEFAULT_COST).unwrap();

    let result = provider
        .login("test@example.com", "password123", &hash)
        .await;

    assert!(result.is_ok());
    let (tokens, info) = result.unwrap();
    assert!(!tokens.access_token.is_empty());
    assert_eq!(tokens.expires_in, 900);
    assert_eq!(info.email, "test@example.com");
}

#[tokio::test]
async fn test_login_invalid_password() {
    let provider = LocalAuthProvider::new("test-secret".to_string(), 15);
    let hash = bcrypt::hash("password123", bcrypt::DEFAULT_COST).unwrap();

    let result = provider
        .login("test@example.com", "wrongpassword", &hash)
        .await;

    assert!(result.is_err());
    match result.unwrap_err() {
        AppError::BadRequest(msg) => assert_eq!(msg, "Invalid email or password"),
        _ => panic!("Expected BadRequest error"),
    }
}

#[tokio::test]
async fn test_introspect_token_roundtrip() {
    let provider = LocalAuthProvider::new("test-secret".to_string(), 15);
    let token = provider
        .generate_access_token("user-123", "test@example.com")
        .await
        .unwrap();

    let result = provider.introspect_token(&token).await;
    assert!(result.is_ok());
    let info = result.unwrap();
    assert_eq!(info.sub, "user-123");
    assert_eq!(info.email, "test@example.com");
}

#[tokio::test]
async fn test_introspect_invalid_token() {
    let provider = LocalAuthProvider::new("test-secret".to_string(), 15);
    let result = provider.introspect_token("invalid-token").await;
    assert!(result.is_err());
}

#[tokio::test]
async fn test_token_expiry_seconds() {
    let provider = LocalAuthProvider::new("test-secret".to_string(), 15);
    assert_eq!(provider.token_expiry_seconds(), 900);
}

#[tokio::test]
async fn test_different_secret_rejects_token() {
    let provider1 = LocalAuthProvider::new("secret-1".to_string(), 15);
    let provider2 = LocalAuthProvider::new("secret-2".to_string(), 15);

    let token = provider1
        .generate_access_token("user-123", "test@example.com")
        .await
        .unwrap();

    let result = provider2.introspect_token(&token).await;
    assert!(result.is_err());
}
