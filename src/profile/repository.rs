use async_trait::async_trait;
use diesel::prelude::*;
use tracing::{error, instrument};
use uuid::Uuid;

use crate::profile::models::{
    ManualPlatform, NewManualPlatform, NewProfile, Profile, UpdateManualPlatform, UpdateProfile,
};
use crate::profile::ports::{ManualPlatformRepository, ProfileRepository};
use crate::schema::{manual_platforms, profiles};
use crate::shared::db::DbPool;
use crate::shared::errors::{AppError, AppResult};

#[derive(Clone)]
pub struct DieselProfileRepository {
    pool: DbPool,
}

impl DieselProfileRepository {
    pub fn new(pool: DbPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl ProfileRepository for DieselProfileRepository {
    #[instrument(skip(self, profile))]
    async fn create(&self, profile: NewProfile) -> AppResult<Profile> {
        let mut conn = self.pool.get().map_err(|e| {
            error!("Connection pool error: {:?}", e);
            AppError::Internal
        })?;
        diesel::insert_into(profiles::table)
            .values(&profile)
            .get_result::<Profile>(&mut conn)
            .map_err(|e| {
                error!("Database error: {:?}", e);
                AppError::Internal
            })
    }

    #[instrument(skip(self))]
    async fn find_by_id(&self, id: Uuid) -> AppResult<Option<Profile>> {
        let mut conn = self.pool.get().map_err(|e| {
            error!("Connection pool error: {:?}", e);
            AppError::Internal
        })?;
        profiles::table
            .find(id)
            .first::<Profile>(&mut conn)
            .optional()
            .map_err(|e| {
                error!("Database error: {:?}", e);
                AppError::Internal
            })
    }

    #[instrument(skip(self))]
    async fn find_by_user_id(&self, user_id: Uuid) -> AppResult<Option<Profile>> {
        let mut conn = self.pool.get().map_err(|e| {
            error!("Connection pool error: {:?}", e);
            AppError::Internal
        })?;
        profiles::table
            .filter(profiles::user_id.eq(user_id))
            .first::<Profile>(&mut conn)
            .optional()
            .map_err(|e| {
                error!("Database error: {:?}", e);
                AppError::Internal
            })
    }

    #[instrument(skip(self))]
    async fn find_by_slug(&self, slug: &str) -> AppResult<Option<Profile>> {
        let mut conn = self.pool.get().map_err(|e| {
            error!("Connection pool error: {:?}", e);
            AppError::Internal
        })?;
        profiles::table
            .filter(profiles::slug.eq(slug))
            .first::<Profile>(&mut conn)
            .optional()
            .map_err(|e| {
                error!("Database error: {:?}", e);
                AppError::Internal
            })
    }

    #[instrument(skip(self, profile))]
    async fn update(&self, id: Uuid, profile: UpdateProfile) -> AppResult<Profile> {
        let mut conn = self.pool.get().map_err(|e| {
            error!("Connection pool error: {:?}", e);
            AppError::Internal
        })?;
        diesel::update(profiles::table.find(id))
            .set(&profile)
            .get_result::<Profile>(&mut conn)
            .map_err(|e| {
                error!("Database error: {:?}", e);
                AppError::Internal
            })
    }

    #[instrument(skip(self))]
    async fn delete(&self, id: Uuid) -> AppResult<()> {
        let mut conn = self.pool.get().map_err(|e| {
            error!("Connection pool error: {:?}", e);
            AppError::Internal
        })?;
        diesel::delete(profiles::table.find(id))
            .execute(&mut conn)
            .map_err(|e| {
                error!("Database error: {:?}", e);
                AppError::Internal
            })?;
        Ok(())
    }
}

#[derive(Clone)]
pub struct DieselManualPlatformRepository {
    pool: DbPool,
}

impl DieselManualPlatformRepository {
    pub fn new(pool: DbPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl ManualPlatformRepository for DieselManualPlatformRepository {
    #[instrument(skip(self, platform))]
    async fn create(&self, platform: NewManualPlatform) -> AppResult<ManualPlatform> {
        let mut conn = self.pool.get().map_err(|e| {
            error!("Connection pool error: {:?}", e);
            AppError::Internal
        })?;
        diesel::insert_into(manual_platforms::table)
            .values(&platform)
            .get_result::<ManualPlatform>(&mut conn)
            .map_err(|e| {
                error!("Database error: {:?}", e);
                AppError::Internal
            })
    }

    #[instrument(skip(self))]
    async fn find_by_id(&self, id: Uuid) -> AppResult<Option<ManualPlatform>> {
        let mut conn = self.pool.get().map_err(|e| {
            error!("Connection pool error: {:?}", e);
            AppError::Internal
        })?;
        manual_platforms::table
            .find(id)
            .first::<ManualPlatform>(&mut conn)
            .optional()
            .map_err(|e| {
                error!("Database error: {:?}", e);
                AppError::Internal
            })
    }

    #[instrument(skip(self))]
    async fn find_by_user_id(&self, user_id: Uuid) -> AppResult<Vec<ManualPlatform>> {
        let mut conn = self.pool.get().map_err(|e| {
            error!("Connection pool error: {:?}", e);
            AppError::Internal
        })?;
        manual_platforms::table
            .filter(manual_platforms::user_id.eq(user_id))
            .load::<ManualPlatform>(&mut conn)
            .map_err(|e| {
                error!("Database error: {:?}", e);
                AppError::Internal
            })
    }

    #[instrument(skip(self, platform))]
    async fn update(&self, id: Uuid, platform: UpdateManualPlatform) -> AppResult<ManualPlatform> {
        let mut conn = self.pool.get().map_err(|e| {
            error!("Connection pool error: {:?}", e);
            AppError::Internal
        })?;
        diesel::update(manual_platforms::table.find(id))
            .set(&platform)
            .get_result::<ManualPlatform>(&mut conn)
            .map_err(|e| {
                error!("Database error: {:?}", e);
                AppError::Internal
            })
    }

    #[instrument(skip(self))]
    async fn delete(&self, id: Uuid) -> AppResult<()> {
        let mut conn = self.pool.get().map_err(|e| {
            error!("Connection pool error: {:?}", e);
            AppError::Internal
        })?;
        diesel::delete(manual_platforms::table.find(id))
            .execute(&mut conn)
            .map_err(|e| {
                error!("Database error: {:?}", e);
                AppError::Internal
            })?;
        Ok(())
    }
}
