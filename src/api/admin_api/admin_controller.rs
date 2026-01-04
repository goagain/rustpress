use crate::dto::admin::{
    AdminSettingsTabsResponse, AdminSettingsUpdateRequest, SettingItem, SettingsTab,
};
use crate::repository::{PostRepository, UserRepository};
use axum::{
    extract::{Extension, State},
    http::StatusCode,
    response::Json,
};
use sea_orm::{ActiveModelTrait, ColumnTrait, EntityTrait, QueryFilter, Set};
use serde_json;
use std::collections::HashMap;
use std::sync::Arc;

// Settings management
use crate::entity::settings;

/// Get all settings tabs
#[utoipa::path(
    get,
    path = "/api/admin/settings/tabs",
    responses(
        (status = 200, description = "Successfully retrieved settings tabs", body = AdminSettingsTabsResponse),
        (status = 500, description = "Internal server error")
    ),
    tag = "Admin"
)]
pub async fn get_settings_tabs<
    PR: PostRepository,
    UR: UserRepository,
    SB: crate::storage::StorageBackend,
>(
    State(state): State<Arc<crate::api::post_controller::ExtendedAppState<PR, UR, SB>>>,
    Extension(_current_user): Extension<Arc<crate::auth::middleware::CurrentUser>>,
) -> Result<Json<AdminSettingsTabsResponse>, StatusCode> {
    // Get database connection from state
    let db = get_db_connection(&state);

    let settings_models = settings::Entity::find().all(&*db).await.map_err(|e| {
        tracing::error!("Failed to fetch settings: {}", e);
        StatusCode::INTERNAL_SERVER_ERROR
    })?;

    let mut settings_map = HashMap::new();
    for setting in settings_models {
        settings_map.insert(setting.key.clone(), setting.value);
    }

    // Define settings tabs structure
    let tabs = vec![
        SettingsTab {
            id: "general".to_string(),
            label: "General".to_string(),
            description: Some("General system settings".to_string()),
            items: vec![
                SettingItem {
                    key: "site_title".to_string(),
                    label: "Site Title".to_string(),
                    description: Some("The title of your site".to_string()),
                    value: serde_json::Value::String(
                        settings_map
                            .get("site_title")
                            .unwrap_or(&"RustPress".to_string())
                            .clone(),
                    ),
                    input_type: "text".to_string(),
                },
                SettingItem {
                    key: "site_description".to_string(),
                    label: "Site Description".to_string(),
                    description: Some("Brief description of your site".to_string()),
                    value: serde_json::Value::String(
                        settings_map
                            .get("site_description")
                            .unwrap_or(&"A modern CMS built with Rust".to_string())
                            .clone(),
                    ),
                    input_type: "textarea".to_string(),
                },
                SettingItem {
                    key: "posts_per_page".to_string(),
                    label: "Posts Per Page".to_string(),
                    description: Some("Number of posts to display per page".to_string()),
                    value: serde_json::Value::String(
                        settings_map
                            .get("posts_per_page")
                            .unwrap_or(&"10".to_string())
                            .clone(),
                    ),
                    input_type: "number".to_string(),
                },
                SettingItem {
                    key: "external_registration".to_string(),
                    label: "Allow External Registration".to_string(),
                    description: Some("Allow users to register accounts".to_string()),
                    value: serde_json::Value::Bool(
                        settings_map
                            .get("external_registration")
                            .map(|s| s == "true")
                            .unwrap_or(false),
                    ),
                    input_type: "checkbox".to_string(),
                },
                SettingItem {
                    key: "maintenance_mode".to_string(),
                    label: "Maintenance Mode".to_string(),
                    description: Some("Put the site in maintenance mode".to_string()),
                    value: serde_json::Value::Bool(
                        settings_map
                            .get("maintenance_mode")
                            .map(|s| s == "true")
                            .unwrap_or(false),
                    ),
                    input_type: "checkbox".to_string(),
                },
            ],
        },
        SettingsTab {
            id: "content".to_string(),
            label: "Content".to_string(),
            description: Some("Content and publishing settings".to_string()),
            items: vec![
                SettingItem {
                    key: "default_category".to_string(),
                    label: "Default Category".to_string(),
                    description: Some("Default category for new posts".to_string()),
                    value: serde_json::Value::String(
                        settings_map
                            .get("default_category")
                            .unwrap_or(&"uncategorized".to_string())
                            .clone(),
                    ),
                    input_type: "text".to_string(),
                },
                SettingItem {
                    key: "auto_publish_comments".to_string(),
                    label: "Auto-publish Comments".to_string(),
                    description: Some(
                        "Automatically publish comments without moderation".to_string(),
                    ),
                    value: serde_json::Value::Bool(
                        settings_map
                            .get("auto_publish_comments")
                            .map(|s| s == "true")
                            .unwrap_or(false),
                    ),
                    input_type: "checkbox".to_string(),
                },
            ],
        },
        SettingsTab {
            id: "plugins".to_string(),
            label: "Plugins".to_string(),
            description: Some("Plugin system settings".to_string()),
            items: vec![
                SettingItem {
                    key: "plugin_cache_dir".to_string(),
                    label: "Plugin Cache Directory".to_string(),
                    description: Some("Directory where plugin files are cached".to_string()),
                    value: serde_json::Value::String(
                        settings_map
                            .get("plugin_cache_dir")
                            .unwrap_or(&"./plugins/cache".to_string())
                            .clone(),
                    ),
                    input_type: "text".to_string(),
                },
                SettingItem {
                    key: "plugin_install_dir".to_string(),
                    label: "Plugin Install Directory".to_string(),
                    description: Some("Directory where plugin packages are stored".to_string()),
                    value: serde_json::Value::String(
                        settings_map
                            .get("plugin_install_dir")
                            .unwrap_or(&"./plugins/install".to_string())
                            .clone(),
                    ),
                    input_type: "text".to_string(),
                },
            ],
        },
    ];

    Ok(Json(AdminSettingsTabsResponse { tabs }))
}

