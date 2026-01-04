use crate::entity::{post_drafts, post_versions, posts};
use chrono::{DateTime, Utc};
use sea_orm::prelude::DateTimeWithTimeZone;
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

/// Post business entity (DTO)
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct Post {
    pub id: i64,
    pub title: String,
    pub content: String,
    pub category: Option<String>,
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
    #[serde(default)]
    pub category: Option<String>,
    pub author_id: i64,
}

/// Request DTO for updating a post
#[derive(Debug, Deserialize, ToSchema)]
pub struct UpdatePostRequest {
    pub title: Option<String>,
    pub content: Option<String>,
    pub category: Option<String>,
    #[serde(default)]
    pub create_version: bool,
    pub change_note: Option<String>,
}

/// Post response DTO (can be extended, e.g., to add additional display fields)
#[derive(Debug, Clone, Serialize, ToSchema)]
pub struct PostResponse {
    pub id: i64,
    pub title: String,
    pub content: String,
    pub category: Option<String>,
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

/// Post version DTO
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct PostVersion {
    pub id: i64,
    pub post_id: i64,
    pub title: String,
    pub content: String,
    pub category: String,
    pub version_number: i32,
    pub created_at: DateTimeWithTimeZone,
    pub created_by: i64,
    pub change_note: Option<String>,
}

/// Post version response DTO
#[derive(Debug, Clone, Serialize, ToSchema)]
pub struct PostVersionResponse {
    pub id: i64,
    pub post_id: i64,
    pub title: String,
    pub content: String,
    pub category: String,
    pub version_number: i32,
    pub created_at: DateTime<Utc>,
    pub created_by: i64,
    pub change_note: Option<String>,
}

/// Request DTO for creating a version
#[derive(Debug, Deserialize, ToSchema)]
pub struct CreateVersionRequest {
    pub change_note: Option<String>,
}

/// Post draft DTO
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct PostDraft {
    pub id: i64,
    pub post_id: Option<i64>,
    pub title: String,
    pub content: String,
    pub category: String,
    pub author_id: i64,
    pub created_at: DateTimeWithTimeZone,
    pub updated_at: DateTimeWithTimeZone,
}

/// Post draft response DTO
#[derive(Debug, Clone, Serialize, ToSchema)]
pub struct PostDraftResponse {
    pub id: i64,
    pub post_id: Option<i64>,
    pub title: String,
    pub content: String,
    pub category: String,
    pub author_id: i64,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Request DTO for saving/updating a draft
#[derive(Debug, Deserialize, ToSchema)]
pub struct SaveDraftRequest {
    pub post_id: Option<i64>,
    pub title: String,
    pub content: String,
    pub category: String,
}

/// Convert from database Model to PostVersion DTO
impl From<post_versions::Model> for PostVersion {
    fn from(model: post_versions::Model) -> Self {
        PostVersion {
            id: model.id,
            post_id: model.post_id,
            title: model.title,
            content: model.content,
            category: model.category,
            version_number: model.version_number,
            created_at: model.created_at,
            created_by: model.created_by,
            change_note: model.change_note,
        }
    }
}

/// Convert from PostVersion to PostVersionResponse
impl From<PostVersion> for PostVersionResponse {
    fn from(version: PostVersion) -> Self {
        PostVersionResponse {
            id: version.id,
            post_id: version.post_id,
            title: version.title,
            content: version.content,
            category: version.category,
            version_number: version.version_number,
            created_at: version.created_at.into(),
            created_by: version.created_by,
            change_note: version.change_note,
        }
    }
}

/// Convert from database Model to PostDraft DTO
impl From<post_drafts::Model> for PostDraft {
    fn from(model: post_drafts::Model) -> Self {
        PostDraft {
            id: model.id,
            post_id: model.post_id,
            title: model.title,
            content: model.content,
            category: model.category,
            author_id: model.author_id,
            created_at: model.created_at,
            updated_at: model.updated_at,
        }
    }
}

/// Convert from PostDraft to PostDraftResponse
impl From<PostDraft> for PostDraftResponse {
    fn from(draft: PostDraft) -> Self {
        PostDraftResponse {
            id: draft.id,
            post_id: draft.post_id,
            title: draft.title,
            content: draft.content,
            category: draft.category,
            author_id: draft.author_id,
            created_at: draft.created_at.into(),
            updated_at: draft.updated_at.into(),
        }
    }
}
