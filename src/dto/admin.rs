use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use crate::dto::{UserResponse, PostResponse};
use serde_json::Value as JsonValue;

/// Admin settings response
#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct AdminSettingsResponse {
    pub allow_external_registration: bool,
    pub maintenance_mode: bool,
}

/// Admin settings update request
#[derive(Debug, Deserialize, ToSchema)]
pub struct AdminSettingsUpdateRequest {
    pub allow_external_registration: Option<bool>,
    pub maintenance_mode: Option<bool>,
}

/// Admin user list response (with ban status)
#[derive(Debug, Serialize, ToSchema)]
pub struct AdminUserListResponse {
    pub user: UserResponse,
    pub is_banned: bool,
}

/// Admin post list response
#[derive(Debug, Serialize, ToSchema)]
pub struct AdminPostListResponse {
    pub post: PostResponse,
}

/// Admin ban user request
#[derive(Debug, Deserialize, ToSchema)]
pub struct AdminBanUserRequest {
    pub banned: bool,
}

/// Admin reset password request
#[derive(Debug, Deserialize, ToSchema)]
pub struct AdminResetPasswordRequest {
    pub new_password: String,
}

/// Admin reset password response
#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct AdminResetPasswordResponse {
    pub success: bool,
    pub message: String,
}

/// Admin plugin list response
#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct AdminPluginListResponse {
    pub id: i64,
    pub name: String,
    pub description: Option<String>,
    pub version: String,
    pub enabled: bool,
    pub config: Option<JsonValue>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Admin plugin update request
#[derive(Debug, Deserialize, ToSchema)]
pub struct AdminPluginUpdateRequest {
    pub enabled: Option<bool>,
    pub config: Option<JsonValue>,
}
