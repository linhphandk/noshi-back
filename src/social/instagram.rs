use reqwest::Client;
use serde::Deserialize;
use std::collections::HashMap;
use tracing::{debug, error, instrument};
use url::Url;

use crate::shared::errors::{AppError, AppResult};
use crate::social::models::{SocialInsights, SocialProfile, SocialTokens};
use crate::social::ports::SocialProvider;

const DEFAULT_AUTH_URL: &str = "https://api.instagram.com/oauth/authorize";
const DEFAULT_TOKEN_URL: &str = "https://api.instagram.com/oauth/access_token";
const DEFAULT_EXCHANGE_URL: &str = "https://graph.instagram.com/access_token";
const DEFAULT_REFRESH_URL: &str = "https://graph.instagram.com/refresh_access_token";
const DEFAULT_GRAPH_URL: &str = "https://graph.instagram.com";

#[derive(Deserialize)]
struct TokenExchangeResponse {
    access_token: String,
    user_id: String,
}

#[derive(Deserialize)]
#[allow(dead_code)]
struct LongLivedTokenResponse {
    access_token: String,
    token_type: String,
    expires_in: i64,
}

#[derive(Deserialize)]
#[allow(dead_code)]
struct ProfileResponse {
    username: String,
    followers_count: i32,
    biography: Option<String>,
    profile_picture_url: Option<String>,
    media_count: i32,
}

#[derive(Deserialize)]
struct InsightsResponse {
    data: Vec<InsightData>,
}

#[derive(Deserialize)]
struct InsightData {
    name: String,
    total_value: Option<InsightTotalValue>,
}

#[derive(Deserialize)]
struct InsightTotalValue {
    value: i64,
    breakdowns: Option<Vec<InsightBreakdown>>,
}

#[derive(Deserialize)]
struct InsightBreakdown {
    dimension_keys: Vec<String>,
    results: Vec<InsightBreakdownResult>,
}

#[derive(Deserialize)]
struct InsightBreakdownResult {
    dimension_values: Vec<String>,
    value: i64,
}

#[derive(Deserialize)]
struct DemographicsResponse {
    data: Vec<DemographicsData>,
}

#[derive(Deserialize)]
struct DemographicsData {
    total_value: Option<InsightTotalValue>,
}

pub struct InstagramProvider {
    client: Client,
    client_id: String,
    client_secret: String,
    redirect_uri: String,
    auth_url: String,
    token_url: String,
    exchange_url: String,
    refresh_url: String,
    graph_url: String,
}

impl InstagramProvider {
    pub fn new(client_id: String, client_secret: String, redirect_uri: String) -> Self {
        Self::with_custom_urls(
            client_id,
            client_secret,
            redirect_uri,
            DEFAULT_AUTH_URL.to_string(),
            DEFAULT_TOKEN_URL.to_string(),
            DEFAULT_EXCHANGE_URL.to_string(),
            DEFAULT_REFRESH_URL.to_string(),
            DEFAULT_GRAPH_URL.to_string(),
        )
    }

    #[allow(clippy::too_many_arguments)]
    pub fn with_custom_urls(
        client_id: String,
        client_secret: String,
        redirect_uri: String,
        auth_url: String,
        token_url: String,
        exchange_url: String,
        refresh_url: String,
        graph_url: String,
    ) -> Self {
        Self {
            client: Client::new(),
            client_id,
            client_secret,
            redirect_uri,
            auth_url,
            token_url,
            exchange_url,
            refresh_url,
            graph_url,
        }
    }
}

impl InstagramProvider {
    fn build_authorize_url(&self, state: &str) -> String {
        let mut url = Url::parse(&self.auth_url).unwrap();
        url.query_pairs_mut()
            .append_pair("client_id", &self.client_id)
            .append_pair("redirect_uri", &self.redirect_uri)
            .append_pair("response_type", "code")
            .append_pair(
                "scope",
                "instagram_business_basic,instagram_business_manage_insights",
            )
            .append_pair("state", state);
        url.to_string()
    }

    async fn exchange_code_for_short(&self, code: &str) -> AppResult<TokenExchangeResponse> {
        let mut params = HashMap::new();
        params.insert("client_id", self.client_id.clone());
        params.insert("client_secret", self.client_secret.clone());
        params.insert("grant_type", "authorization_code".to_string());
        params.insert("redirect_uri", self.redirect_uri.clone());
        params.insert("code", code.to_string());

        let resp = self
            .client
            .post(&self.token_url)
            .form(&params)
            .send()
            .await
            .map_err(|e| {
                error!("Token exchange request failed: {:?}", e);
                AppError::Internal
            })?;

        if !resp.status().is_success() {
            let err = resp.text().await.unwrap_or_default();
            error!("Token exchange failed: {}", err);
            return Err(AppError::Internal);
        }

        resp.json::<TokenExchangeResponse>().await.map_err(|e| {
            error!("Token exchange parse failed: {:?}", e);
            AppError::Internal
        })
    }

