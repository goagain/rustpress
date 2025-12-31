use crate::auth::JwtUtil;
use crate::dto::{
    CreateUserRequest, LoginRequest, LoginResponse, RefreshTokenRequest, RefreshTokenResponse,
    User, UserResponse, UserRole,
};
use crate::repository::{PostRepository, UserRepository};
use crate::repository::postgres_user_repository::verify_password;
use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::Json,
};
use std::sync::Arc;

/// Get all users
///
/// Returns a list of all users
#[utoipa::path(
    get,
    path = "/api/users",
    responses(
        (status = 200, description = "Successfully retrieved user list", body = Vec<UserResponse>),
        (status = 500, description = "Internal server error")
    ),
    tag = "Users"
)]
pub async fn get_users<PR: PostRepository, UR: UserRepository, SB: crate::storage::StorageBackend>(
    State(state): State<Arc<crate::api::post_controller::ExtendedAppState<PR, UR, SB>>>,
) -> Result<Json<Vec<UserResponse>>, StatusCode> {
    match state.app_state.user_repository.find_all().await {
        Ok(users) => Ok(Json(users.into_iter().map(UserResponse::from).collect())),
        Err(_) => Err(StatusCode::INTERNAL_SERVER_ERROR),
    }
}

/// Get a single user by ID
///
/// Get detailed information of a single user by user ID
#[utoipa::path(
    get,
    path = "/api/users/{id}",
    params(
        ("id" = i64, Path, description = "User ID")
    ),
    responses(
        (status = 200, description = "Successfully retrieved user", body = UserResponse),
        (status = 404, description = "User not found"),
        (status = 500, description = "Internal server error")
    ),
    tag = "Users"
)]
pub async fn get_user<PR: PostRepository, UR: UserRepository, SB: crate::storage::StorageBackend>(
    Path(id): Path<i64>,
    State(state): State<Arc<crate::api::post_controller::ExtendedAppState<PR, UR, SB>>>,
) -> Result<Json<UserResponse>, StatusCode> {
    match state.app_state.user_repository.find_by_id(&id).await {
        Ok(Some(user)) => Ok(Json(UserResponse::from(user))),
        Ok(None) => Err(StatusCode::NOT_FOUND),
        Err(_) => Err(StatusCode::INTERNAL_SERVER_ERROR),
    }
}

/// Create a new user
///
/// Create a new user account
#[utoipa::path(
    post,
    path = "/api/users",
    request_body = CreateUserRequest,
    responses(
        (status = 201, description = "Successfully created user", body = UserResponse),
        (status = 400, description = "Bad request"),
        (status = 409, description = "Username or email already exists"),
        (status = 500, description = "Internal server error")
    ),
    tag = "Users"
)]
pub async fn create_user<PR: PostRepository, UR: UserRepository, SB: crate::storage::StorageBackend>(
    State(state): State<Arc<crate::api::post_controller::ExtendedAppState<PR, UR, SB>>>,
    axum::Json(payload): axum::Json<CreateUserRequest>,
) -> Result<(axum::http::StatusCode, Json<UserResponse>), StatusCode> {
    // Check if username already exists
    if let Ok(Some(_)) = state.app_state.user_repository.find_by_username(&payload.username).await {
        return Err(StatusCode::CONFLICT);
    }

    // Check if email already exists
    if let Ok(Some(_)) = state.app_state.user_repository.find_by_email(&payload.email).await {
        return Err(StatusCode::CONFLICT);
    }

    match state.app_state.user_repository.create(payload).await {
        Ok(user) => Ok((StatusCode::CREATED, Json(UserResponse::from(user)))),
        Err(_) => Err(StatusCode::INTERNAL_SERVER_ERROR),
    }
}

/// Update a user
///
/// Update user information by ID
#[utoipa::path(
    put,
    path = "/api/users/{id}",
    params(
        ("id" = i64, Path, description = "User ID")
    ),
    request_body = CreateUserRequest,
    responses(
        (status = 200, description = "Successfully updated user", body = UserResponse),
        (status = 404, description = "User not found"),
        (status = 500, description = "Internal server error")
    ),
    tag = "Users"
)]
pub async fn update_user<PR: PostRepository, UR: UserRepository, SB: crate::storage::StorageBackend>(
    Path(id): Path<i64>,
    State(state): State<Arc<crate::api::post_controller::ExtendedAppState<PR, UR, SB>>>,
    axum::Json(payload): axum::Json<CreateUserRequest>,
) -> Result<Json<UserResponse>, StatusCode> {
    // Get existing user
    let existing_user = match state.app_state.user_repository.find_by_id(&id).await {
        Ok(Some(user)) => user,
        Ok(None) => return Err(StatusCode::NOT_FOUND),
        Err(_) => return Err(StatusCode::INTERNAL_SERVER_ERROR),
    };

    // Build updated user
    let updated_user = User {
        id: existing_user.id,
        username: payload.username,
        email: payload.email,
        role: payload.role,
        salt: existing_user.salt, // Keep original salt
        password_hash: existing_user.password_hash, // Keep original password, password update should be handled separately
        created_at: existing_user.created_at,
        updated_at: existing_user.updated_at,
    };

    match state.app_state.user_repository.update(&id, updated_user).await {
        Ok(Some(user)) => Ok(Json(UserResponse::from(user))),
        Ok(None) => Err(StatusCode::NOT_FOUND),
        Err(_) => Err(StatusCode::INTERNAL_SERVER_ERROR),
    }
}

