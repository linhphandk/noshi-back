use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct Config {
    pub server_host: String,
    pub server_port: u16,
    pub database_url: String,
    pub database_pool_size: u32,
    pub jwt_secret: String,
    pub jwt_expiry_minutes: u64,
    pub aws_s3_bucket: Option<String>,
    pub aws_region: Option<String>,
    pub aws_endpoint: Option<String>,
    pub s3_public_url: Option<String>,
    pub ses_from_email: Option<String>,
    pub ses_from_name: Option<String>,
    pub frontend_url: Option<String>,
    pub instagram_client_id: Option<String>,
    pub instagram_client_secret: Option<String>,
    pub instagram_redirect_uri: Option<String>,
    pub token_encryption_key: Option<String>,
    pub social_oauth_state_secret: Option<String>,
}

impl Config {
    pub fn from_env() -> Result<Self, envy::Error> {
        envy::from_env()
    }
}
