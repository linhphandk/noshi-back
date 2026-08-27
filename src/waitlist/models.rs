use chrono::NaiveDateTime;
use diesel::prelude::*;
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use uuid::Uuid;

#[derive(Queryable, Selectable, Serialize, Clone, Debug)]
#[diesel(table_name = crate::schema::waitlist)]
pub struct WaitlistEntry {
    pub id: Uuid,
    pub email: String,
    pub signed_up_at: NaiveDateTime,
}

#[derive(Insertable)]
#[diesel(table_name = crate::schema::waitlist)]
pub struct NewWaitlistEntry {
    pub email: String,
}

impl std::fmt::Debug for NewWaitlistEntry {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("NewWaitlistEntry")
            .field("email", &self.email)
            .finish()
    }
}

#[derive(Deserialize, ToSchema)]
pub struct JoinWaitlistRequest {
    pub email: String,
}

impl std::fmt::Debug for JoinWaitlistRequest {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("JoinWaitlistRequest")
            .field("email", &self.email)
            .finish()
    }
}

#[derive(Serialize, ToSchema)]
pub struct WaitlistResponse {
    pub message: String,
}
