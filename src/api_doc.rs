use utoipa::OpenApi;

use crate::auth::controller::{AuthResponse, UserResponse};
use crate::auth::models::{
    ForgotPasswordRequest, LoginRequest, RegisterRequest, ResetPasswordRequest,
};
use crate::profile::models::{
    CreateManualPlatformRequest, CreateProfileRequest, ManualPlatform, Profile,
    UpdateManualPlatformRequest, UpdateProfileRequest,
};

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
            crate::shared::errors::ErrorResponse,
        ),
    ),
    tags(
        (name = "auth", description = "Authentication endpoints"),
        (name = "profile", description = "User profile endpoints"),
    ),
)]
pub struct ApiDoc;
