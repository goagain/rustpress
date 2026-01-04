use crate::dto::plugin::PluginHook;
use crate::dto::{
    CreatePostRequest, CreateUserRequest, LoginRequest, LoginResponse, Post, PostDraftResponse,
    PostResponse, PostVersionResponse, RefreshTokenRequest, RefreshTokenResponse, SaveDraftRequest,
    UpdatePostRequest, UserResponse, UserRole,
};
use crate::repository::{PostRepository, UserRepository};
use axum::{
    extract::{Extension, Path, State},
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

/// Extended application state that includes storage backend, database connection, and plugin manager
pub struct ExtendedAppState<
    PR: PostRepository,
    UR: UserRepository,
    SB: crate::storage::StorageBackend,
> {
    pub app_state: Arc<AppState<PR, UR>>,
    pub storage: Arc<SB>,
    pub db: Arc<sea_orm::DatabaseConnection>,
    pub plugin_manager: Arc<crate::plugin::PluginManager>,
}

impl<PR: PostRepository, UR: UserRepository, SB: crate::storage::StorageBackend>
    ExtendedAppState<PR, UR, SB>
{
    pub fn new(
        app_state: Arc<AppState<PR, UR>>,
        storage: Arc<SB>,
        db: sea_orm::DatabaseConnection,
        plugin_manager: Arc<crate::plugin::PluginManager>,
    ) -> Self {
        Self {
            app_state,
            storage,
            db: Arc::new(db),
            plugin_manager,
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
pub async fn get_posts<
    PR: PostRepository,
    UR: UserRepository,
    SB: crate::storage::StorageBackend,
>(
    State(state): State<Arc<ExtendedAppState<PR, UR, SB>>>,
) -> Result<Json<Vec<PostResponse>>, StatusCode> {
    match state.app_state.post_repository.find_all().await {
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
pub async fn get_post<
    PR: PostRepository,
    UR: UserRepository,
    SB: crate::storage::StorageBackend,
>(
    Path(id): Path<String>,
    State(state): State<Arc<ExtendedAppState<PR, UR, SB>>>,
) -> Result<Json<PostResponse>, StatusCode> {
    let id_num: i64 = match id.parse() {
        Ok(num) => num,
        Err(_) => return Err(StatusCode::BAD_REQUEST),
    };

    match state.app_state.post_repository.find_by_id(&id_num).await {
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
pub async fn create_post<
    PR: PostRepository,
    UR: UserRepository,
    SB: crate::storage::StorageBackend,
>(
    State(state): State<Arc<ExtendedAppState<PR, UR, SB>>>,
    Extension(current_user): Extension<Arc<crate::auth::middleware::CurrentUser>>,
    axum::Json(payload): axum::Json<CreatePostRequest>,
) -> Result<(axum::http::StatusCode, Json<PostResponse>), StatusCode> {
    // User must be authenticated (already checked by middleware)

    // Create initial post data for filtering
    let category = payload.category.as_deref().unwrap_or("uncategorized");
    let initial_post_data = serde_json::json!({
        "id": 0, // Will be set after creation
        "title": payload.title,
        "content": payload.content,
        "category": category,
        "author_id": payload.author_id,
        "created_at": chrono::Utc::now(),
        "updated_at": chrono::Utc::now()
    });

    // Apply filter hooks to modify post data before creation
    let plugin_manager = state.plugin_manager.clone();
    let (filtered_post_data, _filter_results) = plugin_manager
        .execute_filter_hook(PluginHook::FilterPostPublished, initial_post_data)
        .await;

    // Extract modified data for post creation
    let filtered_title = filtered_post_data["title"]
        .as_str()
        .unwrap_or(&payload.title)
        .to_string();
    let filtered_content = filtered_post_data["content"]
        .as_str()
        .unwrap_or(&payload.content)
        .to_string();
    let filtered_category = filtered_post_data["category"]
        .as_str()
        .unwrap_or_else(|| payload.category.as_deref().unwrap_or("uncategorized"))
        .to_string();

    // Create post with filtered data
    let filtered_payload = CreatePostRequest {
        title: filtered_title,
        content: filtered_content,
        category: Some(filtered_category),
        author_id: payload.author_id,
    };

    match state
        .app_state
        .post_repository
        .create(filtered_payload)
        .await
    {
        Ok(post) => {
            let post_response = PostResponse::from(post.clone());

            // Execute post published action hooks asynchronously (non-blocking)
            let final_post_data = serde_json::json!({
                "id": post.id,
                "title": post.title,
                "content": post.content,
                "category": post.category,
                "author_id": post.author_id,
                "created_at": post.created_at,
                "updated_at": post.updated_at
            });

            let plugin_manager_clone = state.plugin_manager.clone();
            tokio::spawn(async move {
                plugin_manager_clone
                    .execute_action_hook(PluginHook::ActionPostPublished, final_post_data)
                    .await;
            });

            Ok((StatusCode::CREATED, Json(post_response)))
        }
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
pub async fn update_post<
    PR: PostRepository,
    UR: UserRepository,
    SB: crate::storage::StorageBackend,
>(
    Path(id): Path<String>,
    State(state): State<Arc<ExtendedAppState<PR, UR, SB>>>,
    Extension(current_user): Extension<Arc<crate::auth::middleware::CurrentUser>>,
    axum::Json(payload): axum::Json<UpdatePostRequest>,
) -> Result<Json<PostResponse>, StatusCode> {
    let id_num: i64 = match id.parse() {
        Ok(num) => num,
        Err(_) => return Err(StatusCode::BAD_REQUEST),
    };

    // Get existing post
    let existing_post = match state.app_state.post_repository.find_by_id(&id_num).await {
        Ok(Some(post)) => post,
        Ok(None) => return Err(StatusCode::NOT_FOUND),
        Err(_) => return Err(StatusCode::INTERNAL_SERVER_ERROR),
    };

    // Check if user is author or admin
    if !current_user.is_author_or_admin(existing_post.author_id) {
        return Err(StatusCode::FORBIDDEN);
    }

    // Use current user ID from context
    let user_id = current_user.id;

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

    match state
        .app_state
        .post_repository
        .update(
            &id_num,
            updated_post,
            payload.create_version,
            payload.change_note,
            user_id, // Use extracted user_id from JWT token
        )
        .await
    {
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
pub async fn delete_post<
    PR: PostRepository,
    UR: UserRepository,
    SB: crate::storage::StorageBackend,
>(
    Path(id): Path<String>,
    State(state): State<Arc<ExtendedAppState<PR, UR, SB>>>,
    Extension(current_user): Extension<Arc<crate::auth::middleware::CurrentUser>>,
) -> Result<StatusCode, StatusCode> {
    let id_num: i64 = match id.parse() {
        Ok(num) => num,
        Err(_) => return Err(StatusCode::BAD_REQUEST),
    };

    // Get existing post to check ownership
    let existing_post = match state.app_state.post_repository.find_by_id(&id_num).await {
        Ok(Some(post)) => post,
        Ok(None) => return Err(StatusCode::NOT_FOUND),
        Err(_) => return Err(StatusCode::INTERNAL_SERVER_ERROR),
    };

    // Check if user is author or admin
    if !current_user.is_author_or_admin(existing_post.author_id) {
        return Err(StatusCode::FORBIDDEN);
    }

    match state.app_state.post_repository.delete(&id_num).await {
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

// Version management endpoints

/// Get all versions of a post
#[utoipa::path(
    get,
    path = "/api/posts/{id}/versions",
    params(
        ("id" = String, Path, description = "Post ID")
    ),
    responses(
        (status = 200, description = "Successfully retrieved versions", body = Vec<PostVersionResponse>),
        (status = 400, description = "Bad request"),
        (status = 500, description = "Internal server error")
    ),
    tag = "Posts"
)]
pub async fn get_post_versions<
    PR: PostRepository,
    UR: UserRepository,
    SB: crate::storage::StorageBackend,
>(
    Path(id): Path<String>,
    State(state): State<Arc<ExtendedAppState<PR, UR, SB>>>,
) -> Result<Json<Vec<PostVersionResponse>>, StatusCode> {
    let id_num: i64 = match id.parse() {
        Ok(num) => num,
        Err(_) => return Err(StatusCode::BAD_REQUEST),
    };

    match state.app_state.post_repository.get_versions(&id_num).await {
        Ok(versions) => Ok(Json(
            versions
                .into_iter()
                .map(PostVersionResponse::from)
                .collect(),
        )),
        Err(_) => Err(StatusCode::INTERNAL_SERVER_ERROR),
    }
}

/// Get a specific version
#[utoipa::path(
    get,
    path = "/api/posts/{post_id}/versions/{version_id}",
    params(
        ("post_id" = String, Path, description = "Post ID"),
        ("version_id" = String, Path, description = "Version ID")
    ),
    responses(
        (status = 200, description = "Successfully retrieved version", body = PostVersionResponse),
        (status = 404, description = "Version not found"),
        (status = 400, description = "Bad request"),
        (status = 500, description = "Internal server error")
    ),
    tag = "Posts"
)]
pub async fn get_post_version<
    PR: PostRepository,
    UR: UserRepository,
    SB: crate::storage::StorageBackend,
>(
    Path((post_id, version_id)): Path<(String, String)>,
    State(state): State<Arc<ExtendedAppState<PR, UR, SB>>>,
) -> Result<Json<PostVersionResponse>, StatusCode> {
    let version_id_num: i64 = match version_id.parse() {
        Ok(num) => num,
        Err(_) => return Err(StatusCode::BAD_REQUEST),
    };

    match state
        .app_state
        .post_repository
        .get_version(&version_id_num)
        .await
    {
        Ok(Some(version)) => Ok(Json(PostVersionResponse::from(version))),
        Ok(None) => Err(StatusCode::NOT_FOUND),
        Err(_) => Err(StatusCode::INTERNAL_SERVER_ERROR),
    }
}

/// Restore a post from a version
#[utoipa::path(
    post,
    path = "/api/posts/{post_id}/versions/{version_id}/restore",
    params(
        ("post_id" = String, Path, description = "Post ID"),
        ("version_id" = String, Path, description = "Version ID")
    ),
    responses(
        (status = 200, description = "Successfully restored post", body = PostResponse),
        (status = 404, description = "Post or version not found"),
        (status = 400, description = "Bad request"),
        (status = 500, description = "Internal server error")
    ),
    tag = "Posts"
)]
pub async fn restore_post_from_version<
    PR: PostRepository,
    UR: UserRepository,
    SB: crate::storage::StorageBackend,
>(
    Path((post_id, version_id)): Path<(String, String)>,
    State(state): State<Arc<ExtendedAppState<PR, UR, SB>>>,
    Extension(current_user): Extension<Arc<crate::auth::middleware::CurrentUser>>,
) -> Result<Json<PostResponse>, StatusCode> {
    let post_id_num: i64 = match post_id.parse() {
        Ok(num) => num,
        Err(_) => return Err(StatusCode::BAD_REQUEST),
    };
    let version_id_num: i64 = match version_id.parse() {
        Ok(num) => num,
        Err(_) => return Err(StatusCode::BAD_REQUEST),
    };

    // Get post to check ownership
    let post = match state
        .app_state
        .post_repository
        .find_by_id(&post_id_num)
        .await
    {
        Ok(Some(p)) => p,
        Ok(None) => return Err(StatusCode::NOT_FOUND),
        Err(_) => return Err(StatusCode::INTERNAL_SERVER_ERROR),
    };

    // Check if user is author or admin
    if !current_user.is_author_or_admin(post.author_id) {
        return Err(StatusCode::FORBIDDEN);
    }

    match state
        .app_state
        .post_repository
        .restore_from_version(&post_id_num, &version_id_num, current_user.id)
        .await
    {
        Ok(Some(restored_post)) => Ok(Json(PostResponse::from(restored_post))),
        Ok(None) => Err(StatusCode::NOT_FOUND),
        Err(_) => Err(StatusCode::INTERNAL_SERVER_ERROR),
    }
}

// Draft management endpoints

/// Save or update a draft
#[utoipa::path(
    post,
    path = "/api/drafts",
    request_body = SaveDraftRequest,
    responses(
        (status = 200, description = "Successfully saved draft", body = PostDraftResponse),
        (status = 400, description = "Bad request"),
        (status = 500, description = "Internal server error")
    ),
    tag = "Drafts"
)]
pub async fn save_draft<
    PR: PostRepository,
    UR: UserRepository,
    SB: crate::storage::StorageBackend,
>(
    State(state): State<Arc<ExtendedAppState<PR, UR, SB>>>,
    Extension(current_user): Extension<Arc<crate::auth::middleware::CurrentUser>>,
    axum::Json(payload): axum::Json<SaveDraftRequest>,
) -> Result<Json<PostDraftResponse>, StatusCode> {
    // Use current user ID from context
    let author_id = current_user.id;

    match state
        .app_state
        .post_repository
        .save_draft(author_id, payload)
        .await
    {
        Ok(draft) => Ok(Json(PostDraftResponse::from(draft))),
        Err(_) => Err(StatusCode::INTERNAL_SERVER_ERROR),
    }
}

/// Get a draft (for editing a post or creating a new one)
#[utoipa::path(
    get,
    path = "/api/drafts",
    params(
        ("post_id" = Option<String>, Query, description = "Post ID (optional, for editing existing post)")
    ),
    responses(
        (status = 200, description = "Successfully retrieved draft", body = PostDraftResponse),
        (status = 404, description = "Draft not found"),
        (status = 400, description = "Bad request"),
        (status = 500, description = "Internal server error")
    ),
    tag = "Drafts"
)]
pub async fn get_draft<
    PR: PostRepository,
    UR: UserRepository,
    SB: crate::storage::StorageBackend,
>(
    State(state): State<Arc<ExtendedAppState<PR, UR, SB>>>,
    Extension(current_user): Extension<Arc<crate::auth::middleware::CurrentUser>>,
    axum::extract::Query(params): axum::extract::Query<std::collections::HashMap<String, String>>,
) -> Result<Json<PostDraftResponse>, StatusCode> {
    // Use current user ID from context
    let author_id = current_user.id;

    let post_id = params.get("post_id").and_then(|s| s.parse::<i64>().ok());

    match state
        .app_state
        .post_repository
        .get_draft(post_id, author_id)
        .await
    {
        Ok(Some(draft)) => Ok(Json(PostDraftResponse::from(draft))),
        Ok(None) => Err(StatusCode::NOT_FOUND),
        Err(_) => Err(StatusCode::INTERNAL_SERVER_ERROR),
    }
}

/// Get all drafts for the current user
#[utoipa::path(
    get,
    path = "/api/drafts/all",
    responses(
        (status = 200, description = "Successfully retrieved drafts", body = Vec<PostDraftResponse>),
        (status = 500, description = "Internal server error")
    ),
    tag = "Drafts"
)]
pub async fn get_all_drafts<
    PR: PostRepository,
    UR: UserRepository,
    SB: crate::storage::StorageBackend,
>(
    State(state): State<Arc<ExtendedAppState<PR, UR, SB>>>,
    Extension(current_user): Extension<Arc<crate::auth::middleware::CurrentUser>>,
) -> Result<Json<Vec<PostDraftResponse>>, StatusCode> {
    // Use current user ID from context
    let author_id = current_user.id;

    match state
        .app_state
        .post_repository
        .get_all_drafts(author_id)
        .await
    {
        Ok(drafts) => Ok(Json(
            drafts.into_iter().map(PostDraftResponse::from).collect(),
        )),
        Err(_) => Err(StatusCode::INTERNAL_SERVER_ERROR),
    }
}

/// Delete a draft
#[utoipa::path(
    delete,
    path = "/api/drafts",
    params(
        ("post_id" = Option<String>, Query, description = "Post ID (optional)")
    ),
    responses(
        (status = 204, description = "Successfully deleted draft"),
        (status = 404, description = "Draft not found"),
        (status = 400, description = "Bad request"),
        (status = 500, description = "Internal server error")
    ),
    tag = "Drafts"
)]
pub async fn delete_draft<
    PR: PostRepository,
    UR: UserRepository,
    SB: crate::storage::StorageBackend,
