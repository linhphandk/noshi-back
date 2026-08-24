use std::collections::HashMap;
use std::sync::Arc;
use tracing::{debug, info, instrument};
use uuid::Uuid;

use crate::shared::crypto;
use crate::shared::errors::{AppError, AppResult};
use crate::social::models::{
    ConnectSocialRequest, NewSocialConnection, SocialConnectionResponse, UpdateSocialConnection,
};
use crate::social::ports::{SocialConnectionRepository, SocialProvider};

#[derive(Clone)]
pub struct SocialService<R: SocialConnectionRepository> {
    repo: R,
    pub providers: HashMap<String, Arc<dyn SocialProvider>>,
    encryption_key: [u8; 32],
}

impl<R: SocialConnectionRepository> SocialService<R> {
    pub fn new(
        repo: R,
        providers: HashMap<String, Arc<dyn SocialProvider>>,
        encryption_key: [u8; 32],
    ) -> Self {
        Self {
            repo,
            providers,
            encryption_key,
        }
    }

    #[instrument(skip(self), fields(user_id = %user_id, platform = %req.platform))]
    pub async fn connect(
        &self,
        user_id: Uuid,
        req: ConnectSocialRequest,
    ) -> AppResult<SocialConnectionResponse> {
        let provider = self.providers.get(&req.platform).ok_or_else(|| {
            AppError::BadRequest(format!("Unsupported platform: {}", req.platform))
        })?;

        debug!("Exchanging code for tokens");
        let tokens = provider.exchange_code(&req.code).await?;

        debug!("Fetching profile");
        let profile = provider.fetch_profile(&tokens.access_token).await?;

        debug!("Fetching insights");
        let insights = provider
            .fetch_insights(&tokens.access_token, &tokens.platform_user_id)
            .await?;

        let access_token_encrypted = crypto::encrypt(&self.encryption_key, &tokens.access_token)?;
        let refresh_token_encrypted = tokens
            .refresh_token
            .as_ref()
            .map(|rt| crypto::encrypt(&self.encryption_key, rt))
            .transpose()?;

        let platform_str = req.platform.clone();
        debug!("Checking for existing connection");
        if let Some(existing) = self
            .repo
            .find_by_user_and_platform(user_id, &platform_str)
            .await?
        {
            debug!("Updating existing connection");
            let updated = self
                .repo
                .update(
                    existing.id,
                    UpdateSocialConnection {
                        platform: Some(platform_str.clone()),
                        platform_user_id: Some(tokens.platform_user_id),
                        handle: Some(profile.handle),
                        access_token_encrypted: Some(access_token_encrypted),
                        refresh_token_encrypted,
                        token_expires_at: Some(tokens.expires_at),
                        follower_count: Some(profile.follower_count),
                        engagement_rate: insights.engagement_rate,
                        last_synced_at: Some(chrono::Utc::now().naive_utc()),
                        is_primary: Some(false),
                    },
                )
                .await?;

            if let Some(demographics) = insights.audience_demographics {
                self.repo
                    .update_demographics(existing.id, Some(demographics))
                    .await?;
            }

            info!(user_id = %user_id, platform = %platform_str, "social connection updated");
            return Ok(updated.into());
        }

        debug!("Creating new connection");
        // Clone values needed for potential race condition update
        let platform_str_for_update = platform_str.clone();
        let platform_user_id_for_update = tokens.platform_user_id.clone();
        let handle_for_update = profile.handle.clone();
        let access_token_encrypted_for_update = access_token_encrypted.clone();
        let refresh_token_encrypted_for_update = refresh_token_encrypted.clone();
        let expires_at_for_update = tokens.expires_at;
        let follower_count_for_update = profile.follower_count;
        let engagement_rate_for_update = insights.engagement_rate;

        let connection = match self
            .repo
            .create(NewSocialConnection {
                id: Uuid::now_v7(),
                user_id,
                platform: platform_str.clone(),
                platform_user_id: tokens.platform_user_id,
                handle: profile.handle,
                access_token_encrypted,
                refresh_token_encrypted,
                token_expires_at: tokens.expires_at,
                follower_count: profile.follower_count,
                is_primary: false,
            })
            .await
        {
            Ok(conn) => conn,
            Err(e) if e.to_string().contains("unique") || e.to_string().contains("duplicate") => {
                debug!("Race condition detected, updating existing connection");
                let existing = self
                    .repo
                    .find_by_user_and_platform(user_id, &platform_str_for_update)
                    .await?
                    .ok_or(AppError::Internal)?;
                self.repo
                    .update(
                        existing.id,
                        UpdateSocialConnection {
                            platform: Some(platform_str_for_update),
                            platform_user_id: Some(platform_user_id_for_update),
                            handle: Some(handle_for_update),
                            access_token_encrypted: Some(access_token_encrypted_for_update),
                            refresh_token_encrypted: refresh_token_encrypted_for_update,
                            token_expires_at: Some(expires_at_for_update),
                            follower_count: Some(follower_count_for_update),
                            engagement_rate: engagement_rate_for_update,
                            last_synced_at: Some(chrono::Utc::now().naive_utc()),
                            is_primary: Some(false),
                        },
                    )
                    .await?
            }
            Err(e) => return Err(e),
        };

        if let Some(demographics) = insights.audience_demographics {
            self.repo
                .update_demographics(connection.id, Some(demographics))
                .await?;
        }

        info!(user_id = %user_id, platform = %platform_str, "social connection created");
        Ok(connection.into())
    }

