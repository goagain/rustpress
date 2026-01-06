use crate::entity::openai_api_keys;
use crate::repository::OpenAIApiKeyRepository;
use async_trait::async_trait;
use sea_orm::{ActiveModelTrait, ColumnTrait, DatabaseConnection, EntityTrait, QueryFilter};
use std::sync::Arc;

/// PostgreSQL implementation of OpenAI API key repository (using SeaORM)
pub struct PostgresOpenAIApiKeyRepository {
    db: Arc<DatabaseConnection>,
}

impl PostgresOpenAIApiKeyRepository {
    pub fn new(db: DatabaseConnection) -> Self {
        Self { db: Arc::new(db) }
    }
}

#[async_trait]
impl OpenAIApiKeyRepository for PostgresOpenAIApiKeyRepository {
    async fn find_all(
        &self,
    ) -> Result<Vec<openai_api_keys::Model>, Box<dyn std::error::Error + Send + Sync>> {
        let models = openai_api_keys::Entity::find()
            .all(self.db.as_ref())
            .await?;

        Ok(models)
    }

    async fn find_by_id(
        &self,
        id: &i64,
    ) -> Result<Option<openai_api_keys::Model>, Box<dyn std::error::Error + Send + Sync>> {
        let model = openai_api_keys::Entity::find_by_id(*id)
            .one(self.db.as_ref())
            .await?;

        Ok(model)
    }

    async fn find_by_name(
        &self,
        name: &str,
    ) -> Result<Option<openai_api_keys::Model>, Box<dyn std::error::Error + Send + Sync>> {
        let model = openai_api_keys::Entity::find()
            .filter(openai_api_keys::Column::Name.eq(name))
            .one(self.db.as_ref())
            .await?;

        Ok(model)
    }

    async fn find_default(
        &self,
    ) -> Result<Option<openai_api_keys::Model>, Box<dyn std::error::Error + Send + Sync>> {
        let model = openai_api_keys::Entity::find()
            .filter(openai_api_keys::Column::IsDefault.eq(true))
            .one(self.db.as_ref())
            .await?;

        Ok(model)
    }

    async fn create(
        &self,
        key: openai_api_keys::ActiveModel,
    ) -> Result<openai_api_keys::Model, Box<dyn std::error::Error + Send + Sync>> {
        let result = key.insert(self.db.as_ref()).await?;
        Ok(result)
    }

    async fn update(
        &self,
        id: &i64,
        mut key: openai_api_keys::ActiveModel,
    ) -> Result<Option<openai_api_keys::Model>, Box<dyn std::error::Error + Send + Sync>> {
        // Set the ID for the update
        key.id = sea_orm::Set(*id);

        let result = key.update(self.db.as_ref()).await?;
        Ok(Some(result))
    }

    async fn delete(&self, id: &i64) -> Result<bool, Box<dyn std::error::Error + Send + Sync>> {
        let result = openai_api_keys::Entity::delete_by_id(*id)
            .exec(self.db.as_ref())
            .await?;

        Ok(result.rows_affected > 0)
    }
}
