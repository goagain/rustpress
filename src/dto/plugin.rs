use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

/// Plugin permission types
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum PluginPermission {
    /// Permission to read posts
    #[serde(rename = "post:read")]
    PostRead,
    /// Permission to create/modify posts
    #[serde(rename = "post:write")]
    PostWrite,
    /// Permission to read users
    #[serde(rename = "user:read")]
    UserRead,
    /// Permission to create/modify users
    #[serde(rename = "user:write")]
    UserWrite,
    /// Permission to use AI/chat features
    #[serde(rename = "ai:chat")]
    AiChat,
    /// Permission to read settings
    #[serde(rename = "settings:read")]
    SettingsRead,
    /// Permission to modify settings
    #[serde(rename = "settings:write")]
    SettingsWrite,
    /// Permission to upload files
    #[serde(rename = "upload:write")]
    UploadWrite,
    /// Permission to read files
    #[serde(rename = "upload:read")]
    UploadRead,
}

/// Plugin hook types (actions and filters)
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema, PartialEq, Eq, Hash)]
#[serde(rename_all = "snake_case")]
pub enum PluginHook {
    /// Action hook for when a post is published (async, non-blocking)
    #[serde(rename = "action_post_published")]
    ActionPostPublished,
    /// Filter hook for when a post is published (sync, can modify data)
    #[serde(rename = "filter_post_published")]
    FilterPostPublished,
    /// Action hook for when a user is created
    #[serde(rename = "action_user_created")]
    ActionUserCreated,
    /// Filter hook for when a user is created
    #[serde(rename = "filter_user_created")]
    FilterUserCreated,
    /// Action hook for when a user logs in
    #[serde(rename = "action_user_login")]
    ActionUserLogin,
    /// Filter hook for authentication
    #[serde(rename = "filter_authenticate")]
    FilterAuthenticate,
}

/// Plugin manifest structure (TOML format)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginManifest {
    /// Package information
    pub package: PackageInfo,
    /// Permissions configuration
    #[serde(default)]
    pub permissions: PermissionsConfig,
    /// Hooks configuration
    #[serde(default)]
    pub hooks: HooksConfig,
    /// Optional permissions with descriptions (for backward compatibility)
    #[serde(default)]
    pub optional_permissions: std::collections::HashMap<String, String>,
}

/// Package information in manifest
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PackageInfo {
    /// Unique plugin identifier (e.g., "com.rui.editor")
    pub id: String,
    /// Human-readable plugin name
    pub name: String,
    /// Plugin version
    pub version: String,
    /// Plugin description
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    /// Author information
    #[serde(skip_serializing_if = "Option::is_none")]
    pub author: Option<String>,
}

/// Permissions configuration in manifest
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema, Default)]
pub struct PermissionsConfig {
    /// Required permissions that must be granted
    #[serde(default)]
    pub required: Vec<String>,
}

/// Hooks configuration in manifest
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema, Default)]
pub struct HooksConfig {
    /// Hooks that this plugin wants to register
    #[serde(default)]
    pub registered: Vec<String>,
}

/// Legacy JSON manifest structure (for backward compatibility)
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct PluginManifestLegacy {
    /// Unique plugin identifier (e.g., "goagain.summary")
    pub id: String,
    /// Human-readable plugin name
    pub name: String,
    /// Plugin version
    pub version: String,
    /// Plugin description
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    /// Author information
    #[serde(skip_serializing_if = "Option::is_none")]
    pub author: Option<String>,
    /// Required permissions for this plugin
    pub permissions: Vec<PluginPermission>,
    /// Hooks that this plugin wants to register
    #[serde(skip_serializing_if = "Vec::is_empty", default)]
    pub hooks: Vec<PluginHook>,
}

/// Plugin installation request (RPK file upload)
#[derive(Debug, Deserialize, ToSchema)]
pub struct PluginInstallRequest {
    /// Plugin RPK file (base64 encoded)
    pub rpk_data: String,
    /// Optional: Permission grants during installation
    #[serde(default)]
    pub permission_grants: std::collections::HashMap<String, bool>,
}

/// Plugin permission info
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct PluginPermissionInfo {
    pub permission: String,
    pub is_granted: bool,
    pub permission_type: String, // "required" or "optional"
    pub description: Option<String>,
}

