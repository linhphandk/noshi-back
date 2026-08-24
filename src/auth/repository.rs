use async_trait::async_trait;
use chrono::NaiveDateTime;
use diesel::prelude::*;
use tracing::instrument;
use uuid::Uuid;

use crate::auth::models::{NewPasswordReset, NewUser, PasswordReset, User};
use crate::auth::ports::{PasswordResetRepository, SessionInfo, SessionRepository, UserRepository};
use crate::schema::{password_resets, sessions, users};
use crate::shared::db::DbPool;
use crate::shared::errors::{AppError, AppResult};

#[derive(Clone)]
pub struct DieselUserRepository {
    pool: DbPool,
}

impl DieselUserRepository {
    pub fn new(pool: DbPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl UserRepository for DieselUserRepository {
    #[instrument(skip(self, user))]
    async fn create(&self, user: NewUser) -> AppResult<User> {
        let mut conn = self.pool.get().map_err(|e| {
            tracing::error!("Connection pool error: {:?}", e);
            AppError::Internal
        })?;
        diesel::insert_into(users::table)
            .values(&user)
            .get_result::<User>(&mut conn)
            .map_err(|e| {
                tracing::error!("Database error: {:?}", e);
                AppError::Internal
            })
    }

    #[instrument(skip(self))]
    async fn find_by_email(&self, email: &str) -> AppResult<Option<User>> {
        let email = email.to_lowercase();
        let mut conn = self.pool.get().map_err(|e| {
            tracing::error!("Connection pool error: {:?}", e);
            AppError::Internal
        })?;
        users::table
            .filter(users::email.eq(email))
            .first::<User>(&mut conn)
            .optional()
            .map_err(|e| {
                tracing::error!("Database error: {:?}", e);
                AppError::Internal
            })
    }

    #[instrument(skip(self))]
    async fn find_by_id(&self, id: Uuid) -> AppResult<Option<User>> {
        let mut conn = self.pool.get().map_err(|e| {
            tracing::error!("Connection pool error: {:?}", e);
            AppError::Internal
        })?;
        users::table
            .find(id)
            .first::<User>(&mut conn)
            .optional()
            .map_err(|e| {
                tracing::error!("Database error: {:?}", e);
                AppError::Internal
            })
    }

    #[instrument(skip(self))]
    async fn update_password_hash(&self, id: Uuid, password_hash: String) -> AppResult<()> {
        let mut conn = self.pool.get().map_err(|e| {
            tracing::error!("Connection pool error: {:?}", e);
            AppError::Internal
        })?;
        let affected = diesel::update(users::table.find(id))
            .set(users::password_hash.eq(password_hash))
            .execute(&mut conn)
            .map_err(|e| {
                tracing::error!("Database error: {:?}", e);
                AppError::Internal
            })?;
        if affected == 0 {
            return Err(AppError::NotFound("User not found".to_string()));
        }
        Ok(())
    }
}

#[derive(Clone)]
pub struct DieselSessionRepository {
    pool: DbPool,
}

impl DieselSessionRepository {
    pub fn new(pool: DbPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl SessionRepository for DieselSessionRepository {
    #[instrument(skip(self), fields(user_id = %user_id))]
    async fn create(
        &self,
        user_id: Uuid,
        refresh_token: String,
        expires_at: NaiveDateTime,
    ) -> AppResult<()> {
        let mut conn = self.pool.get().map_err(|e| {
            tracing::error!("Connection pool error: {:?}", e);
            AppError::Internal
        })?;
        let id = Uuid::now_v7();
        diesel::insert_into(sessions::table)
            .values((
                sessions::id.eq(id),
                sessions::user_id.eq(user_id),
                sessions::refresh_token.eq(refresh_token),
                sessions::expires_at.eq(expires_at),
            ))
            .execute(&mut conn)
            .map_err(|e| {
                tracing::error!("Database error: {:?}", e);
                AppError::Internal
            })?;
        Ok(())
    }

    #[instrument(skip(self, refresh_token))]
    async fn find_by_token(&self, refresh_token: &str) -> AppResult<Option<SessionInfo>> {
        let mut conn = self.pool.get().map_err(|e| {
            tracing::error!("Connection pool error: {:?}", e);
            AppError::Internal
        })?;
        sessions::table
            .filter(sessions::refresh_token.eq(refresh_token))
            .select((sessions::user_id, sessions::expires_at))
            .first::<(Uuid, NaiveDateTime)>(&mut conn)
            .optional()
            .map_err(|e| {
                tracing::error!("Database error: {:?}", e);
                AppError::Internal
            })
            .map(|opt| {
                opt.map(|(user_id, expires_at)| SessionInfo {
                    user_id,
                    expires_at,
                })
            })
    }

    #[instrument(skip(self, refresh_token))]
    async fn revoke(&self, refresh_token: &str) -> AppResult<()> {
        let mut conn = self.pool.get().map_err(|e| {
            tracing::error!("Connection pool error: {:?}", e);
            AppError::Internal
        })?;
        diesel::delete(sessions::table.filter(sessions::refresh_token.eq(refresh_token)))
            .execute(&mut conn)
            .map_err(|e| {
                tracing::error!("Database error: {:?}", e);
                AppError::Internal
            })?;
        Ok(())
    }

    #[instrument(skip(self), fields(user_id = %user_id))]
    async fn revoke_all_for_user(&self, user_id: Uuid) -> AppResult<()> {
        let mut conn = self.pool.get().map_err(|e| {
            tracing::error!("Connection pool error: {:?}", e);
            AppError::Internal
        })?;
        diesel::delete(sessions::table.filter(sessions::user_id.eq(user_id)))
            .execute(&mut conn)
            .map_err(|e| {
                tracing::error!("Database error: {:?}", e);
                AppError::Internal
            })?;
        Ok(())
    }
}

#[derive(Clone)]
pub struct DieselPasswordResetRepository {
    pool: DbPool,
}

impl DieselPasswordResetRepository {
    pub fn new(pool: DbPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl PasswordResetRepository for DieselPasswordResetRepository {
    #[instrument(skip(self, reset))]
    async fn create(&self, reset: NewPasswordReset) -> AppResult<PasswordReset> {
        let mut conn = self.pool.get().map_err(|e| {
            tracing::error!("Connection pool error: {:?}", e);
            AppError::Internal
        })?;
        let id = Uuid::now_v7();
        diesel::insert_into(password_resets::table)
            .values((
                password_resets::id.eq(id),
                password_resets::user_id.eq(reset.user_id),
                password_resets::token_hash.eq(&reset.token_hash),
                password_resets::expires_at.eq(reset.expires_at),
            ))
            .execute(&mut conn)
            .map_err(|e| {
                tracing::error!("Database error inserting password reset: {:?}", e);
                AppError::Internal
            })?;
        password_resets::table
            .find(id)
            .first::<PasswordReset>(&mut conn)
            .map_err(|e| {
                tracing::error!("Database error fetching password reset: {:?}", e);
                AppError::Internal
            })
    }

    #[instrument(skip(self, token_hash))]
    async fn find_by_token_hash(&self, token_hash: &str) -> AppResult<Option<PasswordReset>> {
        let mut conn = self.pool.get().map_err(|e| {
            tracing::error!("Connection pool error: {:?}", e);
            AppError::Internal
        })?;
        password_resets::table
            .filter(password_resets::token_hash.eq(token_hash))
            .first::<PasswordReset>(&mut conn)
            .optional()
            .map_err(|e| {
                tracing::error!("Database error: {:?}", e);
                AppError::Internal
            })
    }

    #[instrument(skip(self), fields(id = %id))]
    async fn mark_used(&self, id: Uuid) -> AppResult<()> {
        let mut conn = self.pool.get().map_err(|e| {
            tracing::error!("Connection pool error: {:?}", e);
            AppError::Internal
        })?;
        let affected = diesel::update(password_resets::table.find(id))
            .set(password_resets::used_at.eq(chrono::Utc::now().naive_utc()))
            .execute(&mut conn)
            .map_err(|e| {
                tracing::error!("Database error: {:?}", e);
                AppError::Internal
            })?;
        if affected == 0 {
            return Err(AppError::NotFound("Password reset not found".to_string()));
        }
        Ok(())
    }
}