    async fn exchange_short_for_long(
        &self,
        short_token: &str,
    ) -> AppResult<LongLivedTokenResponse> {
        let mut url = Url::parse(&self.exchange_url).unwrap();
        url.query_pairs_mut()
            .append_pair("grant_type", "ig_exchange_token")
            .append_pair("client_secret", &self.client_secret)
            .append_pair("access_token", short_token);

        let resp = self.client.get(url).send().await.map_err(|e| {
            error!("Long-lived token exchange failed: {:?}", e);
            AppError::Internal
        })?;

        if !resp.status().is_success() {
            let err = resp.text().await.unwrap_or_default();
            error!("Long-lived token exchange failed: {}", err);
            return Err(AppError::Internal);
        }

        resp.json::<LongLivedTokenResponse>().await.map_err(|e| {
            error!("Long-lived token parse failed: {:?}", e);
            AppError::Internal
        })
    }

    async fn refresh_long_lived(&self, token: &str) -> AppResult<LongLivedTokenResponse> {
        let mut url = Url::parse(&self.refresh_url).unwrap();
        url.query_pairs_mut()
            .append_pair("grant_type", "ig_refresh_token")
            .append_pair("access_token", token);

        let resp = self.client.get(url).send().await.map_err(|e| {
            error!("Token refresh failed: {:?}", e);
            AppError::Internal
        })?;

        if !resp.status().is_success() {
            let err = resp.text().await.unwrap_or_default();
            error!("Token refresh failed: {}", err);
            return Err(AppError::Internal);
        }

        resp.json::<LongLivedTokenResponse>().await.map_err(|e| {
            error!("Token refresh parse failed: {:?}", e);
            AppError::Internal
        })
    }
}

#[async_trait::async_trait]
impl SocialProvider for InstagramProvider {
    fn platform(&self) -> &str {
        "instagram"
    }

    fn authorize_url(&self, state: &str) -> String {
        self.build_authorize_url(state)
    }

    #[instrument(skip(self))]
    async fn exchange_code(&self, code: &str) -> AppResult<SocialTokens> {
        debug!("Exchanging code for tokens");
        let short = self.exchange_code_for_short(code).await?;
        debug!("Got short-lived token, exchanging for long-lived");
        let long = self.exchange_short_for_long(&short.access_token).await?;

        let expires_at = chrono::Utc::now() + chrono::Duration::seconds(long.expires_in);

        Ok(SocialTokens {
            access_token: long.access_token.clone(),
            refresh_token: Some(long.access_token),
            expires_at: expires_at.naive_utc(),
            platform_user_id: short.user_id,
        })
    }

    #[instrument(skip(self))]
    async fn refresh_token(&self, token: &str) -> AppResult<SocialTokens> {
        debug!("Refreshing long-lived token");
        let resp = self.refresh_long_lived(token).await?;
        let expires_at = chrono::Utc::now() + chrono::Duration::seconds(resp.expires_in);

        Ok(SocialTokens {
            access_token: resp.access_token.clone(),
            refresh_token: Some(resp.access_token),
            expires_at: expires_at.naive_utc(),
            platform_user_id: String::new(), // will be fetched from profile if needed
        })
    }

    #[instrument(skip(self))]
    async fn fetch_profile(&self, token: &str) -> AppResult<SocialProfile> {
        debug!("Fetching Instagram profile");
        let mut url = Url::parse(&format!("{}/me", self.graph_url)).unwrap();
        url.query_pairs_mut()
            .append_pair(
                "fields",
                "username,followers_count,biography,profile_picture_url,media_count",
            )
            .append_pair("access_token", token);

        let resp = self.client.get(url).send().await.map_err(|e| {
            error!("Profile fetch failed: {:?}", e);
            AppError::Internal
        })?;

        if !resp.status().is_success() {
            let err = resp.text().await.unwrap_or_default();
            error!("Profile fetch failed: {}", err);
            return Err(AppError::Internal);
        }

        let profile: ProfileResponse = resp.json().await.map_err(|e| {
            error!("Profile parse failed: {:?}", e);
            AppError::Internal
        })?;

        Ok(SocialProfile {
            handle: profile.username,
            follower_count: profile.followers_count,
            profile_picture_url: profile.profile_picture_url,
        })
    }

