use utoipa::OpenApi;

use crate::auth::controller::{AuthResponse, UserResponse};
use crate::auth::models::{
    ForgotPasswordRequest, LoginRequest, RegisterRequest, ResetPasswordRequest,
};
use crate::profile::models::{
    CreateManualPlatformRequest, CreateProfileRequest, ManualPlatform, Profile, PublicProfile,
    UpdateManualPlatformRequest, UpdateProfileRequest,
};
use crate::social::models::{AuthorizeUrlResponse, ConnectSocialRequest, SocialConnectionResponse};

#[derive(OpenApi)]
#[openapi(
    paths(
        crate::auth::controller::register,
        crate::auth::controller::login,
        crate::auth::controller::logout,
        crate::auth::controller::refresh,
        crate::auth::controller::me,
        crate::auth::controller::forgot_password,
        crate::auth::controller::reset_password,
        crate::profile::controller::create_profile,
        crate::profile::controller::get_profile,
        crate::profile::controller::update_profile,
        crate::profile::controller::delete_profile,
        crate::profile::controller::add_manual_platform,
        crate::profile::controller::get_manual_platforms,
        crate::profile::controller::update_manual_platform,
        crate::profile::controller::delete_manual_platform,
        crate::profile::controller::get_public_profile,
        crate::social::controller::get_authorize_url,
        crate::social::controller::connect,
        crate::social::controller::list_connections,
        crate::social::controller::disconnect,
        crate::social::controller::sync,
    ),
    components(
        schemas(
            AuthResponse,
            UserResponse,
            RegisterRequest,
            LoginRequest,
            ForgotPasswordRequest,
            ResetPasswordRequest,
            Profile,
            CreateProfileRequest,
            UpdateProfileRequest,
            ManualPlatform,
            CreateManualPlatformRequest,
            UpdateManualPlatformRequest,
            PublicProfile,
            AuthorizeUrlResponse,
            ConnectSocialRequest,
            SocialConnectionResponse,
            crate::shared::errors::ErrorResponse,
        ),
    ),
    tags(
        (name = "auth", description = "Authentication endpoints"),
        (name = "profile", description = "User profile endpoints"),
        (name = "social", description = "Social platform connections"),
    ),
)]
pub struct ApiDoc;