    #[instrument(skip(self), fields(user_id = %user_id))]
    pub async fn list_connections(
        &self,
        user_id: Uuid,
    ) -> AppResult<Vec<crate::social::models::SocialConnectionResponse>> {
        debug!("Listing social connections");
        let connections = self.repo.find_by_user_id(user_id).await?;
        Ok(connections.into_iter().map(|c| c.into()).collect())
    }

    #[instrument(skip(self), fields(user_id = %user_id, connection_id = %connection_id))]
    pub async fn disconnect(&self, user_id: Uuid, connection_id: Uuid) -> AppResult<()> {
        debug!("Disconnecting social connection");
        let connection = self
            .repo
            .find_by_id(connection_id)
            .await?
            .ok_or_else(|| AppError::NotFound("Connection not found".to_string()))?;

        if connection.user_id != user_id {
            return Err(AppError::Forbidden("Not your connection".to_string()));
        }

        self.repo.delete(connection_id).await?;
        info!(user_id = %user_id, connection_id = %connection_id, "social connection deleted");
        Ok(())
    }

    #[instrument(skip(self), fields(user_id = %user_id, connection_id = %connection_id))]
    pub async fn sync(
        &self,
        user_id: Uuid,
        connection_id: Uuid,
    ) -> AppResult<crate::social::models::SocialConnectionResponse> {
        debug!("Syncing social connection");
        let connection = self
            .repo
            .find_by_id(connection_id)
            .await?
            .ok_or_else(|| AppError::NotFound("Connection not found".to_string()))?;

        if connection.user_id != user_id {
            return Err(AppError::Forbidden("Not your connection".to_string()));
        }

        let provider = self.providers.get(&connection.platform).ok_or_else(|| {
            AppError::BadRequest(format!("Unsupported platform: {}", connection.platform))
        })?;

        let access_token =
            crypto::decrypt(&self.encryption_key, &connection.access_token_encrypted)?;

        let needs_refresh = chrono::Utc::now().naive_utc() + chrono::Duration::days(7)
            >= connection.token_expires_at;
        let access_token = if needs_refresh {
            debug!("Token expiring soon, refreshing");
            let new_tokens = provider.refresh_token(&access_token).await?;
            let new_encrypted = crypto::encrypt(&self.encryption_key, &new_tokens.access_token)?;
            self.repo
                .update(
                    connection.id,
                    UpdateSocialConnection {
                        access_token_encrypted: Some(new_encrypted),
                        token_expires_at: Some(new_tokens.expires_at),
                        ..Default::default()
                    },
                )
                .await?;
            new_tokens.access_token
        } else {
            access_token
        };

        let profile = provider.fetch_profile(&access_token).await?;
        let insights = provider
            .fetch_insights(&access_token, &connection.platform_user_id)
            .await?;

        let updated = self
            .repo
            .update(
                connection.id,
                UpdateSocialConnection {
                    handle: Some(profile.handle),
                    follower_count: Some(profile.follower_count),
                    engagement_rate: insights.engagement_rate,
                    last_synced_at: Some(chrono::Utc::now().naive_utc()),
                    ..Default::default()
                },
            )
            .await?;

        if let Some(demographics) = insights.audience_demographics {
            self.repo
                .update_demographics(connection.id, Some(demographics))
                .await?;
        }

        info!(user_id = %user_id, connection_id = %connection_id, "social connection synced");
        Ok(updated.into())
    }
}
