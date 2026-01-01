use crate::dto::{AdminPostListResponse, PostResponse};
use crate::repository::{PostRepository, UserRepository};
use axum::{
    extract::{Extension, Path, State},
    http::StatusCode,
    response::Json,
};
use std::sync::Arc;

/// Get all posts (admin view)
#[utoipa::path(
    get,
    path = "/api/admin/posts",
    responses(
        (status = 200, description = "Successfully retrieved post list", body = Vec<AdminPostListResponse>),
        (status = 500, description = "Internal server error")
    ),
    tag = "Admin"
)]
pub async fn get_all_posts<PR: PostRepository, UR: UserRepository, SB: crate::storage::StorageBackend>(
    State(state): State<Arc<crate::api::post_controller::ExtendedAppState<PR, UR, SB>>>,
    Extension(_current_user): Extension<Arc<crate::auth::middleware::CurrentUser>>,
) -> Result<Json<Vec<AdminPostListResponse>>, StatusCode> {
    let posts = state.app_state.post_repository.find_all().await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let result: Vec<AdminPostListResponse> = posts
        .into_iter()
        .map(|post| AdminPostListResponse {
            post: PostResponse::from(post),
        })
        .collect();

    Ok(Json(result))
}

/// Delete a post (admin can delete any post)
#[utoipa::path(
    delete,
    path = "/api/admin/posts/{id}",
    params(
        ("id" = String, Path, description = "Post ID")
    ),
    responses(
        (status = 204, description = "Successfully deleted post"),
        (status = 404, description = "Post not found"),
        (status = 500, description = "Internal server error")
    ),
    tag = "Admin"
)]
pub async fn admin_delete_post<PR: PostRepository, UR: UserRepository, SB: crate::storage::StorageBackend>(
    Path(id): Path<String>,
    State(state): State<Arc<crate::api::post_controller::ExtendedAppState<PR, UR, SB>>>,
    Extension(_current_user): Extension<Arc<crate::auth::middleware::CurrentUser>>,
) -> Result<StatusCode, StatusCode> {
    let id_num: i64 = id.parse().map_err(|_| StatusCode::BAD_REQUEST)?;

    match state.app_state.post_repository.delete(&id_num).await {
        Ok(true) => Ok(StatusCode::NO_CONTENT),
        Ok(false) => Err(StatusCode::NOT_FOUND),
        Err(_) => Err(StatusCode::INTERNAL_SERVER_ERROR),
    }
}
