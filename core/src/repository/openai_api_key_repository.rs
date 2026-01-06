use crate::entity::openai_api_keys;
use async_trait::async_trait;

/// OpenAI API key repository interface
#[async_trait]
pub trait OpenAIApiKeyRepository: Send + Sync {
    async fn find_all(
        &self,
    ) -> Result<Vec<openai_api_keys::Model>, Box<dyn std::error::Error + Send + Sync>>;
    async fn find_by_id(
        &self,
        id: &i64,
    ) -> Result<Option<openai_api_keys::Model>, Box<dyn std::error::Error + Send + Sync>>;
    async fn find_by_name(
        &self,
        name: &str,
    ) -> Result<Option<openai_api_keys::Model>, Box<dyn std::error::Error + Send + Sync>>;
    async fn find_default(
        &self,
    ) -> Result<Option<openai_api_keys::Model>, Box<dyn std::error::Error + Send + Sync>>;
    async fn create(
        &self,
        key: openai_api_keys::ActiveModel,
    ) -> Result<openai_api_keys::Model, Box<dyn std::error::Error + Send + Sync>>;
    async fn update(
        &self,
        id: &i64,
        key: openai_api_keys::ActiveModel,
    ) -> Result<Option<openai_api_keys::Model>, Box<dyn std::error::Error + Send + Sync>>;
    async fn delete(&self, id: &i64) -> Result<bool, Box<dyn std::error::Error + Send + Sync>>;
}
