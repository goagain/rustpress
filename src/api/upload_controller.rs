//! Image upload controller
//! 
//! Handles image upload requests and delegates to storage backend

use crate::storage::StorageBackend;
use axum::{
    extract::{Multipart, State},
    http::StatusCode,
    response::Json,
};
use serde::Serialize;
use std::sync::Arc;

/// Upload response DTO
#[derive(Debug, Serialize)]
pub struct UploadResponse {
    pub url: String,
    pub filename: String,
}

/// Upload image endpoint
/// 
/// Accepts multipart/form-data with an "image" field
/// Returns the URL where the uploaded image can be accessed
pub async fn upload_image<PR, UR, SB>(
    State(state): State<Arc<crate::api::post_controller::ExtendedAppState<PR, UR, SB>>>,
    mut multipart: Multipart,
) -> Result<Json<UploadResponse>, StatusCode>
where
    PR: crate::repository::PostRepository,
    UR: crate::repository::UserRepository,
    SB: StorageBackend,
{
    // Find the image field in the multipart data
    while let Some(field) = multipart.next_field().await.map_err(|_| StatusCode::BAD_REQUEST)? {
        let field_name = field.name();
        
        // Only process fields named "image"
        if field_name != Some("image") {
            continue;
        }

        let file_name = field
            .file_name()
            .ok_or(StatusCode::BAD_REQUEST)?
            .to_string();
        
        let content_type = field
            .content_type()
            .unwrap_or("application/octet-stream")
            .to_string();

        // Validate content type (only images)
        if !content_type.starts_with("image/") {
            return Err(StatusCode::BAD_REQUEST);
        }

        // Read file data
        let file_data = field
            .bytes()
            .await
            .map_err(|_| StatusCode::BAD_REQUEST)?
            .to_vec();

        // Validate file size (max 10MB)
        const MAX_FILE_SIZE: usize = 10 * 1024 * 1024; // 10MB
        if file_data.len() > MAX_FILE_SIZE {
            return Err(StatusCode::PAYLOAD_TOO_LARGE);
        }

        // Upload to storage backend
        let url = state.storage
            .upload_file(file_data, file_name.clone(), content_type)
            .await
            .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

        return Ok(Json(UploadResponse { url, filename: file_name }));
    }

    Err(StatusCode::BAD_REQUEST)
}