    #[instrument(skip(self))]
    async fn fetch_insights(
        &self,
        token: &str,
        platform_user_id: &str,
    ) -> AppResult<SocialInsights> {
        debug!("Fetching Instagram insights");

        let engagement_rate = self.fetch_engagement_rate(token, platform_user_id).await?;
        let demographics = self.fetch_demographics(token, platform_user_id).await?;

        Ok(SocialInsights {
            engagement_rate,
            audience_demographics: demographics,
        })
    }
}

impl InstagramProvider {
    async fn fetch_engagement_rate(
        &self,
        token: &str,
        platform_user_id: &str,
    ) -> AppResult<Option<f64>> {
        let since = (chrono::Utc::now() - chrono::Duration::days(30)).timestamp();
        let until = chrono::Utc::now().timestamp();

        let mut url =
            Url::parse(&format!("{}/{}/insights", self.graph_url, platform_user_id)).unwrap();
        url.query_pairs_mut()
            .append_pair("metric", "reach,accounts_engaged,total_interactions")
            .append_pair("period", "day")
            .append_pair("metric_type", "total_value")
            .append_pair("since", &since.to_string())
            .append_pair("until", &until.to_string())
            .append_pair("access_token", token);

        let resp = self.client.get(url).send().await.map_err(|e| {
            error!("Insights fetch failed: {:?}", e);
            AppError::Internal
        })?;

        if !resp.status().is_success() {
            let err = resp.text().await.unwrap_or_default();
            error!("Insights fetch failed: {}", err);
            return Ok(None);
        }

        let insights: InsightsResponse = resp.json().await.map_err(|e| {
            error!("Insights parse failed: {:?}", e);
            AppError::Internal
        })?;

        let mut reach = 0i64;
        let mut accounts_engaged = 0i64;

        for insight in insights.data {
            if let Some(total) = insight.total_value {
                match insight.name.as_str() {
                    "reach" => reach = total.value,
                    "accounts_engaged" => accounts_engaged = total.value,
                    _ => {}
                }
            }
        }

        if reach > 0 {
            Ok(Some((accounts_engaged as f64 / reach as f64) * 100.0))
        } else {
            Ok(None)
        }
    }

    async fn fetch_demographics(
        &self,
        token: &str,
        platform_user_id: &str,
    ) -> AppResult<Option<serde_json::Value>> {
        let mut url =
            Url::parse(&format!("{}/{}/insights", self.graph_url, platform_user_id)).unwrap();
        url.query_pairs_mut()
            .append_pair("metric", "follower_demographics")
            .append_pair("period", "lifetime")
            .append_pair("timeframe", "this_month")
            .append_pair("breakdown", "age,gender,country")
            .append_pair("metric_type", "total_value")
            .append_pair("access_token", token);

        let resp = self.client.get(url).send().await.map_err(|e| {
            error!("Demographics fetch failed: {:?}", e);
            AppError::Internal
        })?;

        if !resp.status().is_success() {
            let err = resp.text().await.unwrap_or_default();
            error!("Demographics fetch failed: {}", err);
            return Ok(None);
        }

        let demographics: DemographicsResponse = resp.json().await.map_err(|e| {
            error!("Demographics parse failed: {:?}", e);
            AppError::Internal
        })?;

        let mut result = serde_json::json!({});

        for demo in demographics.data {
            if let Some(total) = demo.total_value {
                if let Some(breakdowns) = total.breakdowns {
                    for breakdown in breakdowns {
                        for key in &breakdown.dimension_keys {
                            match key.as_str() {
                                "age" => {
                                    let mut age_data = Vec::new();
                                    for res in &breakdown.results {
                                        age_data.push(serde_json::json!({
                                            "value": res.dimension_values[0],
                                            "count": res.value
                                        }));
                                    }
                                    result["age"] = serde_json::json!(age_data);
                                }
                                "gender" => {
                                    let mut gender_data = serde_json::json!({});
                                    for res in &breakdown.results {
                                        gender_data[res.dimension_values[0].to_lowercase()] =
                                            serde_json::json!(res.value);
                                    }
                                    result["gender"] = gender_data;
                                }
                                "country" => {
                                    let mut country_data = Vec::new();
                                    for res in &breakdown.results {
                                        country_data.push(serde_json::json!({
                                            "value": res.dimension_values[0],
                                            "count": res.value
                                        }));
                                    }
                                    result["countries"] = serde_json::json!(country_data);
                                }
                                _ => {}
                            }
                        }
                    }
                }
            }
        }

        if result.as_object().is_none_or(|o| o.is_empty()) {
            Ok(None)
        } else {
            Ok(Some(result))
        }
    }
}
