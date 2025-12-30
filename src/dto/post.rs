use chrono::{DateTime, Utc};
use sea_orm::prelude::DateTimeWithTimeZone;
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use crate::entity::posts;

/// Post business entity (DTO)
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct Post {
    pub id: i64,
    pub title: String,
    pub content: String,
    pub category: String,
    pub author_id: i64,
    pub created_at: DateTimeWithTimeZone,
    pub updated_at: DateTimeWithTimeZone,
    pub archived_at: Option<DateTimeWithTimeZone>,
    pub deleted_at: Option<DateTimeWithTimeZone>,
}

/// Request DTO for creating a post
#[derive(Debug, Deserialize, ToSchema)]
pub struct CreatePostRequest {
    pub title: String,
    pub content: String,
    pub category: String,
    pub author_id: i64,
}

/// Request DTO for updating a post
#[derive(Debug, Deserialize, ToSchema)]
pub struct UpdatePostRequest {
    pub title: Option<String>,
    pub content: Option<String>,
    pub category: Option<String>,
}

/// Post response DTO (can be extended, e.g., to add additional display fields)
#[derive(Debug, Clone, Serialize, ToSchema)]
pub struct PostResponse {
    pub id: i64,
    pub title: String,
    pub content: String,
    pub category: String,
    pub author_id: i64,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Convert from database Model to business DTO
impl From<posts::Model> for Post {
    fn from(model: posts::Model) -> Self {
        Post {
            id: model.id,
            title: model.title,
            content: model.content,
            category: model.category,
            author_id: model.author_id,
            created_at: model.created_at.into(),
            updated_at: model.updated_at.into(),
            archived_at: model.archived_at,
            deleted_at: model.deleted_at,
        }
    }
}

/// Convert from business DTO to API response DTO
impl From<Post> for PostResponse {
    fn from(post: Post) -> Self {
        PostResponse {
            id: post.id,
            title: post.title,
            content: post.content,
            category: post.category,
            author_id: post.author_id,
            created_at: post.created_at.into(),
            updated_at: post.updated_at.into(),
        }
    }
}

