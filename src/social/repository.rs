use async_trait::async_trait;
use diesel::prelude::*;
use tracing::{error, instrument};
use uuid::Uuid;

use crate::schema::social_connections;
use crate::shared::db::DbPool;
use crate::shared::errors::{AppError, AppResult};
use crate::social::models::{NewSocialConnection, SocialConnection, UpdateSocialConnection};
use crate::social::ports::SocialConnectionRepository;

#[derive(Clone)]
pub struct DieselSocialConnectionRepository {
    pool: DbPool,
}

impl DieselSocialConnectionRepository {
    pub fn new(pool: DbPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl SocialConnectionRepository for DieselSocialConnectionRepository {
    #[instrument(skip(self, connection))]
    async fn create(&self, connection: NewSocialConnection) -> AppResult<SocialConnection> {
        let mut conn = self.pool.get().map_err(|e| {
            error!("Connection pool error: {:?}", e);
            AppError::Internal
        })?;
        diesel::insert_into(social_connections::table)
            .values(&connection)
            .get_result::<SocialConnection>(&mut conn)
            .map_err(|e| {
                error!("Database error: {:?}", e);
                AppError::Internal
            })
    }

    #[instrument(skip(self), fields(id = %id))]
    async fn find_by_id(&self, id: Uuid) -> AppResult<Option<SocialConnection>> {
        let mut conn = self.pool.get().map_err(|e| {
            error!("Connection pool error: {:?}", e);
            AppError::Internal
        })?;
        social_connections::table
            .find(id)
            .first::<SocialConnection>(&mut conn)
            .optional()
            .map_err(|e| {
                error!("Database error: {:?}", e);
                AppError::Internal
            })
    }

    #[instrument(skip(self), fields(user_id = %user_id))]
    async fn find_by_user_id(&self, user_id: Uuid) -> AppResult<Vec<SocialConnection>> {
        let mut conn = self.pool.get().map_err(|e| {
            error!("Connection pool error: {:?}", e);
            AppError::Internal
        })?;
        social_connections::table
            .filter(social_connections::user_id.eq(user_id))
            .load::<SocialConnection>(&mut conn)
            .map_err(|e| {
                error!("Database error: {:?}", e);
                AppError::Internal
            })
    }

    #[instrument(skip(self), fields(user_id = %user_id, platform = %platform))]
    async fn find_by_user_and_platform(
        &self,
        user_id: Uuid,
        platform: &str,
    ) -> AppResult<Option<SocialConnection>> {
        let mut conn = self.pool.get().map_err(|e| {
            error!("Connection pool error: {:?}", e);
            AppError::Internal
        })?;
        social_connections::table
            .filter(social_connections::user_id.eq(user_id))
            .filter(social_connections::platform.eq(platform))
            .first::<SocialConnection>(&mut conn)
            .optional()
            .map_err(|e| {
                error!("Database error: {:?}", e);
                AppError::Internal
            })
    }

    #[instrument(skip(self, connection))]
    async fn update(
        &self,
        id: Uuid,
        connection: UpdateSocialConnection,
    ) -> AppResult<SocialConnection> {
        let mut conn = self.pool.get().map_err(|e| {
            error!("Connection pool error: {:?}", e);
            AppError::Internal
        })?;
        diesel::update(social_connections::table.find(id))
            .set(&connection)
            .get_result::<SocialConnection>(&mut conn)
            .map_err(|e| {
                error!("Database error: {:?}", e);
                AppError::Internal
            })
    }

    #[instrument(skip(self), fields(id = %id))]
    async fn update_demographics(
        &self,
        id: Uuid,
        demographics: Option<serde_json::Value>,
    ) -> AppResult<SocialConnection> {
        let mut conn = self.pool.get().map_err(|e| {
            error!("Connection pool error: {:?}", e);
            AppError::Internal
        })?;
        diesel::update(social_connections::table.find(id))
            .set(social_connections::audience_demographics.eq(demographics))
            .get_result::<SocialConnection>(&mut conn)
            .map_err(|e| {
                error!("Database error: {:?}", e);
                AppError::Internal
            })
    }

    #[instrument(skip(self), fields(id = %id))]
    async fn delete(&self, id: Uuid) -> AppResult<()> {
        let mut conn = self.pool.get().map_err(|e| {
            error!("Connection pool error: {:?}", e);
            AppError::Internal
        })?;
        diesel::delete(social_connections::table.find(id))
            .execute(&mut conn)
            .map_err(|e| {
                error!("Database error: {:?}", e);
                AppError::Internal
            })?;
        Ok(())
    }
}
