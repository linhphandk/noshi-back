use chrono::NaiveDateTime;
use diesel::prelude::*;
use serde::Serialize;
use uuid::Uuid;

#[derive(Queryable, Selectable, Serialize, Clone, Debug)]
#[diesel(table_name = crate::schema::waitlist)]
pub struct WaitlistEntry {
    pub id: Uuid,
    pub email: String,
    pub position: i32,
    pub profile_complete_at: Option<NaiveDateTime>,
    pub is_featured: bool,
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

#[derive(Serialize)]
pub struct WaitlistResponse {
    pub position: i32,
    pub message: String,
}
