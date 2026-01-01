pub mod admin_controller;
pub mod post_controller;
pub mod user_controller;

pub use admin_controller::*;
pub use post_controller::*;
pub use user_controller::*;

use crate::api::post_controller::ExtendedAppState;
use crate::repository::{PostRepository, UserRepository};
use crate::storage::StorageBackend;
use axum::{
    middleware,
    routing::{delete, get, post, put},
    Router,
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
        // GET /api/admin/settings - Get all settings
        // PUT /api/admin/settings - Update settings
        .route(
            "/settings",
            get(get_settings::<PR, UR, SB>).put(update_settings::<PR, UR, SB>),
        )
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
        // PUT /api/admin/plugins/:id - Update plugin status
        .route("/plugins", get(get_all_plugins::<PR, UR, SB>))
        .route("/plugins/:id", put(update_plugin::<PR, UR, SB>))
        // Apply both auth_middleware and admin_middleware to all admin routes
        // Order matters: auth_middleware must run first, then admin_middleware
        .layer(middleware::from_fn(admin_middleware))
        .layer(middleware::from_fn(auth_middleware))
}