>(
    State(state): State<Arc<ExtendedAppState<PR, UR, SB>>>,
    Extension(current_user): Extension<Arc<crate::auth::middleware::CurrentUser>>,
    axum::extract::Query(params): axum::extract::Query<std::collections::HashMap<String, String>>,
) -> Result<StatusCode, StatusCode> {
    // Use current user ID from context
    let author_id = current_user.id;

    let post_id = params.get("post_id").and_then(|s| s.parse::<i64>().ok());

    match state
        .app_state
        .post_repository
        .delete_draft(post_id, author_id)
        .await
    {
        Ok(_) => Ok(StatusCode::NO_CONTENT),
        Err(_) => Err(StatusCode::INTERNAL_SERVER_ERROR),
    }
}

// Documentation wrapper functions
#[utoipa::path(
    get,
    path = "/api/posts/{id}/versions",
    params(
        ("id" = String, Path, description = "Post ID")
    ),
    responses(
        (status = 200, description = "Successfully retrieved versions", body = Vec<PostVersionResponse>),
        (status = 500, description = "Internal server error")
    ),
    tag = "Posts"
)]
#[allow(dead_code)]
fn get_post_versions_doc(_id: String) {}

#[utoipa::path(
    get,
    path = "/api/posts/{post_id}/versions/{version_id}",
    params(
        ("post_id" = String, Path, description = "Post ID"),
        ("version_id" = String, Path, description = "Version ID")
    ),
    responses(
        (status = 200, description = "Successfully retrieved version", body = PostVersionResponse),
        (status = 404, description = "Version not found"),
        (status = 500, description = "Internal server error")
    ),
    tag = "Posts"
)]
#[allow(dead_code)]
fn get_post_version_doc(_post_id: String, _version_id: String) {}

#[utoipa::path(
    post,
    path = "/api/posts/{post_id}/versions/{version_id}/restore",
    params(
        ("post_id" = String, Path, description = "Post ID"),
        ("version_id" = String, Path, description = "Version ID")
    ),
    responses(
        (status = 200, description = "Successfully restored post", body = PostResponse),
        (status = 404, description = "Post or version not found"),
        (status = 500, description = "Internal server error")
    ),
    tag = "Posts"
)]
#[allow(dead_code)]
fn restore_post_from_version_doc(_post_id: String, _version_id: String) {}
