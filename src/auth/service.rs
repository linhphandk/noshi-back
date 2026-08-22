use chrono::{Duration, Utc};
use sha2::{Digest, Sha256};
use tracing::{debug, error, info, instrument, warn};
use uuid::Uuid;

use crate::auth::email::EmailProvider;
use crate::auth::models::{NewPasswordReset, NewUser};
use crate::auth::ports::{
    AuthProvider, PasswordResetRepository, SessionRepository, UserRepository,
};
use crate::shared::errors::{AppError, AppResult};

#[derive(Clone)]
pub struct AuthService<
    A: AuthProvider,
    U: UserRepository,
    S: SessionRepository,
    P: PasswordResetRepository,
    E: EmailProvider,
> {
    auth_provider: A,
    user_repo: U,
    session_repo: S,
    password_reset_repo: P,
    email_provider: E,
    refresh_token_expiry_days: i64,
    frontend_url: String,
}

impl<
        A: AuthProvider,
        U: UserRepository,
        S: SessionRepository,
        P: PasswordResetRepository,
        E: EmailProvider,
    > AuthService<A, U, S, P, E>
{
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        auth_provider: A,
        user_repo: U,
        session_repo: S,
        password_reset_repo: P,
        email_provider: E,
        refresh_token_expiry_days: i64,
        frontend_url: String,
    ) -> Self {
        Self {
            auth_provider,
            user_repo,
            session_repo,
            password_reset_repo,
            email_provider,
            refresh_token_expiry_days,
            frontend_url,
        }
    }

    #[instrument(skip(self), fields(email = %email))]
    pub async fn register(
        &self,
        email: String,
        name: String,
        password: String,
    ) -> AppResult<(crate::auth::models::User, crate::auth::models::Tokens)> {
        debug!("validating input");
        if email.is_empty() {
            return Err(AppError::BadRequest("Email cannot be empty".to_string()));
        }
        if name.is_empty() {
            return Err(AppError::BadRequest("Name cannot be empty".to_string()));
        }
        if password.len() < 8 {
            return Err(AppError::BadRequest(
                "Password must be at least 8 characters".to_string(),
            ));
        }

        debug!("checking for existing user");
        if self.user_repo.find_by_email(&email).await?.is_some() {
            return Err(AppError::Conflict("Email already registered".to_string()));
        }

        debug!("calling auth provider");
        let (user_info, tokens) = self
            .auth_provider
            .register(&email, &name, &password)
            .await?;

        let user_id = Uuid::now_v7();
        let new_user = NewUser {
            id: user_id,
            email,
            name,
            password_hash: user_info.password_hash,
        };

        debug!("persisting user");
        let user = self.user_repo.create(new_user).await?;

        let refresh_token = Uuid::now_v7().to_string();
        let expires_at = Utc::now()
            .checked_add_signed(Duration::days(self.refresh_token_expiry_days))
            .ok_or(AppError::Internal)?
            .naive_utc();

        debug!("creating session");
        self.session_repo
            .create(user.id, refresh_token.clone(), expires_at)
            .await?;

        let tokens = crate::auth::models::Tokens {
            access_token: tokens.access_token,
            expires_in: tokens.expires_in,
            refresh_token: Some(refresh_token),
        };

        info!(user_id = %user.id, "user registered");
        Ok((user, tokens))
    }

    #[instrument(skip(self), fields(email = %email))]
    pub async fn login(
        &self,
        email: String,
        password: String,
    ) -> AppResult<(crate::auth::models::User, crate::auth::models::Tokens)> {
        debug!("looking up user");
        let user = self.user_repo.find_by_email(&email).await?;
        let user = user.ok_or_else(|| {
            warn!(email = %email, "login failed: user not found");
            AppError::BadRequest("Invalid email or password".to_string())
        })?;

        debug!("validating credentials");
        let (tokens, _user_info) = self
            .auth_provider
            .login(&email, &password, &user.password_hash)
            .await?;

        let refresh_token = Uuid::now_v7().to_string();
        let expires_at = Utc::now()
            .checked_add_signed(Duration::days(self.refresh_token_expiry_days))
            .ok_or(AppError::Internal)?
            .naive_utc();

        debug!("creating session");
        self.session_repo
            .create(user.id, refresh_token.clone(), expires_at)
            .await?;

        let tokens = crate::auth::models::Tokens {
            access_token: tokens.access_token,
            expires_in: tokens.expires_in,
            refresh_token: Some(refresh_token),
        };

        info!(user_id = %user.id, "user logged in");
        Ok((user, tokens))
    }

    #[instrument(skip(self, refresh_token))]
    pub async fn logout(&self, refresh_token: String) -> AppResult<()> {
        debug!("revoking session");
        self.session_repo.revoke(&refresh_token).await
    }

    #[instrument(skip(self, refresh_token))]
    pub async fn refresh_token(
        &self,
        refresh_token: String,
    ) -> AppResult<(crate::auth::models::User, crate::auth::models::Tokens)> {
        debug!("looking up session");
        let session = self.session_repo.find_by_token(&refresh_token).await?;
        let session = session.ok_or_else(|| {
            warn!("refresh_token: invalid token");
            AppError::BadRequest("Invalid refresh token".to_string())
        })?;

        if session.expires_at < Utc::now().naive_utc() {
            debug!("token expired, revoking");
            self.session_repo.revoke(&refresh_token).await?;
            warn!("refresh_token: expired token revoked");
            return Err(AppError::BadRequest("Refresh token expired".to_string()));
        }

        debug!("loading user");
        let user = self.user_repo.find_by_id(session.user_id).await?;
        let user = user.ok_or_else(|| {
            error!(user_id = %session.user_id, "refresh_token: user not found");
            AppError::Internal
        })?;

        debug!("generating new access token");
        let new_access_token = self
            .auth_provider
            .generate_access_token(&user.id.to_string(), &user.email)
            .await?;

        debug!("rotating refresh token");
        self.session_repo.revoke(&refresh_token).await?;

        let new_refresh_token = Uuid::now_v7().to_string();
        let expires_at = Utc::now()
            .checked_add_signed(Duration::days(self.refresh_token_expiry_days))
            .ok_or(AppError::Internal)?
            .naive_utc();

        self.session_repo
            .create(user.id, new_refresh_token.clone(), expires_at)
            .await?;

        let tokens = crate::auth::models::Tokens {
            access_token: new_access_token,
            expires_in: self.auth_provider.token_expiry_seconds(),
            refresh_token: Some(new_refresh_token),
        };

        info!(user_id = %user.id, "token refreshed");
        Ok((user, tokens))
    }

    #[instrument(skip(self, access_token))]
    pub async fn get_current_user(
        &self,
        access_token: String,
    ) -> AppResult<crate::auth::models::User> {
        debug!("introspecting token");
        let user_info = self.auth_provider.introspect_token(&access_token).await?;
        debug!("looking up user by email");
        let user = self.user_repo.find_by_email(&user_info.email).await?;
        user.ok_or_else(|| {
            warn!(email = %user_info.email, "get_current_user: user not found");
            AppError::BadRequest("User not found".to_string())
        })
    }

    #[instrument(skip(self), fields(email = %email))]
    pub async fn forgot_password(&self, email: String) -> AppResult<()> {
        debug!("looking up user");
        let user = match self.user_repo.find_by_email(&email).await {
            Ok(Some(u)) => u,
            Ok(None) => {
                info!(email = %email, "forgot_password: user not found, returning silently");
                return Ok(());
            }
            Err(e) => return Err(e),
        };

        let token = Uuid::new_v4().to_string();
        let token_hash = format!("{:x}", Sha256::digest(token.as_bytes()));
        let expires_at = Utc::now()
            .checked_add_signed(Duration::hours(1))
            .ok_or(AppError::Internal)?
            .naive_utc();

        let reset_url = format!(
            "{}/reset-password?token={}",
            self.frontend_url.trim_end_matches('/'),
            token
        );

        debug!("storing password reset token");
        self.password_reset_repo
            .create(NewPasswordReset {
                user_id: user.id,
                token_hash,
                expires_at,
            })
            .await?;

        debug!("sending reset email");
        if let Err(e) = self
            .email_provider
            .send_password_reset(&user.email, &reset_url)
            .await
        {
            warn!(user_id = %user.id, "forgot_password: failed to send email: {:?}", e);
        }

        info!(user_id = %user.id, "forgot_password: reset email sent");
        Ok(())
    }

    #[instrument(skip(self, token, new_password))]
    pub async fn reset_password(&self, token: String, new_password: String) -> AppResult<()> {
        debug!("validating password length");
        if new_password.len() < 8 {
            return Err(AppError::BadRequest(
                "Password must be at least 8 characters".to_string(),
            ));
        }

        debug!("looking up reset token");
        let token_hash = format!("{:x}", Sha256::digest(token.as_bytes()));
        let reset = self
            .password_reset_repo
            .find_by_token_hash(&token_hash)
            .await?;
        let reset = reset.ok_or_else(|| {
            warn!("reset_password: invalid token");
            AppError::BadRequest("Invalid or expired token".to_string())
        })?;

        if reset.expires_at < Utc::now().naive_utc() {
            warn!(user_id = %reset.user_id, "reset_password: expired token");
            return Err(AppError::BadRequest("Invalid or expired token".to_string()));
        }

        if reset.used_at.is_some() {
            debug!("token reuse detected, revoking all sessions");
            self.session_repo.revoke_all_for_user(reset.user_id).await?;
            warn!(
                user_id = %reset.user_id,
                "reset_password: token reuse detected, sessions revoked"
            );
            return Err(AppError::BadRequest("Token already used".to_string()));
        }

        debug!("hashing new password");
        let password_hash = bcrypt::hash(&new_password, bcrypt::DEFAULT_COST).map_err(|e| {
            error!("reset_password: bcrypt error: {:?}", e);
            AppError::Internal
        })?;

        debug!("updating password hash");
        self.user_repo
            .update_password_hash(reset.user_id, password_hash)
            .await?;

        debug!("marking token used and revoking sessions");
        self.password_reset_repo.mark_used(reset.id).await?;
        self.session_repo.revoke_all_for_user(reset.user_id).await?;

        info!(user_id = %reset.user_id, "password reset successfully");
        Ok(())
    }
}
