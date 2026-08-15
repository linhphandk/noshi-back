use crate::schema::waitlist;
use crate::shared::db::DbPool;
use crate::shared::errors::{AppError, AppResult};
use crate::waitlist::models::{NewWaitlistEntry, WaitlistEntry};
use crate::waitlist::ports::WaitlistRepository;
use async_trait::async_trait;
use diesel::prelude::*;
use diesel::result::DatabaseErrorKind;
use tracing::instrument;

#[derive(Clone)]
pub struct DieselWaitlistRepository {
    pool: DbPool,
}

impl DieselWaitlistRepository {
    pub fn new(pool: DbPool) -> Self {
        Self { pool }
    }
}

fn map_diesel_error(e: diesel::result::Error, context: &str) -> AppError {
    match e {
        diesel::result::Error::DatabaseError(DatabaseErrorKind::UniqueViolation, _) => {
            AppError::BadRequest(format!("{} already exists", context))
        }
        _ => {
            tracing::error!("Database error: {:?}", e);
            AppError::Internal
        }
    }
}

#[async_trait]
impl WaitlistRepository for DieselWaitlistRepository {
    #[instrument(skip(self))]
    async fn create(&self, entry: NewWaitlistEntry) -> AppResult<WaitlistEntry> {
        let mut conn = self.pool.get().map_err(|e| {
            tracing::error!("Connection pool error: {:?}", e);
            AppError::Internal
        })?;

        diesel::insert_into(waitlist::table)
            .values(&entry)
            .returning(waitlist::all_columns)
            .get_result::<WaitlistEntry>(&mut conn)
            .map_err(|e| map_diesel_error(e, "Email"))
    }

    #[instrument(skip(self))]
    async fn find_by_email(&self, email: &str) -> AppResult<Option<WaitlistEntry>> {
        let mut conn = self.pool.get().map_err(|e| {
            tracing::error!("Connection pool error: {:?}", e);
            AppError::Internal
        })?;

        waitlist::table
            .filter(waitlist::email.eq(email))
            .first::<WaitlistEntry>(&mut conn)
            .optional()
            .map_err(|e| {
                tracing::error!("Database error: {:?}", e);
                AppError::Internal
            })
    }
}
