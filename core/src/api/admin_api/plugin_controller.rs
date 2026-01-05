use crate::dto::{
    AdminPluginEnableResponse, AdminPluginListResponse, AdminPluginUpdateRequest,
    ApprovePluginPermissionsRequest, PluginPermissionsResponse, UpdatePluginPermissionsRequest,
};
use crate::repository::{PostRepository, UserRepository};
use axum::{
    extract::{Extension, Multipart, Path, State},
    http::StatusCode,
    response::Json,
};
use sea_orm::{ActiveModelTrait, ColumnTrait, EntityTrait, QueryFilter, Set};
use std::sync::Arc;

// Plugin management
use crate::entity::plugins;

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
    // Get database connection from state
    let db = state.db.clone();

    let plugins_list = plugins::Entity::find()
        .all(&*db)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let result: Vec<AdminPluginListResponse> = plugins_list
        .into_iter()
        .map(|plugin| AdminPluginListResponse {
            id: plugin.id,
            plugin_id: plugin.plugin_id,
            name: plugin.name,
            description: plugin.description,
            version: plugin.version,
            enabled: plugin.enabled,
            status: plugin.status,
            config: plugin.config,
            manifest: plugin.manifest,
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

/// Uninstall a plugin completely
pub async fn update_plugin<
    PR: PostRepository,
    UR: UserRepository,
    SB: crate::storage::StorageBackend,
>(
    State(state): State<Arc<crate::api::post_controller::ExtendedAppState<PR, UR, SB>>>,
    Extension(_current_user): Extension<Arc<crate::auth::middleware::CurrentUser>>,
    Path(id): Path<i64>,
    axum::Json(payload): axum::Json<AdminPluginUpdateRequest>,
) -> Result<Json<AdminPluginEnableResponse>, StatusCode> {
    // Get database connection from state
    let db = state.db.clone();

    let plugin_model = plugins::Entity::find_by_id(id)
        .one(&*db)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
        .ok_or(StatusCode::NOT_FOUND)?;

    // Check if we're enabling the plugin and analyze new permissions
    let new_permissions = if payload.enabled == Some(true) && !plugin_model.enabled {
        // Plugin is being enabled - check for new permissions
        match state
            .plugin_registry
            .analyze_enable_permissions(&plugin_model.name)
            .await
        {
            Ok(permissions) => permissions,
            Err(e) => {
                tracing::error!(
                    "Failed to analyze permissions for plugin {}: {}",
                    plugin_model.name,
                    e
                );
                return Err(StatusCode::INTERNAL_SERVER_ERROR);
            }
        }
    } else {
        std::collections::HashMap::new()
    };

    let mut plugin: plugins::ActiveModel = plugin_model.into();

    // Update fields based on payload
    if let Some(enabled) = payload.enabled {
        plugin.enabled = Set(enabled);
        // Also update status when enabling/disabling
        plugin.status = Set(if enabled {
            "enabled".to_string()
        } else {
            "disabled".to_string()
        });
    }
    if let Some(config) = &payload.config {
        plugin.config = Set(Some(config.clone()));
    }

    let updated = plugin
        .update(&*db)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    // If plugin was enabled and there are new permissions, return enable response
    // Otherwise return normal response
    let plugin_response = AdminPluginListResponse {
        id: updated.id,
        plugin_id: updated.plugin_id,
        name: updated.name.clone(),
        description: updated.description,
        version: updated.version,
        enabled: updated.enabled,
        status: updated.status,
        config: updated.config,
        manifest: updated.manifest,
        created_at: updated.created_at.into(),
        updated_at: updated.updated_at.into(),
    };

    Ok(Json(AdminPluginEnableResponse {
        plugin: plugin_response,
        new_permissions: new_permissions.keys().cloned().collect(),
        requires_permission_review: !new_permissions.is_empty(),
    }))
}

/// Uninstall a plugin completely
pub async fn uninstall_plugin<
    PR: PostRepository,
    UR: UserRepository,
    SB: crate::storage::StorageBackend,
>(
    State(state): State<Arc<crate::api::post_controller::ExtendedAppState<PR, UR, SB>>>,
    Extension(_current_user): Extension<Arc<crate::auth::middleware::CurrentUser>>,
    Path(id): Path<i64>,
) -> Result<StatusCode, StatusCode> {
    // Get database connection from state
    let db = state.db.clone();

    // First get the plugin to get its name
    let plugin = plugins::Entity::find_by_id(id)
        .one(&*db)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
        .ok_or(StatusCode::NOT_FOUND)?;

    let plugin_name = plugin.name.clone();

    // Uninstall plugin using plugin registry
    state
        .plugin_registry
        .uninstall_plugin(&plugin_name)
        .await
        .map_err(|e| {
            tracing::error!("Failed to uninstall plugin '{}': {}", plugin_name, e);
            StatusCode::INTERNAL_SERVER_ERROR
        })?;

    tracing::info!("Plugin '{}' uninstalled successfully", plugin_name);

    Ok(StatusCode::OK)
}

/// Request body for plugin upload (multipart/form-data)
#[derive(utoipa::ToSchema)]
struct PluginUploadRequest {
    /// RPK plugin file
    plugin: String, // This represents the file field in multipart form
}

/// Install a new plugin
#[utoipa::path(
    post,
    path = "/api/admin/plugins/upload",
    request_body(content_type = "multipart/form-data", content = inline(PluginUploadRequest)),
    responses(
        (status = 200, description = "Successfully installed plugin"),
        (status = 400, description = "Invalid RPK file or missing file field"),
        (status = 409, description = "Plugin already exists"),
        (status = 500, description = "Internal server error")
    ),
    tag = "Admin"
)]
pub async fn install_plugin<
    PR: PostRepository,
    UR: UserRepository,
    SB: crate::storage::StorageBackend,
