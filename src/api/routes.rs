use crate::api::admin_api;
use crate::api::page_controller::{serve_admin_spa, serve_spa};
use crate::api::post_controller::{ApiDoc, *};
use crate::api::upload_controller::*;
use crate::api::user_controller::*;
use crate::auth::middleware::auth_middleware;
use crate::repository::{PostRepository, UserRepository};
use crate::storage::StorageBackend;
#[allow(unused_imports)] // post, put, delete, patch are used via method chaining
use axum::{
    Router, middleware,
    routing::{delete, get, patch, post, put},
};
use std::sync::Arc;
use tower_http::cors::{AllowOrigin, CorsLayer};
use tower_http::services::ServeDir;
use tower_http::trace::TraceLayer;
use utoipa::OpenApi;
use utoipa_swagger_ui::SwaggerUi;

/// Create and configure all routes
/// RESTful API route design
pub fn create_router<
    PR: PostRepository + 'static,
    UR: UserRepository + 'static,
    SB: StorageBackend + 'static,
>(
    app_state: Arc<AppState<PR, UR>>,
    storage: Arc<SB>,
    db: sea_orm::DatabaseConnection,
    plugin_registry: Arc<crate::plugin::registry::PluginRegistry>,
    plugin_executer: Arc<crate::plugin::registry::PluginExecuter>,
) -> Router {
    // Create extended state that includes storage, database, and plugin system
    let state = Arc::new(ExtendedAppState::new(
        app_state,
        storage,
        db,
        plugin_registry,
        plugin_executer,
    ));

    // Public routes (no authentication required)
    let public_routes = Router::new()
        // Health check
        .route("/api/health", get(health_check))
        // Auth API (not RESTful, but follows common practices)
        // POST /api/auth/login   - User login
        // POST /api/auth/refresh - Refresh access token
        .route("/api/auth/login", post(login::<PR, UR, SB>))
        .route("/api/auth/refresh", post(refresh_token::<PR, UR, SB>))
        // Public posts endpoints
        .route("/api/posts", get(get_posts::<PR, UR, SB>))
        .route("/api/posts/:id", get(get_post::<PR, UR, SB>));

    // Protected routes (authentication required)
    let protected_routes = Router::new()
        // Posts RESTful API (protected)
        // POST   /api/posts      - Create new post
        // PUT    /api/posts/:id  - Full update post
        // DELETE /api/posts/:id  - Delete post
        .route("/api/posts", post(create_post::<PR, UR, SB>))
        .route(
            "/api/posts/:id",
            put(update_post::<PR, UR, SB>).delete(delete_post::<PR, UR, SB>),
        )
        // Post version management
        .route(
            "/api/posts/:id/versions",
            get(get_post_versions::<PR, UR, SB>),
        )
        .route(
            "/api/posts/:post_id/versions/:version_id",
            get(get_post_version::<PR, UR, SB>),
        )
        .route(
            "/api/posts/:post_id/versions/:version_id/restore",
            post(restore_post_from_version::<PR, UR, SB>),
        )
        // Draft management
        .route(
            "/api/drafts",
            post(save_draft::<PR, UR, SB>)
                .get(get_draft::<PR, UR, SB>)
                .delete(delete_draft::<PR, UR, SB>),
        )
        .route("/api/drafts/all", get(get_all_drafts::<PR, UR, SB>))
        // Users RESTful API
        // GET    /api/users      - Get all users
        // POST   /api/users      - Create new user
        // GET    /api/users/:id  - Get single user
        // PUT    /api/users/:id  - Full update user
        // DELETE /api/users/:id  - Delete user
        .route(
            "/api/users",
            get(get_users::<PR, UR, SB>).post(create_user::<PR, UR, SB>),
        )
        .route(
            "/api/users/:id",
            get(get_user::<PR, UR, SB>)
                .put(update_user::<PR, UR, SB>)
                .delete(delete_user::<PR, UR, SB>),
        )
        // Upload API
        // POST /api/upload/image - Upload image
        .route("/api/upload/image", post(upload_image::<PR, UR, SB>))
        .layer(middleware::from_fn(auth_middleware));

    // Admin API router (all routes under /api/admin/*)
    // Create admin router with nested routes and middleware
    let admin_router = admin_api::create_admin_router::<PR, UR, SB>();

    Router::new()
        .merge(public_routes)
        .merge(protected_routes)
        // Nest admin router under /api/admin
        .nest("/api/admin", admin_router)
        // Admin frontend routes (must be before main frontend and fallback)
        .nest_service("/admin/assets", ServeDir::new("admin-frontend/dist/assets"))
        .route("/admin", get(serve_admin_spa))
        .route("/admin/", get(serve_admin_spa))
        .route("/admin/*path", get(serve_admin_spa))
        // Serve uploaded files
        // GET /uploads/* - Serve uploaded files
        .nest_service("/uploads", ServeDir::new("uploads"))
        // Swagger UI
        .merge(SwaggerUi::new("/swagger-ui").url("/api-doc/openapi.json", ApiDoc::openapi()))
        // Serve main frontend static files
        .nest_service("/assets", ServeDir::new("frontend/dist/assets"))
        .nest_service("/", ServeDir::new("frontend/dist"))
        // SPA fallback - serve index.html for all non-API routes
        // This must be last so API routes take precedence
        .fallback(serve_spa)
        .layer(
            CorsLayer::new()
                .allow_origin(AllowOrigin::any())
                .allow_methods([
                    axum::http::Method::GET,
                    axum::http::Method::POST,
                    axum::http::Method::PUT,
                    axum::http::Method::PATCH,
                    axum::http::Method::DELETE,
                ])
                .allow_headers([
                    axum::http::header::CONTENT_TYPE,
                    axum::http::header::AUTHORIZATION,
                ]),
        )
        .layer(TraceLayer::new_for_http())
        .with_state(state)
}
