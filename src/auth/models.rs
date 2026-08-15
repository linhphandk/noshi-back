use chrono::NaiveDateTime;
use diesel::prelude::*;
use uuid::Uuid;

#[derive(Debug, Clone)]
pub struct Tokens {
    pub access_token: String,
    pub expires_in: u64,
}

#[derive(Debug, Clone)]
pub struct UserInfo {
    pub sub: String,
    pub email: String,
    pub name: String,
    pub password_hash: String,
}

#[derive(Queryable, Selectable, Clone, Debug)]
#[diesel(table_name = crate::schema::users)]
pub struct User {
    pub id: Uuid,
    pub email: String,
    pub name: String,
    pub password_hash: String,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
}

#[derive(Insertable)]
#[diesel(table_name = crate::schema::users)]
pub struct NewUser {
    pub id: Uuid,
    pub email: String,
    pub name: String,
    pub password_hash: String,
}

#[derive(Queryable, Selectable, Clone, Debug)]
#[diesel(table_name = crate::schema::sessions)]
pub struct Session {
    pub id: Uuid,
    pub user_id: Uuid,
    pub refresh_token: String,
    pub expires_at: NaiveDateTime,
    pub created_at: NaiveDateTime,
}

#[derive(Queryable, Selectable, Clone, Debug)]
#[diesel(table_name = crate::schema::password_resets)]
pub struct PasswordReset {
    pub id: Uuid,
    pub user_id: Uuid,
    pub token_hash: String,
    pub expires_at: NaiveDateTime,
    pub used_at: Option<NaiveDateTime>,
    pub created_at: NaiveDateTime,
}

#[derive(Insertable)]
#[diesel(table_name = crate::schema::password_resets)]
pub struct NewPasswordReset {
    pub user_id: Uuid,
    pub token_hash: String,
    pub expires_at: NaiveDateTime,
}
