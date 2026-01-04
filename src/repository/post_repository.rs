use crate::dto::{Post, CreatePostRequest, PostVersion, PostDraft, SaveDraftRequest};
use async_trait::async_trait;

/// Post repository interface
#[allow(unused)]
#[async_trait]
pub trait PostRepository: Send + Sync {
    async fn find_all(&self) -> Result<Vec<Post>, Box<dyn std::error::Error + Send + Sync>>;
    async fn find_by_id(
        &self,
        id: &i64,
    ) -> Result<Option<Post>, Box<dyn std::error::Error + Send + Sync>>;
    /// Create a new post
    /// ID, created_at and updated_at are automatically generated, no need to provide in request
    async fn create(&self, request: CreatePostRequest) -> Result<Post, Box<dyn std::error::Error + Send + Sync>>;
    async fn update(
        &self,
        id: &i64,
        post: Post,
        create_version: bool,
        change_note: Option<String>,
        user_id: i64,
    ) -> Result<Option<Post>, Box<dyn std::error::Error + Send + Sync>>;
    async fn delete(&self, id: &i64) -> Result<bool, Box<dyn std::error::Error + Send + Sync>>;
    async fn archive(&self, id: &i64) -> Result<bool, Box<dyn std::error::Error + Send + Sync>>;
    async fn restore(&self, id: &i64) -> Result<bool, Box<dyn std::error::Error + Send + Sync>>;
    async fn hard_delete(&self, id: &i64) -> Result<bool, Box<dyn std::error::Error + Send + Sync>>;
    
    // Version management
    async fn get_versions(&self, post_id: &i64) -> Result<Vec<PostVersion>, Box<dyn std::error::Error + Send + Sync>>;
    async fn get_version(&self, version_id: &i64) -> Result<Option<PostVersion>, Box<dyn std::error::Error + Send + Sync>>;
    async fn restore_from_version(&self, post_id: &i64, version_id: &i64, user_id: i64) -> Result<Option<Post>, Box<dyn std::error::Error + Send + Sync>>;
    
    // Draft management
    async fn save_draft(&self, author_id: i64, request: SaveDraftRequest) -> Result<PostDraft, Box<dyn std::error::Error + Send + Sync>>;
    async fn get_draft(&self, post_id: Option<i64>, author_id: i64) -> Result<Option<PostDraft>, Box<dyn std::error::Error + Send + Sync>>;
    async fn delete_draft(&self, post_id: Option<i64>, author_id: i64) -> Result<bool, Box<dyn std::error::Error + Send + Sync>>;
    async fn get_all_drafts(&self, author_id: i64) -> Result<Vec<PostDraft>, Box<dyn std::error::Error + Send + Sync>>;

    // Category statistics
    async fn get_category_stats(&self) -> Result<Vec<(String, i64)>, Box<dyn std::error::Error + Send + Sync>>;
}
