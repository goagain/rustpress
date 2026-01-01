use crate::dto::{
    AdminPluginListResponse, AdminPluginUpdateRequest, AdminSettingsResponse,
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

/// Get all settings
#[utoipa::path(
    get,
    path = "/api/admin/settings",
    responses(
        (status = 200, description = "Successfully retrieved settings", body = AdminSettingsResponse),
        (status = 500, description = "Internal server error")
    ),
    tag = "Admin"
)]
pub async fn get_settings<
    PR: PostRepository,
    UR: UserRepository,
    SB: crate::storage::StorageBackend,
>(
    State(state): State<Arc<crate::api::post_controller::ExtendedAppState<PR, UR, SB>>>,
    Extension(_current_user): Extension<Arc<crate::auth::middleware::CurrentUser>>,
) -> Result<Json<AdminSettingsResponse>, StatusCode> {
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

    // Default values if not in database
    let allow_external_registration = settings_map
        .get("allow_external_registration")
        .map(|s| s == "true")
        .unwrap_or(true);

    Ok(Json(AdminSettingsResponse {
        allow_external_registration,
        maintenance_mode: settings_map
            .get("maintenance_mode")
            .map(|s| s == "true")
            .unwrap_or(false),
    }))
}

/// Update settings
#[utoipa::path(
    put,
    path = "/api/admin/settings",
    request_body = AdminSettingsUpdateRequest,
    responses(
        (status = 200, description = "Successfully updated settings", body = AdminSettingsResponse),
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
) -> Result<Json<AdminSettingsResponse>, StatusCode> {
    let db = get_db_connection(&state);

    // Update allow_external_registration
    if let Some(value) = payload.allow_external_registration {
        let existing = settings::Entity::find_by_id("allow_external_registration")
            .one(&*db)
            .await
            .map_err(|e| {
                tracing::error!("Failed to query settings: {}", e);
                StatusCode::INTERNAL_SERVER_ERROR
            })?;

        if let Some(existing_model) = existing {
            let mut active_model: settings::ActiveModel = existing_model.into();
            active_model.value = Set(value.to_string());
            active_model.update(&*db).await.map_err(|e| {
                tracing::error!("Failed to update settings: {}", e);
                StatusCode::INTERNAL_SERVER_ERROR
            })?;
        } else {
            let setting = settings::ActiveModel {
                key: Set("allow_external_registration".to_string()),
                value: Set(value.to_string()),
                description: Set(Some("Whether external users can register".to_string())),
                ..Default::default()
            };
            setting.insert(&*db).await.map_err(|e| {
                tracing::error!("Failed to insert settings: {}", e);
                StatusCode::INTERNAL_SERVER_ERROR
            })?;
        }
    }

    // Update maintenance_mode
    if let Some(value) = payload.maintenance_mode {
        let existing = settings::Entity::find_by_id("maintenance_mode")
            .one(&*db)
            .await
            .map_err(|e| {
                tracing::error!("Failed to query settings: {}", e);
                StatusCode::INTERNAL_SERVER_ERROR
            })?;

        if let Some(existing_model) = existing {
            let mut active_model: settings::ActiveModel = existing_model.into();
            active_model.value = Set(value.to_string());
            active_model.update(&*db).await.map_err(|e| {
                tracing::error!("Failed to update settings: {}", e);
                StatusCode::INTERNAL_SERVER_ERROR
            })?;
        } else {
            let setting = settings::ActiveModel {
                key: Set("maintenance_mode".to_string()),
                value: Set(value.to_string()),
                description: Set(Some("Whether the site is in maintenance mode".to_string())),
                ..Default::default()
            };
            setting.insert(&*db).await.map_err(|e| {
                tracing::error!("Failed to insert settings: {}", e);
                StatusCode::INTERNAL_SERVER_ERROR
            })?;
        }
    }

    // Return updated settings
    get_settings(State(state), Extension(current_user)).await
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
