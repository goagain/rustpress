use crate::dto::{AdminUserListResponse, AdminBanUserRequest, AdminResetPasswordRequest, AdminResetPasswordResponse, UserResponse};
use crate::repository::{PostRepository, UserRepository};
use axum::{
    extract::{Extension, Path, State},
    http::StatusCode,
    response::Json,
};
use sea_orm::{EntityTrait, Set, ActiveModelTrait};
use std::sync::Arc;

/// Get all users (admin view with ban status)
#[utoipa::path(
    get,
    path = "/api/admin/users",
    responses(
        (status = 200, description = "Successfully retrieved user list", body = Vec<AdminUserListResponse>),
        (status = 500, description = "Internal server error")
    ),
    tag = "Admin"
)]
pub async fn get_all_users<PR: PostRepository, UR: UserRepository, SB: crate::storage::StorageBackend>(
    State(state): State<Arc<crate::api::post_controller::ExtendedAppState<PR, UR, SB>>>,
    Extension(_current_user): Extension<Arc<crate::auth::middleware::CurrentUser>>,
) -> Result<Json<Vec<AdminUserListResponse>>, StatusCode> {
    let users = state.app_state.user_repository.find_all().await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let db = get_db_connection(&state);

    let mut result = Vec::new();
    for user in users {
        // Check if user is banned
        let user_model = crate::entity::users::Entity::find_by_id(user.id)
            .one(&*db)
            .await
            .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

        let is_banned = user_model
            .map(|u| u.banned_at.is_some())
            .unwrap_or(false);

        result.push(AdminUserListResponse {
            user: UserResponse::from(user),
            is_banned,
        });
    }

    Ok(Json(result))
}

/// Ban or unban a user
#[utoipa::path(
    post,
    path = "/api/admin/users/{id}/ban",
    params(
        ("id" = i64, Path, description = "User ID")
    ),
    request_body = AdminBanUserRequest,
    responses(
        (status = 200, description = "Successfully updated ban status"),
        (status = 404, description = "User not found"),
        (status = 500, description = "Internal server error")
    ),
    tag = "Admin"
)]
pub async fn ban_user<PR: PostRepository, UR: UserRepository, SB: crate::storage::StorageBackend>(
    Path(id): Path<i64>,
    State(state): State<Arc<crate::api::post_controller::ExtendedAppState<PR, UR, SB>>>,
    Extension(current_user): Extension<Arc<crate::auth::middleware::CurrentUser>>,
    axum::Json(payload): axum::Json<AdminBanUserRequest>,
) -> Result<StatusCode, StatusCode> {
    // Prevent banning yourself
    if current_user.id == id {
        return Err(StatusCode::BAD_REQUEST);
    }

    let db = get_db_connection(&state);

    let mut user: crate::entity::users::ActiveModel = crate::entity::users::Entity::find_by_id(id)
        .one(&*db)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
        .ok_or(StatusCode::NOT_FOUND)?
        .into();

    if payload.banned {
        user.banned_at = Set(Some(chrono::Utc::now().into()));
    } else {
        user.banned_at = Set(None);
    }

    user.update(&*db)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(StatusCode::OK)
}

/// Reset user password
#[utoipa::path(
    post,
    path = "/api/admin/users/{id}/reset-password",
    params(
        ("id" = i64, Path, description = "User ID")
    ),
    request_body = AdminResetPasswordRequest,
    responses(
        (status = 200, description = "Successfully reset password", body = AdminResetPasswordResponse),
        (status = 404, description = "User not found"),
        (status = 500, description = "Internal server error")
    ),
    tag = "Admin"
)]
pub async fn reset_user_password<PR: PostRepository, UR: UserRepository, SB: crate::storage::StorageBackend>(
    Path(id): Path<i64>,
    State(state): State<Arc<crate::api::post_controller::ExtendedAppState<PR, UR, SB>>>,
    Extension(_current_user): Extension<Arc<crate::auth::middleware::CurrentUser>>,
    axum::Json(payload): axum::Json<AdminResetPasswordRequest>,
) -> Result<Json<AdminResetPasswordResponse>, StatusCode> {
    let db = get_db_connection(&state);

    let mut user: crate::entity::users::ActiveModel = crate::entity::users::Entity::find_by_id(id)
        .one(&*db)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
        .ok_or(StatusCode::NOT_FOUND)?
        .into();

    // Hash new password
    let password_hash = bcrypt::hash(&payload.new_password, bcrypt::DEFAULT_COST)
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    user.password_hash = Set(password_hash);

    user.update(&*db)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(Json(AdminResetPasswordResponse {
        success: true,
        message: "Password reset successfully".to_string(),
    }))
}

// Helper function to get database connection from state
fn get_db_connection<PR: PostRepository, UR: UserRepository, SB: crate::storage::StorageBackend>(
    state: &Arc<crate::api::post_controller::ExtendedAppState<PR, UR, SB>>,
) -> Arc<sea_orm::DatabaseConnection> {
    state.db.clone()
}
