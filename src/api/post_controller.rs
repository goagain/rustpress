use crate::dto::{
    CreatePostRequest, Post, PostResponse, UpdatePostRequest,
    CreateUserRequest, LoginRequest, LoginResponse, RefreshTokenRequest, RefreshTokenResponse,
    UserResponse, UserRole,
};
use crate::repository::{PostRepository, UserRepository};
use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::Json,
};
use std::sync::Arc;
use utoipa::OpenApi;

/// Application state, containing Post and User Repository
pub struct AppState<PR: PostRepository, UR: UserRepository> {
    pub post_repository: Arc<PR>,
    pub user_repository: Arc<UR>,
}

impl<PR: PostRepository, UR: UserRepository> AppState<PR, UR> {
    pub fn new(post_repository: PR, user_repository: UR) -> Self {
        Self {
            post_repository: Arc::new(post_repository),
            user_repository: Arc::new(user_repository),
        }
    }
}

/// Get all posts
///
/// Returns a list of all posts, sorted by creation time in descending order
#[utoipa::path(
    get,
    path = "/api/posts",
    responses(
        (status = 200, description = "Successfully retrieved post list", body = Vec<PostResponse>),
        (status = 500, description = "Internal server error")
    ),
    tag = "Posts"
)]
pub async fn get_posts<PR: PostRepository, UR: UserRepository>(
    State(state): State<Arc<AppState<PR, UR>>>,
) -> Result<Json<Vec<PostResponse>>, StatusCode> {
    match state.post_repository.find_all().await {
        Ok(posts) => Ok(Json(posts.into_iter().map(PostResponse::from).collect())),
        Err(_) => Err(StatusCode::INTERNAL_SERVER_ERROR),
    }
}

/// Get a single post by ID
///
/// Get detailed information of a single post by post ID
#[utoipa::path(
    get,
    path = "/api/posts/{id}",
    params(
        ("id" = String, Path, description = "Post ID")
    ),
    responses(
        (status = 200, description = "Successfully retrieved post", body = PostResponse),
        (status = 404, description = "Post not found"),
        (status = 400, description = "Bad request"),
        (status = 500, description = "Internal server error")
    ),
    tag = "Posts"
)]
pub async fn get_post<PR: PostRepository, UR: UserRepository>(
    Path(id): Path<String>,
    State(state): State<Arc<AppState<PR, UR>>>,
) -> Result<Json<PostResponse>, StatusCode> {
    let id_num: i64 = match id.parse() {
        Ok(num) => num,
        Err(_) => return Err(StatusCode::BAD_REQUEST),
    };
    
        match state.post_repository.find_by_id(&id_num).await {
        Ok(Some(post)) => Ok(Json(PostResponse::from(post))),
        Ok(None) => Err(StatusCode::NOT_FOUND),
        Err(_) => Err(StatusCode::INTERNAL_SERVER_ERROR),
    }
}

/// Create a new post
///
/// Create a new post
#[utoipa::path(
    post,
    path = "/api/posts",
    request_body = CreatePostRequest,
    responses(
        (status = 201, description = "Successfully created post", body = PostResponse),
        (status = 400, description = "Bad request"),
        (status = 500, description = "Internal server error")
    ),
    tag = "Posts"
)]
pub async fn create_post<PR: PostRepository, UR: UserRepository>(
    State(state): State<Arc<AppState<PR, UR>>>,
    axum::Json(payload): axum::Json<CreatePostRequest>,
) -> Result<(axum::http::StatusCode, Json<PostResponse>), StatusCode> {
    match state.post_repository.create(payload).await {
        Ok(post) => Ok((StatusCode::CREATED, Json(PostResponse::from(post)))),
        Err(_) => Err(StatusCode::INTERNAL_SERVER_ERROR),
    }
}

/// Update a post (full update)
///
/// Update post information by ID (PUT method, full update)
#[utoipa::path(
    put,
    path = "/api/posts/{id}",
    params(
        ("id" = String, Path, description = "Post ID")
    ),
    request_body = UpdatePostRequest,
    responses(
        (status = 200, description = "Successfully updated post", body = PostResponse),
        (status = 404, description = "Post not found"),
        (status = 400, description = "Bad request"),
        (status = 500, description = "Internal server error")
    ),
    tag = "Posts"
)]
pub async fn update_post<PR: PostRepository, UR: UserRepository>(
    Path(id): Path<String>,
    State(state): State<Arc<AppState<PR, UR>>>,
    axum::Json(payload): axum::Json<UpdatePostRequest>,
) -> Result<Json<PostResponse>, StatusCode> {
    let id_num: i64 = match id.parse() {
        Ok(num) => num,
        Err(_) => return Err(StatusCode::BAD_REQUEST),
    };
    
    // Get existing post
    let existing_post = match state.post_repository.find_by_id(&id_num).await {
        Ok(Some(post)) => post,
        Ok(None) => return Err(StatusCode::NOT_FOUND),
        Err(_) => return Err(StatusCode::INTERNAL_SERVER_ERROR),
    };

    // Build updated post
    let updated_post = Post {
        id: existing_post.id,
        title: payload.title.unwrap_or(existing_post.title),
        content: payload.content.unwrap_or(existing_post.content),
        category: payload.category.unwrap_or(existing_post.category),
        author_id: existing_post.author_id,
        created_at: existing_post.created_at,
        updated_at: existing_post.updated_at, // Placeholder value, actually updated automatically by ActiveModelBehavior
        archived_at: existing_post.archived_at,
        deleted_at: existing_post.deleted_at,
    };

        match state.post_repository.update(&id_num, updated_post).await {
        Ok(Some(post)) => Ok(Json(PostResponse::from(post))),
        Ok(None) => Err(StatusCode::NOT_FOUND),
        Err(_) => Err(StatusCode::INTERNAL_SERVER_ERROR),
    }
}

