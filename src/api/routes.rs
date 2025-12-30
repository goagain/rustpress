use crate::api::post_controller::{ApiDoc, *};
use crate::api::user_controller::*;
use crate::repository::{PostRepository, UserRepository};
#[allow(unused_imports)] // post, put, delete, patch are used via method chaining
use axum::{
    routing::{get, post, put, delete, patch},
    Router,
};
use std::sync::Arc;
use tower_http::cors::{AllowOrigin, CorsLayer};
use tower_http::trace::TraceLayer;
use utoipa::OpenApi;
use utoipa_swagger_ui::SwaggerUi;

/// Create and configure all routes
/// RESTful API route design
pub fn create_router<PR: PostRepository + 'static, UR: UserRepository + 'static>(
    app_state: Arc<AppState<PR, UR>>,
) -> Router {
    Router::new()
        // Health check
        .route("/api/health", get(health_check))
        
        // Posts RESTful API
        // GET    /api/posts      - Get all posts
        // POST   /api/posts      - Create new post
        // GET    /api/posts/:id  - Get single post
        // PUT    /api/posts/:id  - Full update post
        // DELETE /api/posts/:id  - Delete post
        .route("/api/posts", get(get_posts::<PR, UR>).post(create_post::<PR, UR>))
        .route(
            "/api/posts/:id",
            get(get_post::<PR, UR>)
                .put(update_post::<PR, UR>)
                .delete(delete_post::<PR, UR>),
        )
        
        // Users RESTful API
        // GET    /api/users      - Get all users
        // POST   /api/users      - Create new user
        // GET    /api/users/:id  - Get single user
        // PUT    /api/users/:id  - Full update user
        // DELETE /api/users/:id  - Delete user
        .route("/api/users", get(get_users::<PR, UR>).post(create_user::<PR, UR>))
        .route(
            "/api/users/:id",
            get(get_user::<PR, UR>)
                .put(update_user::<PR, UR>)
                .delete(delete_user::<PR, UR>),
        )
        
        // Auth API (not RESTful, but follows common practices)
        // POST /api/auth/login   - User login
        // POST /api/auth/refresh - Refresh access token
        .route("/api/auth/login", post(login::<PR, UR>))
        .route("/api/auth/refresh", post(refresh_token::<PR, UR>))
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
        .with_state(app_state)
}