>(
    State(state): State<Arc<crate::api::post_controller::ExtendedAppState<PR, UR, SB>>>,
    Extension(_current_user): Extension<Arc<crate::auth::middleware::CurrentUser>>,
    mut multipart: Multipart,
) -> Result<StatusCode, StatusCode> {
    // Read RPK file from multipart form
    let field = multipart
        .next_field()
        .await
        .map_err(|e| {
            tracing::error!("Failed to read multipart field: {}", e);
            StatusCode::BAD_REQUEST
        })?
        .ok_or_else(|| {
            tracing::error!("Missing RPK file field in multipart request");
            StatusCode::BAD_REQUEST
        })?;

    // Get the raw bytes from the field (RPK files are binary)
    let rpk_data = field.bytes().await.map_err(|e| {
        tracing::error!("Failed to read RPK file data: {}", e);
        StatusCode::BAD_REQUEST
    })?;

    // Install plugin using plugin registry
    let (_manifest, _update_analysis) = state
        .plugin_registry
        .install_plugin(&rpk_data)
        .await
        .map_err(|e| {
            tracing::error!("Failed to install plugin: {}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        })?;

    Ok(axum::http::StatusCode::CREATED)
}

/// Get plugin permissions
#[utoipa::path(
    get,
    path = "/api/admin/plugins/{id}/permissions",
    params(
        ("id" = String, Path, description = "Plugin ID")
    ),
    responses(
        (status = 200, description = "Successfully retrieved plugin permissions", body = PluginPermissionsResponse),
        (status = 404, description = "Plugin not found"),
        (status = 500, description = "Internal server error")
    ),
    tag = "Admin"
)]
pub async fn get_plugin_permissions<
    PR: PostRepository,
    UR: UserRepository,
    SB: crate::storage::StorageBackend,
>(
    State(state): State<Arc<crate::api::post_controller::ExtendedAppState<PR, UR, SB>>>,
    Extension(_current_user): Extension<Arc<crate::auth::middleware::CurrentUser>>,
    Path(plugin_id): Path<String>,
) -> Result<Json<PluginPermissionsResponse>, StatusCode> {
    // Get database connection
    let db = state.db.clone();

    // Find the plugin in the database
    let plugin = plugins::Entity::find()
        .filter(plugins::Column::Name.eq(&plugin_id))
        .one(&*db)
        .await
        .map_err(|e| {
            tracing::error!("Failed to find plugin '{}': {}", plugin_id, e);
            StatusCode::INTERNAL_SERVER_ERROR
        })?
        .ok_or(StatusCode::NOT_FOUND)?;

    // Parse the manifest to get required permissions
    let manifest: serde_json::Value = serde_json::from_str(&plugin.manifest.unwrap().to_string())
        .map_err(|e| {
        tracing::error!("Failed to parse plugin manifest for '{}': {}", plugin_id, e);
        StatusCode::INTERNAL_SERVER_ERROR
    })?;

    // Get current granted permissions from plugin registry
    let current_permissions = state
        .plugin_registry
        .get_plugin_permissions(&plugin_id)
        .await
        .map_err(|e| {
            tracing::error!(
                "Failed to get current permissions for '{}': {}",
                plugin_id,
                e
            );
            StatusCode::INTERNAL_SERVER_ERROR
        })?;

    // Extract required and optional permissions from manifest
    let mut permissions: Vec<crate::dto::plugin::PluginPermissionInfo> = Vec::new();

    // Add required permissions
    if let Some(required_perms) = manifest
        .get("permissions")
        .and_then(|p| p.get("required"))
        .and_then(|r| r.as_array())
    {
        for perm in required_perms {
            if let Some(perm_str) = perm.as_str() {
                permissions.push(crate::dto::plugin::PluginPermissionInfo {
                    permission: perm_str.to_string(),
                    is_granted: true, // Required permissions are always granted
                    permission_type: "required".to_string(),
                    description: get_permission_description(perm_str),
                });
            }
        }
    }

    // Add optional permissions
    if let Some(optional_perms) = manifest
        .get("permissions")
        .and_then(|p| p.get("optional"))
        .and_then(|o| o.as_array())
    {
        for perm in optional_perms {
            if let Some(perm_str) = perm.as_str() {
                let is_granted = current_permissions.get(perm_str).copied().unwrap_or(false);
                permissions.push(crate::dto::plugin::PluginPermissionInfo {
                    permission: perm_str.to_string(),
                    is_granted,
                    permission_type: "optional".to_string(),
                    description: get_permission_description(perm_str),
                });
            }
        }
    }

    // If no permissions defined in manifest, check hooks for implicit permissions
    if permissions.is_empty()
        && let Some(hooks) = manifest
            .get("hooks")
            .and_then(|h| h.get("registered"))
            .and_then(|r| r.as_array())
    {
        for hook in hooks {
            if let Some(hook_str) = hook.as_str() {
                // Get required permission for this hook
                if let Some(required_perm) =
                    crate::plugin::hook_registry::HookRegistry::get_hook_permission(hook_str)
                {
                    permissions.push(crate::dto::plugin::PluginPermissionInfo {
                        permission: required_perm.clone(),
                        is_granted: true, // Hook-registered permissions are granted
                        permission_type: "required".to_string(),
                        description: get_permission_description(&required_perm),
                    });
                }
            }
        }
    }

    let response = PluginPermissionsResponse {
        plugin_id: plugin_id.clone(),
        permissions,
    };

    Ok(Json(response))
}

/// Get human-readable description for a permission
fn get_permission_description(permission: &str) -> Option<String> {
    match permission {
        "post:read" => Some("Read access to blog posts and their content".to_string()),
        "post:write" => Some("Write/modify access to blog posts".to_string()),
        "post:list_category" => Some("List all categories and their post counts".to_string()),
        "ai:chat" => Some("Access to AI chat completion APIs".to_string()),
        _ => None,
    }
}

/// Update plugin permissions
#[utoipa::path(
    put,
    path = "/api/admin/plugins/{id}/permissions",
    params(
        ("id" = String, Path, description = "Plugin ID")
    ),
    request_body = UpdatePluginPermissionsRequest,
    responses(
        (status = 200, description = "Successfully updated plugin permissions"),
        (status = 404, description = "Plugin not found"),
        (status = 500, description = "Internal server error")
    ),
    tag = "Admin"
)]
pub async fn update_plugin_permissions<
    PR: PostRepository,
    UR: UserRepository,
    SB: crate::storage::StorageBackend,
