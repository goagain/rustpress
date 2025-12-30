use crate::dto::{Post, CreatePostRequest};
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
    ) -> Result<Option<Post>, Box<dyn std::error::Error + Send + Sync>>;
    async fn delete(&self, id: &i64) -> Result<bool, Box<dyn std::error::Error + Send + Sync>>;
    async fn archive(&self, id: &i64) -> Result<bool, Box<dyn std::error::Error + Send + Sync>>;
    async fn restore(&self, id: &i64) -> Result<bool, Box<dyn std::error::Error + Send + Sync>>;
    async fn hard_delete(&self, id: &i64) -> Result<bool, Box<dyn std::error::Error + Send + Sync>>;
}
