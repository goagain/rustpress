use crate::dto::admin::{SettingItem, SettingsTab};
use crate::dto::{
    AdminPluginListResponse, AdminPluginUpdateRequest, AdminSettingsTabsResponse,
    AdminSettingsUpdateRequest,
};
use crate::repository::{PostRepository, UserRepository};
use axum::{
    extract::{Extension, Path, State},
    http::StatusCode,
    response::Json,
};
use sea_orm::{ActiveModelTrait, EntityTrait, Set};
use std::collections::HashMap;
use std::sync::Arc;

// Settings and plugins management
use crate::entity::{plugins, settings};

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

    // Build settings tabs
    let mut tabs = Vec::new();

    // General Settings Tab
    let general_items = vec![
        SettingItem {
            key: "allow_external_registration".to_string(),
            value: serde_json::Value::Bool(
                settings_map
                    .get("allow_external_registration")
                    .map(|s| s == "true")
                    .unwrap_or(true),
            ),
            label: "Allow external user registration".to_string(),
            description: Some("Allow users to register new accounts".to_string()),
            input_type: "checkbox".to_string(),
        },
        SettingItem {
            key: "maintenance_mode".to_string(),
            value: serde_json::Value::Bool(
                settings_map
                    .get("maintenance_mode")
                    .map(|s| s == "true")
                    .unwrap_or(false),
            ),
            label: "Maintenance mode".to_string(),
            description: Some("Enable maintenance mode to restrict site access".to_string()),
            input_type: "checkbox".to_string(),
        },
    ];

    tabs.push(SettingsTab {
        id: "general".to_string(),
        label: "General".to_string(),
        description: Some("General system settings".to_string()),
        items: general_items,
    });

    // OpenAI Settings Tab (empty items, managed by separate component)
    tabs.push(SettingsTab {
        id: "openai".to_string(),
        label: "OpenAI".to_string(),
        description: Some("OpenAI API key and model management".to_string()),
        items: vec![], // OpenAI keys are managed via separate API endpoints
    });

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
    axum::Json(payload): axum::Json<AdminSettingsUpdateRequest>,
) -> Result<Json<AdminSettingsTabsResponse>, StatusCode> {
    let db = get_db_connection(&state);

    // Update each setting in the payload
    for (key, value) in payload.settings {
        let value_str = match value {
            serde_json::Value::Bool(b) => b.to_string(),
            serde_json::Value::Number(n) => n.to_string(),
            serde_json::Value::String(s) => s,
            serde_json::Value::Null => "".to_string(),
            _ => value.to_string(),
        };

        let existing = settings::Entity::find_by_id(&key)
            .one(&*db)
            .await
            .map_err(|e| {
                tracing::error!("Failed to query settings: {}", e);
                StatusCode::INTERNAL_SERVER_ERROR
            })?;

        if let Some(existing_model) = existing {
            let mut active_model: settings::ActiveModel = existing_model.into();
            active_model.value = Set(value_str);
            active_model.update(&*db).await.map_err(|e| {
                tracing::error!("Failed to update settings: {}", e);
                StatusCode::INTERNAL_SERVER_ERROR
            })?;
        } else {
            let setting = settings::ActiveModel {
                key: Set(key.clone()),
                value: Set(value_str),
                description: Set(None),
                ..Default::default()
            };
            setting.insert(&*db).await.map_err(|e| {
                tracing::error!("Failed to insert settings: {}", e);
                StatusCode::INTERNAL_SERVER_ERROR
            })?;
        }
    }

    // Return updated settings tabs
    get_settings_tabs(State(state), Extension(current_user)).await
}

/// Get all plugins
#[utoipa::path(
    get,
    path = "/api/admin/plugins",
    responses(
        (status = 200, description = "Successfully retrieved plugin list", body = Vec<AdminPluginListResponse>),
        (status = 500, description = "Internal server error")
    ),
    tag = "Admin"
)]
pub async fn get_all_plugins<
    PR: PostRepository,
    UR: UserRepository,
    SB: crate::storage::StorageBackend,
>(
    State(state): State<Arc<crate::api::post_controller::ExtendedAppState<PR, UR, SB>>>,
    Extension(_current_user): Extension<Arc<crate::auth::middleware::CurrentUser>>,
) -> Result<Json<Vec<AdminPluginListResponse>>, StatusCode> {
    let db = get_db_connection(&state);

    let plugins_list = plugins::Entity::find()
        .all(&*db)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let result: Vec<AdminPluginListResponse> = plugins_list
        .into_iter()
        .map(|plugin| AdminPluginListResponse {
            id: plugin.id,
            name: plugin.name,
            description: plugin.description,
            version: plugin.version,
            enabled: plugin.enabled,
            config: plugin.config,
            created_at: plugin.created_at.into(),
            updated_at: plugin.updated_at.into(),
        })
        .collect();

    Ok(Json(result))
}

/// Update plugin status
#[utoipa::path(
    put,
    path = "/api/admin/plugins/{id}",
    params(
        ("id" = i64, Path, description = "Plugin ID")
    ),
    request_body = AdminPluginUpdateRequest,
    responses(
        (status = 200, description = "Successfully updated plugin", body = AdminPluginListResponse),
        (status = 404, description = "Plugin not found"),
        (status = 500, description = "Internal server error")
    ),
    tag = "Admin"
)]
pub async fn update_plugin<
    PR: PostRepository,
    UR: UserRepository,
    SB: crate::storage::StorageBackend,
>(
    Path(id): Path<i64>,
    State(state): State<Arc<crate::api::post_controller::ExtendedAppState<PR, UR, SB>>>,
    Extension(_current_user): Extension<Arc<crate::auth::middleware::CurrentUser>>,
    axum::Json(payload): axum::Json<AdminPluginUpdateRequest>,
) -> Result<Json<AdminPluginListResponse>, StatusCode> {
    let db = get_db_connection(&state);

    let mut plugin: plugins::ActiveModel = plugins::Entity::find_by_id(id)
        .one(&*db)
        .await
        .map_err(|e| {
            tracing::error!("Failed to query plugin: {}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        })?
        .ok_or(StatusCode::NOT_FOUND)?
        .into();

    if let Some(enabled) = payload.enabled {
        plugin.enabled = Set(enabled);
    }

    if payload.config.is_some() {
        plugin.config = Set(payload.config.clone());
    }

    let updated = plugin.update(&*db).await.map_err(|e| {
        tracing::error!("Failed to update plugin: {}", e);
        StatusCode::INTERNAL_SERVER_ERROR
    })?;

    Ok(Json(AdminPluginListResponse {
        id: updated.id,
        name: updated.name,
        description: updated.description,
        version: updated.version,
        enabled: updated.enabled,
        config: updated.config,
        created_at: updated.created_at.into(),
        updated_at: updated.updated_at.into(),
    }))
}

// Helper function to get database connection from state
fn get_db_connection<PR: PostRepository, UR: UserRepository, SB: crate::storage::StorageBackend>(
    state: &Arc<crate::api::post_controller::ExtendedAppState<PR, UR, SB>>,
) -> Arc<sea_orm::DatabaseConnection> {
    state.db.clone()
}