/// Update settings
#[utoipa::path(
    put,
    path = "/api/admin/settings",
    request_body = AdminSettingsUpdateRequest,
    responses(
        (status = 200, description = "Successfully updated settings", body = AdminSettingsTabsResponse),
        (status = 500, description = "Internal server error")
    ),
    tag = "Admin"
)]
pub async fn update_settings<
    PR: PostRepository,
    UR: UserRepository,
    SB: crate::storage::StorageBackend,
>(
    State(state): State<Arc<crate::api::post_controller::ExtendedAppState<PR, UR, SB>>>,
    Extension(current_user): Extension<Arc<crate::auth::middleware::CurrentUser>>,
    axum::Json(updates): axum::Json<AdminSettingsUpdateRequest>,
) -> Result<Json<AdminSettingsTabsResponse>, StatusCode> {
    // Get database connection from state
    let db = get_db_connection(&state);

    // Update each setting
    for (key, value) in &updates.settings {
        // Try to find existing setting
        let existing_setting = settings::Entity::find()
            .filter(settings::Column::Key.eq(key))
            .one(&*db)
            .await
            .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

        if let Some(setting) = existing_setting {
            // Update existing setting
            let mut setting: settings::ActiveModel = setting.into();
            setting.value = Set(value.to_string());
            setting.update(&*db).await.map_err(|e| {
                tracing::error!("Failed to update setting {}: {}", key, e);
                StatusCode::INTERNAL_SERVER_ERROR
            })?;
        } else {
            // Insert new setting
            let new_setting = settings::ActiveModel {
                key: Set(key.clone()),
                value: Set(value.to_string()),
                ..Default::default()
            };
            new_setting.insert(&*db).await.map_err(|e| {
                tracing::error!("Failed to insert setting {}: {}", key, e);
                StatusCode::INTERNAL_SERVER_ERROR
            })?;
        }
    }

    // Return updated settings tabs
    get_settings_tabs(State(state), Extension(current_user)).await
}

// Helper function to get database connection from state
fn get_db_connection<PR: PostRepository, UR: UserRepository, SB: crate::storage::StorageBackend>(
    state: &Arc<crate::api::post_controller::ExtendedAppState<PR, UR, SB>>,
) -> Arc<sea_orm::DatabaseConnection> {
    state.db.clone()
}
