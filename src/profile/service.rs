use tracing::{debug, info, instrument};
use uuid::Uuid;

use crate::profile::models::{
    CreateManualPlatformRequest, CreateProfileRequest, NewManualPlatform, NewProfile,
    UpdateManualPlatform, UpdateManualPlatformRequest, UpdateProfile, UpdateProfileRequest,
};
use crate::profile::ports::{ManualPlatformRepository, ProfileRepository};
use crate::shared::errors::{AppError, AppResult};

#[derive(Clone)]
pub struct ProfileService<P: ProfileRepository, M: ManualPlatformRepository> {
    profile_repo: P,
    manual_platform_repo: M,
}

impl<P: ProfileRepository, M: ManualPlatformRepository> ProfileService<P, M> {
    pub fn new(profile_repo: P, manual_platform_repo: M) -> Self {
        Self {
            profile_repo,
            manual_platform_repo,
        }
    }

    #[instrument(skip(self, req), fields(user_id = %user_id))]
    pub async fn create_profile(
        &self,
        user_id: Uuid,
        req: CreateProfileRequest,
    ) -> AppResult<crate::profile::models::Profile> {
        debug!("checking for existing profile");
        if self.profile_repo.find_by_user_id(user_id).await?.is_some() {
            return Err(AppError::Conflict("Profile already exists".to_string()));
        }
        debug!("checking slug availability");
        if self.profile_repo.find_by_slug(&req.slug).await?.is_some() {
            return Err(AppError::Conflict("Slug already taken".to_string()));
        }
        debug!("inserting profile");
        let profile = self
            .profile_repo
            .create(NewProfile {
                id: Uuid::now_v7(),
                user_id,
                slug: req.slug,
                niches: req.niches.into_iter().map(Some).collect(),
                headline: req.headline,
                is_published: req.is_published.unwrap_or(false),
            })
            .await?;
        info!(user_id = %user_id, "profile created");
        Ok(profile)
    }

    #[instrument(skip(self), fields(user_id = %user_id))]
    pub async fn get_profile(&self, user_id: Uuid) -> AppResult<crate::profile::models::Profile> {
        debug!("fetching profile");
        self.profile_repo
            .find_by_user_id(user_id)
            .await?
            .ok_or_else(|| AppError::NotFound("Profile not found".to_string()))
    }

    #[instrument(skip(self), fields(slug = %slug))]
    pub async fn get_public_profile(
        &self,
        slug: &str,
    ) -> AppResult<(
        crate::profile::models::Profile,
        Vec<crate::profile::models::ManualPlatform>,
    )> {
        debug!("looking up profile by slug");
        let profile = self
            .profile_repo
            .find_by_slug(slug)
            .await?
            .ok_or_else(|| AppError::NotFound("Profile not found".to_string()))?;
        if !profile.is_published {
            return Err(AppError::NotFound("Profile not found".to_string()));
        }
        debug!("fetching platforms");
        let platforms = self
            .manual_platform_repo
            .find_by_user_id(profile.user_id)
            .await?;
        Ok((profile, platforms))
    }

    #[instrument(skip(self, req), fields(user_id = %user_id))]
    pub async fn update_profile(
        &self,
        user_id: Uuid,
        req: UpdateProfileRequest,
    ) -> AppResult<crate::profile::models::Profile> {
        debug!("fetching current profile");
        let profile = self.get_profile(user_id).await?;
        if let Some(ref slug) = req.slug {
            debug!("checking slug uniqueness");
            if let Some(existing) = self.profile_repo.find_by_slug(slug).await? {
                if existing.id != profile.id {
                    return Err(AppError::Conflict("Slug already taken".to_string()));
                }
            }
        }
        debug!("updating profile");
        self.profile_repo
            .update(
                profile.id,
                UpdateProfile {
                    slug: req.slug,
                    niches: req.niches.map(|n| n.into_iter().map(Some).collect()),
                    headline: req.headline,
                    is_published: req.is_published,
                    completion_score: None,
                },
            )
            .await
    }

    #[instrument(skip(self), fields(user_id = %user_id))]
    pub async fn delete_profile(&self, user_id: Uuid) -> AppResult<()> {
        debug!("fetching profile to delete");
        let profile = self.get_profile(user_id).await?;
        debug!("deleting profile");
        self.profile_repo.delete(profile.id).await
    }

    #[instrument(skip(self, req), fields(user_id = %user_id))]
    pub async fn add_manual_platform(
        &self,
        user_id: Uuid,
        req: CreateManualPlatformRequest,
    ) -> AppResult<crate::profile::models::ManualPlatform> {
        debug!("verifying profile exists");
        let _profile = self.get_profile(user_id).await?;
        debug!("checking for existing platform");
        if self
            .manual_platform_repo
            .find_by_user_and_platform(user_id, &req.platform)
            .await?
            .is_some()
        {
            return Err(AppError::Conflict(format!(
                "Platform '{}' already exists",
                req.platform
            )));
        }
        debug!("inserting platform");
        self.manual_platform_repo
            .create(NewManualPlatform {
                id: Uuid::now_v7(),
                user_id,
                platform: req.platform,
                handle: req.handle,
                follower_count: req.follower_count,
            })
            .await
    }

    #[instrument(skip(self), fields(user_id = %user_id))]
    pub async fn get_manual_platforms(
        &self,
        user_id: Uuid,
    ) -> AppResult<Vec<crate::profile::models::ManualPlatform>> {
        debug!("fetching platforms");
        self.manual_platform_repo.find_by_user_id(user_id).await
    }

    #[instrument(skip(self, req), fields(user_id = %user_id, platform_id = %platform_id))]
    pub async fn update_manual_platform(
        &self,
        user_id: Uuid,
        platform_id: Uuid,
        req: UpdateManualPlatformRequest,
    ) -> AppResult<crate::profile::models::ManualPlatform> {
        debug!("fetching platform");
        let platform = self
            .manual_platform_repo
            .find_by_id(platform_id)
            .await?
            .ok_or_else(|| AppError::NotFound("Platform not found".to_string()))?;
        if platform.user_id != user_id {
            return Err(AppError::Forbidden("Not your platform".to_string()));
        }
        debug!("updating platform");
        self.manual_platform_repo
            .update(
                platform_id,
                UpdateManualPlatform {
                    platform: req.platform,
                    handle: req.handle,
                    follower_count: req.follower_count,
                },
            )
            .await
    }

    #[instrument(skip(self), fields(user_id = %user_id, platform_id = %platform_id))]
    pub async fn delete_manual_platform(&self, user_id: Uuid, platform_id: Uuid) -> AppResult<()> {
        debug!("fetching platform");
        let platform = self
            .manual_platform_repo
            .find_by_id(platform_id)
            .await?
            .ok_or_else(|| AppError::NotFound("Platform not found".to_string()))?;
        if platform.user_id != user_id {
            return Err(AppError::Forbidden("Not your platform".to_string()));
        }
        debug!("deleting platform");
        self.manual_platform_repo.delete(platform_id).await
    }
}