>(
    State(state): State<Arc<crate::api::post_controller::ExtendedAppState<PR, UR, SB>>>,
    Extension(_current_user): Extension<Arc<crate::auth::middleware::CurrentUser>>,
    Path(plugin_id): Path<String>,
    axum::Json(payload): axum::Json<UpdatePluginPermissionsRequest>,
) -> Result<StatusCode, StatusCode> {
    // First get current permissions to check if plugin exists
    let _current_permissions = state
        .plugin_registry
        .get_plugin_permissions(&plugin_id)
        .await
        .map_err(|_| StatusCode::NOT_FOUND)?;

    // Update permissions
    state
        .plugin_registry
        .update_plugin_permissions(&plugin_id, &payload.permissions)
        .await
        .map_err(|e| {
            tracing::error!("Failed to update plugin permissions: {}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        })?;

    Ok(StatusCode::OK)
}

/// Review and approve plugin permissions (for plugins in pending_review status)
#[utoipa::path(
    post,
    path = "/api/admin/plugins/{id}/review-permissions",
    params(
        ("id" = String, Path, description = "Plugin ID")
    ),
    request_body = ApprovePluginPermissionsRequest,
    responses(
        (status = 200, description = "Successfully approved permissions and enabled plugin"),
        (status = 400, description = "Invalid request or plugin not in pending review"),
        (status = 404, description = "Plugin not found"),
        (status = 500, description = "Internal server error")
    ),
    tag = "Admin"
)]
pub async fn review_plugin_permissions<
    PR: PostRepository,
    UR: UserRepository,
    SB: crate::storage::StorageBackend,
>(
    State(state): State<Arc<crate::api::post_controller::ExtendedAppState<PR, UR, SB>>>,
    Extension(_current_user): Extension<Arc<crate::auth::middleware::CurrentUser>>,
    Path(plugin_id): Path<String>,
    axum::Json(payload): axum::Json<ApprovePluginPermissionsRequest>,
) -> Result<StatusCode, StatusCode> {
    // Get database connection from state
    let db = state.db.clone();

    // Check if plugin exists and is in pending_review status
    let plugin = plugins::Entity::find()
        .filter(plugins::Column::Name.eq(&plugin_id))
        .one(&*db)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
        .ok_or(StatusCode::NOT_FOUND)?;

    if plugin.status != "pending_review" {
        return Err(StatusCode::BAD_REQUEST);
    }

    // Update permissions first
    state
        .plugin_registry
        .update_plugin_permissions(&plugin_id, &payload.approved_permissions)
        .await
        .map_err(|e| {
            tracing::error!("Failed to update plugin permissions: {}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        })?;

    // Update plugin status to enabled
    let mut plugin_model: plugins::ActiveModel = plugin.into();
    plugin_model.status = Set("enabled".to_string());
    plugin_model.enabled = Set(true);
    plugin_model
        .update(&*db)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    tracing::info!("Plugin '{}' permissions approved and enabled", plugin_id);

    Ok(StatusCode::OK)
}