/// Plugin permissions list response
#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct PluginPermissionsResponse {
    pub plugin_id: String,
    pub permissions: Vec<PluginPermissionInfo>,
}

/// Update plugin permissions request
#[derive(Debug, Deserialize, ToSchema)]
pub struct UpdatePluginPermissionsRequest {
    pub permissions: std::collections::HashMap<String, bool>,
}

/// Plugin status enum
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum PluginStatus {
    /// Plugin is enabled and running
    #[serde(rename = "enabled")]
    Enabled,
    /// Plugin is disabled
    #[serde(rename = "disabled")]
    Disabled,
    /// Plugin is pending permission review after update
    #[serde(rename = "pending_review")]
    PendingReview,
}

/// Plugin update analysis result
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct PluginUpdateAnalysis {
    pub plugin_id: String,
    pub current_version: String,
    pub new_version: String,
    pub status: PluginUpdateStatus,
    pub new_required_permissions: Vec<String>,
    pub new_optional_permissions: Vec<String>,
    pub message: String,
}

/// Plugin update status
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum PluginUpdateStatus {
    /// Update can proceed without issues
    Safe,
    /// Update requires permission review
    NeedsReview,
    /// Update would break security (should not be allowed)
    SecurityViolation,
}

/// Approve plugin permissions request
#[derive(Debug, Deserialize, ToSchema)]
pub struct ApprovePluginPermissionsRequest {
    pub approved_permissions: std::collections::HashMap<String, bool>,
}

/// Plugin execution context for hooks
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginExecutionContext {
    /// Plugin ID
    pub plugin_id: String,
    /// Hook type being executed
    pub hook: PluginHook,
    /// Execution data (depends on hook type)
    pub data: serde_json::Value,
}

/// Plugin execution result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginExecutionResult {
    /// Whether execution was successful
    pub success: bool,
    /// Result data (if any)
    pub data: Option<serde_json::Value>,
    /// Error message (if execution failed)
    pub error: Option<String>,
}

/// Access denied error for plugins
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct PluginAccessError {
    pub error: String,
    pub required_permission: PluginPermission,
    pub plugin_id: String,
}

/// Permission denied error for plugin API calls
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PermissionDeniedError {
    pub message: String,
    pub required_permission: String,
}

impl PermissionDeniedError {
    pub fn new(permission: &str) -> Self {
        Self {
            message: format!("Permission '{}' not granted", permission),
            required_permission: permission.to_string(),
        }
    }
}

impl std::fmt::Display for PermissionDeniedError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.message)
    }
}

impl std::error::Error for PermissionDeniedError {}

/// Post data structure for plugin hooks
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PostData {
    /// Post ID
    pub id: i64,
    /// Post title
    pub title: String,
    /// Post content (markdown)
    pub content: String,
    /// Post category
    pub category: String,
    /// Author ID
    pub author_id: i64,
    /// Creation timestamp
    pub created_at: chrono::DateTime<chrono::Utc>,
    /// Last update timestamp
    pub updated_at: chrono::DateTime<chrono::Utc>,
}

/// User data structure for plugin hooks
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserData {
    /// User ID
    pub id: i64,
    /// Username
    pub username: String,
    /// Email
    pub email: String,
    /// User role
    pub role: String,
    /// Creation timestamp
    pub created_at: chrono::DateTime<chrono::Utc>,
}

/// Authentication data structure for plugin hooks
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuthData {
    /// Username
    pub username: String,
    /// Password (for login attempts)
    pub password: String,
    /// IP address
    pub ip_address: Option<String>,
    /// User agent
    pub user_agent: Option<String>,
}

/// Plugin hook input data (structured)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum PluginHookInput {
    /// Filter post published hook
    FilterPostPublished(PostData),
    /// Action post published hook
    ActionPostPublished(PostData),
    /// Filter user created hook
    FilterUserCreated(UserData),
    /// Action user created hook
    ActionUserCreated(UserData),
    /// Action user login hook
    ActionUserLogin(UserData),
    /// Filter authenticate hook
    FilterAuthenticate(AuthData),
}

/// Plugin hook output data (structured)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum PluginHookOutput {
    /// Filter post published result
    FilterPostPublished(PostData),
    /// Filter user created result
    FilterUserCreated(UserData),
    /// Filter authenticate result
    FilterAuthenticate(bool), // true = allow, false = deny
}
