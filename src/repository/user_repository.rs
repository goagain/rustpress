use crate::dto::{CreateUserRequest, User};
use async_trait::async_trait;

/// User repository interface
#[async_trait]
pub trait UserRepository: Send + Sync {
    async fn find_all(&self) -> Result<Vec<User>, Box<dyn std::error::Error + Send + Sync>>;
    async fn find_by_id(&self, id: &i64) -> Result<Option<User>, Box<dyn std::error::Error + Send + Sync>>;
    async fn find_by_username(&self, username: &str) -> Result<Option<User>, Box<dyn std::error::Error + Send + Sync>>;
    async fn find_by_email(&self, email: &str) -> Result<Option<User>, Box<dyn std::error::Error + Send + Sync>>;
    /// Create a new user
    /// ID, created_at and updated_at are automatically generated, no need to provide in request
    async fn create(&self, request: CreateUserRequest) -> Result<User, Box<dyn std::error::Error + Send + Sync>>;
    async fn update(&self, id: &i64, user: User) -> Result<Option<User>, Box<dyn std::error::Error + Send + Sync>>;
    async fn delete(&self, id: &i64) -> Result<bool, Box<dyn std::error::Error + Send + Sync>>;
}

