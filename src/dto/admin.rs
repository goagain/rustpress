use crate::dto::{PostResponse, UserResponse};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::Value as JsonValue;
use utoipa::ToSchema;

/// Setting item in a settings tab
#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct SettingItem {
    pub key: String,
    pub value: JsonValue,
    pub label: String,
    pub description: Option<String>,
    pub input_type: String, // "checkbox", "text", "password", "number", "textarea", etc.
}

/// Settings tab
#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct SettingsTab {
    pub id: String,
    pub label: String,
    pub description: Option<String>,
    pub items: Vec<SettingItem>,
}

/// Admin settings tabs response
#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct AdminSettingsTabsResponse {
    pub tabs: Vec<SettingsTab>,
}

/// Admin settings update request (key-value pairs)
#[derive(Debug, Deserialize, ToSchema)]
pub struct AdminSettingsUpdateRequest {
    pub settings: std::collections::HashMap<String, JsonValue>,
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

/// Admin OpenAI API test response
#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct AdminOpenAITestResponse {
    pub success: bool,
    pub message: String,
}
