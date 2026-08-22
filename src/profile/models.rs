use chrono::NaiveDateTime;
use diesel::prelude::*;
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use uuid::Uuid;

#[derive(Queryable, Selectable, Clone, Debug, Serialize, Deserialize, ToSchema)]
#[diesel(table_name = crate::schema::profiles)]
pub struct Profile {
    pub id: Uuid,
    pub user_id: Uuid,
    pub slug: String,
    pub niches: Vec<Option<String>>,
    pub headline: String,
    pub is_published: bool,
    pub completion_score: i32,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
}

#[derive(Insertable)]
#[diesel(table_name = crate::schema::profiles)]
pub struct NewProfile {
    pub id: Uuid,
    pub user_id: Uuid,
    pub slug: String,
    pub niches: Vec<Option<String>>,
    pub headline: String,
}

#[derive(AsChangeset)]
#[diesel(table_name = crate::schema::profiles)]
pub struct UpdateProfile {
    pub slug: Option<String>,
    pub niches: Option<Vec<Option<String>>>,
    pub headline: Option<String>,
    pub is_published: Option<bool>,
    pub completion_score: Option<i32>,
}

#[derive(Debug, Deserialize, ToSchema)]
pub struct CreateProfileRequest {
    pub slug: String,
    pub niches: Vec<String>,
    pub headline: String,
}

#[derive(Debug, Deserialize, ToSchema)]
pub struct UpdateProfileRequest {
    pub slug: Option<String>,
    pub niches: Option<Vec<String>>,
    pub headline: Option<String>,
    pub is_published: Option<bool>,
}

#[derive(Queryable, Selectable, Clone, Debug, Serialize, Deserialize, ToSchema)]
#[diesel(table_name = crate::schema::manual_platforms)]
pub struct ManualPlatform {
    pub id: Uuid,
    pub user_id: Uuid,
    pub platform: String,
    pub handle: String,
    pub follower_count: i32,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
}

#[derive(Insertable)]
#[diesel(table_name = crate::schema::manual_platforms)]
pub struct NewManualPlatform {
    pub user_id: Uuid,
    pub platform: String,
    pub handle: String,
    pub follower_count: i32,
}

#[derive(AsChangeset)]
#[diesel(table_name = crate::schema::manual_platforms)]
pub struct UpdateManualPlatform {
    pub platform: Option<String>,
    pub handle: Option<String>,
    pub follower_count: Option<i32>,
}

#[derive(Debug, Deserialize, ToSchema)]
pub struct CreateManualPlatformRequest {
    pub platform: String,
    pub handle: String,
    pub follower_count: i32,
}

#[derive(Debug, Deserialize, ToSchema)]
pub struct UpdateManualPlatformRequest {
    pub platform: Option<String>,
    pub handle: Option<String>,
    pub follower_count: Option<i32>,
}

#[derive(Serialize, ToSchema)]
pub struct PublicProfile {
    #[serde(flatten)]
    pub profile: Profile,
    pub platforms: Vec<ManualPlatform>,
}