/// Delete a post
///
/// Delete a post by ID
#[utoipa::path(
    delete,
    path = "/api/posts/{id}",
    params(
        ("id" = String, Path, description = "Post ID")
    ),
    responses(
        (status = 204, description = "Successfully deleted post"),
        (status = 404, description = "Post not found"),
        (status = 400, description = "Bad request"),
        (status = 500, description = "Internal server error")
    ),
    tag = "Posts"
)]
pub async fn delete_post<PR: PostRepository, UR: UserRepository>(
    Path(id): Path<String>,
    State(state): State<Arc<AppState<PR, UR>>>,
) -> Result<StatusCode, StatusCode> {
    let id_num: i64 = match id.parse() {
        Ok(num) => num,
        Err(_) => return Err(StatusCode::BAD_REQUEST),
    };
    
    match state.post_repository.delete(&id_num).await {
        Ok(true) => Ok(StatusCode::NO_CONTENT),
        Ok(false) => Err(StatusCode::NOT_FOUND),
        Err(_) => Err(StatusCode::INTERNAL_SERVER_ERROR),
    }
}

/// Health check endpoint
///
/// Check if the API service is running normally
#[utoipa::path(
    get,
    path = "/api/health",
    responses(
        (status = 200, description = "Service is running normally", body = String)
    ),
    tag = "Health"
)]
pub async fn health_check() -> &'static str {
    "OK"
}

// Create non-generic wrapper functions for OpenAPI documentation
// utoipa requires concrete function signatures, cannot use generic functions
type PostRepo = crate::repository::PostgresPostRepository;
type UserRepo = crate::repository::PostgresUserRepository;

/// OpenAPI documentation structure
#[derive(OpenApi)]
#[openapi(
    paths(
        get_posts_doc,
        get_post_doc,
        create_post_doc,
        update_post_doc,
        delete_post_doc,
        health_check,
    ),
    components(schemas(
        PostResponse,
        CreatePostRequest,
        UpdatePostRequest,
        UserResponse,
        CreateUserRequest,
        UserRole,
        LoginRequest,
        LoginResponse,
        RefreshTokenRequest,
        RefreshTokenResponse,
    )),
    tags(
        (name = "Posts", description = "Post management API"),
        (name = "Users", description = "User management API"),
        (name = "Auth", description = "Authentication API"),
        (name = "Health", description = "Health check API"),
    ),
    info(
        title = "RustPress API",
        description = "RustPress blog system RESTful API documentation",
        version = "1.0.0",
        contact(
            name = "RustPress",
        ),
    ),
    servers(
        (url = "http://localhost:3000", description = "Local development server"),
    ),
)]
pub struct ApiDoc;

// Wrapper functions for OpenAPI documentation (non-generic)
#[utoipa::path(
    get,
    path = "/api/posts",
    responses(
        (status = 200, description = "Successfully retrieved post list", body = Vec<PostResponse>),
        (status = 500, description = "Internal server error")
    ),
    tag = "Posts"
)]
#[allow(dead_code)]
fn get_posts_doc() {}

#[utoipa::path(
    get,
    path = "/api/posts/{id}",
    params(
        ("id" = String, Path, description = "Post ID")
    ),
    responses(
        (status = 200, description = "Successfully retrieved post", body = PostResponse),
        (status = 404, description = "Post not found"),
        (status = 500, description = "Internal server error")
    ),
    tag = "Posts"
)]
#[allow(dead_code)]
fn get_post_doc(_id: String) {}

#[utoipa::path(
    post,
    path = "/api/posts",
    request_body = CreatePostRequest,
    responses(
        (status = 201, description = "Successfully created post", body = PostResponse),
        (status = 400, description = "Bad request"),
        (status = 500, description = "Internal server error")
    ),
    tag = "Posts"
)]
#[allow(dead_code)]
fn create_post_doc(_payload: CreatePostRequest) {}

#[utoipa::path(
    put,
    path = "/api/posts/{id}",
    params(
        ("id" = String, Path, description = "Post ID")
    ),
    request_body = UpdatePostRequest,
    responses(
        (status = 200, description = "Successfully updated post", body = PostResponse),
        (status = 404, description = "Post not found"),
        (status = 500, description = "Internal server error")
    ),
    tag = "Posts"
)]
#[allow(dead_code)]
fn update_post_doc(_id: String, _payload: UpdatePostRequest) {}

#[utoipa::path(
    delete,
    path = "/api/posts/{id}",
    params(
        ("id" = String, Path, description = "Post ID")
    ),
    responses(
        (status = 204, description = "Successfully deleted post"),
        (status = 404, description = "Post not found"),
        (status = 500, description = "Internal server error")
    ),
    tag = "Posts"
)]
#[allow(dead_code)]
fn delete_post_doc(_id: String) {}

