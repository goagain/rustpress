pub mod admin_controller;
pub mod metrics_controller;
pub mod openai_controller;
pub mod plugin_controller;
pub mod post_controller;
pub mod settings_helper;
pub mod user_controller;

pub use admin_controller::*;
pub use metrics_controller::*;
pub use openai_controller::*;
pub use plugin_controller::*;
pub use post_controller::*;
pub use user_controller::*;

use crate::api::post_controller::ExtendedAppState;
use crate::repository::{PostRepository, UserRepository};
use crate::storage::StorageBackend;
use axum::{
    Router, middleware,
    routing::{delete, get, post, put},
};
use std::sync::Arc;

/// Create admin API router
/// All routes under /api/admin/* require authentication and admin role
/// Note: The router returned does not have state attached, as it will inherit from parent router when nested
pub fn create_admin_router<
    PR: PostRepository + 'static,
    UR: UserRepository + 'static,
    SB: StorageBackend + 'static,
>() -> Router<Arc<ExtendedAppState<PR, UR, SB>>> {
    use crate::auth::middleware::{admin_middleware, auth_middleware};

    Router::new()
        // Settings management
        // GET /api/admin/settings/tabs - Get all settings tabs
        // PUT /api/admin/settings - Update settings
        .route("/settings/tabs", get(get_settings_tabs::<PR, UR, SB>))
        .route("/settings", put(update_settings::<PR, UR, SB>))
        // User management
        // GET /api/admin/users - Get all users (with ban status)
        // POST /api/admin/users/:id/ban - Ban or unban a user
        // POST /api/admin/users/:id/reset-password - Reset user password
        .route("/users", get(get_all_users::<PR, UR, SB>))
        .route("/users/:id/ban", post(ban_user::<PR, UR, SB>))
        .route(
            "/users/:id/reset-password",
            post(reset_user_password::<PR, UR, SB>),
        )
        // Post management
        // GET /api/admin/posts - Get all posts (admin view)
        // DELETE /api/admin/posts/:id - Delete any post
        .route("/posts", get(get_all_posts::<PR, UR, SB>))
        .route("/posts/:id", delete(admin_delete_post::<PR, UR, SB>))
        // Plugin management
        // GET /api/admin/plugins - Get all plugins
        // POST /api/admin/plugins - Install new plugin
        // POST /api/admin/plugins/upload - Upload RPK file
        // PUT /api/admin/plugins/:id - Update plugin status
        // DELETE /api/admin/plugins/:id - Uninstall plugin completely
        // GET /api/admin/plugins/:id/permissions - Get plugin permissions
        // PUT /api/admin/plugins/:id/permissions - Update plugin permissions
        // POST /api/admin/plugins/:id/review-permissions - Approve pending permissions
        .route(
            "/plugins",
            get(get_all_plugins::<PR, UR, SB>).post(install_plugin::<PR, UR, SB>),
        )
        .route("/plugins/upload", post(install_plugin::<PR, UR, SB>))
        .route(
            "/plugins/:id",
            put(update_plugin::<PR, UR, SB>).delete(uninstall_plugin::<PR, UR, SB>),
        )
        .route(
            "/plugins/:id/permissions",
            get(get_plugin_permissions::<PR, UR, SB>).put(update_plugin_permissions::<PR, UR, SB>),
        )
        .route(
            "/plugins/:id/review-permissions",
            post(review_plugin_permissions::<PR, UR, SB>),
        )
        // OpenAI API key management
        // GET /api/admin/openai/keys - Get all API keys
        // POST /api/admin/openai/keys - Create new API key
        // PUT /api/admin/openai/keys/:id - Update API key
        // DELETE /api/admin/openai/keys/:id - Delete API key
        // POST /api/admin/openai/keys/:id/test - Test API key
        // GET /api/admin/openai/keys/:id/models - List available models
        // POST /api/admin/openai/keys/:id/models - Set default model
        .route(
            "/openai/keys",
            get(get_openai_keys::<PR, UR, SB>).post(create_openai_key::<PR, UR, SB>),
        )
        .route(
            "/openai/keys/:id",
            put(update_openai_key::<PR, UR, SB>).delete(delete_openai_key::<PR, UR, SB>),
        )
        .route("/openai/keys/:id/test", post(test_openai_key::<PR, UR, SB>))
        .route(
            "/openai/keys/:id/models",
            get(list_openai_models::<PR, UR, SB>).post(set_default_model::<PR, UR, SB>),
        )
        // Apply both auth_middleware and admin_middleware to all admin routes
        // Order matters: auth_middleware must run first, then admin_middleware
        .layer(middleware::from_fn(admin_middleware))
        .layer(middleware::from_fn(auth_middleware))
}
