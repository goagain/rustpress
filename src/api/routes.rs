use crate::api::post_controller::{ApiDoc, *};
use crate::api::upload_controller::*;
use crate::api::user_controller::*;
use crate::repository::{PostRepository, UserRepository};
use crate::storage::StorageBackend;
#[allow(unused_imports)] // post, put, delete, patch are used via method chaining
use axum::{
    routing::{get, post, put, delete, patch},
    Router,
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
) -> Router {
    // Create extended state that includes storage
    let state = Arc::new(ExtendedAppState::new(app_state, storage));
    Router::new()
        // Health check
        .route("/api/health", get(health_check))
        
        // Posts RESTful API
        // GET    /api/posts      - Get all posts
        // POST   /api/posts      - Create new post
        // GET    /api/posts/:id  - Get single post
        // PUT    /api/posts/:id  - Full update post
        // DELETE /api/posts/:id  - Delete post
        .route("/api/posts", get(get_posts::<PR, UR, SB>).post(create_post::<PR, UR, SB>))
        .route(
            "/api/posts/:id",
            get(get_post::<PR, UR, SB>)
                .put(update_post::<PR, UR, SB>)
                .delete(delete_post::<PR, UR, SB>),
        )
        
        // Users RESTful API
        // GET    /api/users      - Get all users
        // POST   /api/users      - Create new user
        // GET    /api/users/:id  - Get single user
        // PUT    /api/users/:id  - Full update user
        // DELETE /api/users/:id  - Delete user
        .route("/api/users", get(get_users::<PR, UR, SB>).post(create_user::<PR, UR, SB>))
        .route(
            "/api/users/:id",
            get(get_user::<PR, UR, SB>)
                .put(update_user::<PR, UR, SB>)
                .delete(delete_user::<PR, UR, SB>),
        )
        
        // Auth API (not RESTful, but follows common practices)
        // POST /api/auth/login   - User login
        // POST /api/auth/refresh - Refresh access token
        .route("/api/auth/login", post(login::<PR, UR, SB>))
        .route("/api/auth/refresh", post(refresh_token::<PR, UR, SB>))
        
        // Upload API
        // POST /api/upload/image - Upload image
        .route("/api/upload/image", post(upload_image::<PR, UR, SB>))
        
        // Serve uploaded files
        // GET /uploads/* - Serve uploaded files
        .nest_service("/uploads", ServeDir::new("uploads"))
        
        // Swagger UI
        .merge(
            SwaggerUi::new("/swagger-ui")
                .url("/api-doc/openapi.json", ApiDoc::openapi()),
        )
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
                .allow_headers([axum::http::header::CONTENT_TYPE]),
        )
        .layer(TraceLayer::new_for_http())
        .with_state(state)
}

