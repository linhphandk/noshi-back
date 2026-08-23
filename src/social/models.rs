use chrono::NaiveDateTime;
use diesel::prelude::*;
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use uuid::Uuid;

#[derive(Queryable, Selectable, Clone, Debug, Serialize, Deserialize, ToSchema)]
#[diesel(table_name = crate::schema::social_connections)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct SocialConnection {
    pub id: Uuid,
    pub user_id: Uuid,
    pub platform: String,
    pub platform_user_id: String,
    pub handle: String,
    pub access_token_encrypted: Vec<u8>,
    pub refresh_token_encrypted: Option<Vec<u8>>,
    pub token_expires_at: NaiveDateTime,
    pub follower_count: i32,
    pub engagement_rate: Option<f64>,
    pub audience_demographics: Option<serde_json::Value>,
    pub last_synced_at: Option<NaiveDateTime>,
    pub is_primary: bool,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
}

#[derive(Insertable)]
#[diesel(table_name = crate::schema::social_connections)]
pub struct NewSocialConnection {
    pub id: Uuid,
    pub user_id: Uuid,
    pub platform: String,
    pub platform_user_id: String,
    pub handle: String,
    pub access_token_encrypted: Vec<u8>,
    pub refresh_token_encrypted: Option<Vec<u8>>,
    pub token_expires_at: NaiveDateTime,
    pub follower_count: i32,
    pub is_primary: bool,
}

#[derive(AsChangeset, Default)]
#[diesel(table_name = crate::schema::social_connections)]
pub struct UpdateSocialConnection {
    pub platform: Option<String>,
    pub platform_user_id: Option<String>,
    pub handle: Option<String>,
    pub access_token_encrypted: Option<Vec<u8>>,
    pub refresh_token_encrypted: Option<Vec<u8>>,
    pub token_expires_at: Option<NaiveDateTime>,
    pub follower_count: Option<i32>,
    pub engagement_rate: Option<f64>,
    pub last_synced_at: Option<NaiveDateTime>,
    pub is_primary: Option<bool>,
}

#[derive(Debug, Deserialize, ToSchema)]
pub struct ConnectSocialRequest {
    pub platform: String,
    pub code: String,
    pub state: String,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct AuthorizeUrlResponse {
    pub authorize_url: String,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct SocialConnectionResponse {
    pub id: Uuid,
    pub platform: String,
    pub handle: String,
    pub follower_count: i32,
    pub engagement_rate: Option<f64>,
    pub audience_demographics: Option<serde_json::Value>,
    pub last_synced_at: Option<NaiveDateTime>,
    pub is_primary: bool,
    pub created_at: NaiveDateTime,
}

impl From<SocialConnection> for SocialConnectionResponse {
    fn from(c: SocialConnection) -> Self {
        Self {
            id: c.id,
            platform: c.platform,
            handle: c.handle,
            follower_count: c.follower_count,
            engagement_rate: c.engagement_rate,
            audience_demographics: c.audience_demographics,
            last_synced_at: c.last_synced_at,
            is_primary: c.is_primary,
            created_at: c.created_at,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SocialTokens {
    pub access_token: String,
    pub refresh_token: Option<String>,
    pub expires_at: NaiveDateTime,
    pub platform_user_id: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SocialProfile {
    pub handle: String,
    pub follower_count: i32,
    pub profile_picture_url: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SocialInsights {
    pub engagement_rate: Option<f64>,
    pub audience_demographics: Option<serde_json::Value>,
}