/// Delete a user
///
/// Delete a user by ID
#[utoipa::path(
    delete,
    path = "/api/users/{id}",
    params(
        ("id" = i64, Path, description = "User ID")
    ),
    responses(
        (status = 204, description = "Successfully deleted user"),
        (status = 404, description = "User not found"),
        (status = 500, description = "Internal server error")
    ),
    tag = "Users"
)]
pub async fn delete_user<PR: PostRepository, UR: UserRepository, SB: crate::storage::StorageBackend>(
    Path(id): Path<i64>,
    State(state): State<Arc<crate::api::post_controller::ExtendedAppState<PR, UR, SB>>>,
) -> Result<StatusCode, StatusCode> {
    match state.app_state.user_repository.delete(&id).await {
        Ok(true) => Ok(StatusCode::NO_CONTENT),
        Ok(false) => Err(StatusCode::NOT_FOUND),
        Err(_) => Err(StatusCode::INTERNAL_SERVER_ERROR),
    }
}

/// User login
///
/// Login with username and password, returns access token and refresh token
#[utoipa::path(
    post,
    path = "/api/auth/login",
    request_body = LoginRequest,
    responses(
        (status = 200, description = "Login successful", body = LoginResponse),
        (status = 401, description = "Invalid username or password"),
        (status = 500, description = "Internal server error")
    ),
    tag = "Auth"
)]
pub async fn login<PR: PostRepository, UR: UserRepository, SB: crate::storage::StorageBackend>(
    State(state): State<Arc<crate::api::post_controller::ExtendedAppState<PR, UR, SB>>>,
    axum::Json(payload): axum::Json<LoginRequest>,
) -> Result<Json<LoginResponse>, StatusCode> {
    // Find user
    let user = match state.app_state.user_repository.find_by_username(&payload.username).await {
        Ok(Some(user)) => user,
        Ok(None) => return Err(StatusCode::UNAUTHORIZED),
        Err(_) => return Err(StatusCode::INTERNAL_SERVER_ERROR),
    };

    // Verify password
    if !verify_password(&payload.password, &user.password_hash).unwrap_or(false) {
        return Err(StatusCode::UNAUTHORIZED);
    }

    // Generate tokens
    let role_str = match user.role {
        UserRole::Root => "Root",
        UserRole::Admin => "Admin",
        UserRole::User => "User",
    };

    let access_token = JwtUtil::generate_access_token(user.id, user.username.clone(), role_str.to_string())
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let refresh_token = JwtUtil::generate_refresh_token(user.id, user.username, role_str.to_string())
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(Json(LoginResponse {
        access_token,
        refresh_token,
        token_type: "Bearer".to_string(),
        expires_in: 3600, // 1 hour
    }))
}

/// Refresh Access Token
///
/// Get a new access token using refresh token
#[utoipa::path(
    post,
    path = "/api/auth/refresh",
    request_body = RefreshTokenRequest,
    responses(
        (status = 200, description = "Successfully refreshed token", body = RefreshTokenResponse),
        (status = 401, description = "Invalid refresh token"),
        (status = 500, description = "Internal server error")
    ),
    tag = "Auth"
)]
pub async fn refresh_token<PR: PostRepository, UR: UserRepository, SB: crate::storage::StorageBackend>(
    State(state): State<Arc<crate::api::post_controller::ExtendedAppState<PR, UR, SB>>>,
    axum::Json(payload): axum::Json<RefreshTokenRequest>,
) -> Result<Json<RefreshTokenResponse>, StatusCode> {
    // Verify refresh token
    let claims = JwtUtil::verify_refresh_token(&payload.refresh_token)
        .map_err(|_| StatusCode::UNAUTHORIZED)?;

    // Get user information
    let user = match state.app_state.user_repository.find_by_id(&claims.sub).await {
        Ok(Some(user)) => user,
        Ok(None) => return Err(StatusCode::UNAUTHORIZED),
        Err(_) => return Err(StatusCode::INTERNAL_SERVER_ERROR),
    };

    // Generate new access token
    let role_str = match user.role {
        UserRole::Root => "Root",
        UserRole::Admin => "Admin",
        UserRole::User => "User",
    };

    let access_token = JwtUtil::generate_access_token(user.id, user.username, role_str.to_string())
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(Json(RefreshTokenResponse {
        access_token,
        token_type: "Bearer".to_string(),
        expires_in: 3600, // 1 hour
    }))
}

