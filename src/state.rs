use crate::auth::email::SmtpEmailProvider;
use crate::auth::ports::AuthProvider;
use crate::auth::provider::LocalAuthProvider;
use crate::auth::repository::{
    DieselPasswordResetRepository, DieselSessionRepository, DieselUserRepository,
};
use crate::auth::service::AuthService;
use crate::profile::repository::{DieselManualPlatformRepository, DieselProfileRepository};
use crate::profile::service::ProfileService;
use crate::shared::crypto;
use crate::shared::db::DbPool;
use crate::shared::email_client::EmailClient;
use crate::shared::errors::{AppError, AppResult};
use crate::shared::types::AuthenticatedUser;
use crate::social::instagram::InstagramProvider;
use crate::social::ports::SocialProvider;
use crate::social::repository::DieselSocialConnectionRepository;
use crate::social::service::SocialService;
use crate::waitlist::repository::DieselWaitlistRepository;
use crate::waitlist::service::WaitlistService;
use std::sync::Arc;

pub type ConcreteAuthService = AuthService<
    LocalAuthProvider,
    DieselUserRepository,
    DieselSessionRepository,
    DieselPasswordResetRepository,
    SmtpEmailProvider,
>;

pub type ConcreteProfileService =
    ProfileService<DieselProfileRepository, DieselManualPlatformRepository>;

pub type ConcreteSocialService = SocialService<DieselSocialConnectionRepository>;

#[derive(Clone)]
pub struct AppState {
    pub waitlist_service: WaitlistService<DieselWaitlistRepository>,
    pub auth_service: ConcreteAuthService,
    pub profile_service: ConcreteProfileService,
    pub social_service: ConcreteSocialService,
    pub social_oauth_state_secret: String,
}

impl AppState {
    pub fn new(
        pool: DbPool,
        jwt_secret: String,
        jwt_expiry_minutes: u64,
        frontend_url: String,
        config: &crate::shared::config::Config,
    ) -> AppResult<Self> {
        let waitlist_repo = DieselWaitlistRepository::new(pool.clone());
        let waitlist_service = WaitlistService::new(waitlist_repo);

        let email_client = config
            .email_function_url
            .clone()
            .map(|url| {
                tracing::info!("Email function client configured");
                Arc::new(EmailClient::new(url))
            })
            .or_else(|| {
                tracing::warn!("EMAIL_FUNCTION_URL not set, waitlist confirmation emails disabled");
                None
            });

        let waitlist_service = waitlist_service.with_email_client(email_client);

        let auth_provider = LocalAuthProvider::new(jwt_secret.clone(), jwt_expiry_minutes);
        let user_repo = DieselUserRepository::new(pool.clone());
        let session_repo = DieselSessionRepository::new(pool.clone());
        let password_reset_repo = DieselPasswordResetRepository::new(pool.clone());

        let smtp_host = std::env::var("SMTP_HOST").unwrap_or_else(|_| "127.0.0.1".to_string());
        let smtp_port: u16 = std::env::var("SMTP_PORT")
            .unwrap_or_else(|_| "1025".to_string())
            .parse()
            .unwrap_or(1025);
        let smtp_user = std::env::var("SMTP_USER").unwrap_or_default();
        let smtp_password = std::env::var("SMTP_PASSWORD").unwrap_or_default();
        let smtp_from_email =
            std::env::var("SMTP_FROM_EMAIL").unwrap_or_else(|_| "noreply@noshi.com".to_string());
        let smtp_from_name =
            std::env::var("SMTP_FROM_NAME").unwrap_or_else(|_| "Noshi".to_string());

        let email_provider = SmtpEmailProvider::new(
            &smtp_host,
            smtp_port,
            &smtp_user,
            &smtp_password,
            &smtp_from_email,
            &smtp_from_name,
        );

        let auth_service = AuthService::new(
            auth_provider,
            user_repo,
            session_repo,
            password_reset_repo,
            email_provider,
            7,
            frontend_url,
        );

        let social_repo = DieselSocialConnectionRepository::new(pool.clone());

        let encryption_key = config
            .token_encryption_key
            .as_ref()
            .and_then(|k| crypto::decode_key(k).ok())
            .ok_or_else(|| {
                tracing::error!(
                    "TOKEN_ENCRYPTION_KEY not set or invalid (must be 32 bytes base64)"
                );
                AppError::Internal
            })?;

        let mut providers: std::collections::HashMap<String, Arc<dyn SocialProvider>> =
            std::collections::HashMap::new();
        if let (Some(client_id), Some(client_secret), Some(redirect_uri)) = (
            config.instagram_client_id.clone(),
            config.instagram_client_secret.clone(),
            config.instagram_redirect_uri.clone(),
        ) {
            providers.insert(
                "instagram".to_string(),
                Arc::new(InstagramProvider::new(
                    client_id,
                    client_secret,
                    redirect_uri,
                )),
            );
        }

        let social_service = SocialService::new(social_repo, providers, encryption_key);

        let social_oauth_state_secret =
            config.social_oauth_state_secret.clone().unwrap_or_else(|| {
                tracing::warn!("SOCIAL_OAUTH_STATE_SECRET not set, falling back to JWT_SECRET");
                jwt_secret.clone()
            });

        Ok(Self {
            waitlist_service,
            auth_service,
            profile_service: ProfileService::new(
                DieselProfileRepository::new(pool.clone()),
                DieselManualPlatformRepository::new(pool),
            ),
            social_service,
            social_oauth_state_secret,
        })
    }
}

pub async fn get_auth_user(
    _request: &axum::http::Request<axum::body::Body>,
    token: &str,
) -> AppResult<AuthenticatedUser> {
    let jwt_secret = std::env::var("JWT_SECRET").unwrap_or_else(|_| "change-me".to_string());
    let jwt_expiry_minutes: u64 = std::env::var("JWT_EXPIRY_MINUTES")
        .unwrap_or_else(|_| "15".to_string())
        .parse()
        .unwrap_or(15);

    let provider = LocalAuthProvider::new(jwt_secret, jwt_expiry_minutes);
    let user_info = provider.introspect_token(token).await?;

    Ok(AuthenticatedUser {
        id: user_info.sub,
        email: user_info.email,
        name: user_info.name,
        token: token.to_string(),
    })
}
