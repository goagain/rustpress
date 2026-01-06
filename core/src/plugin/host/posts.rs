use crate::plugin::rustpress::plugin::posts::*;
use crate::repository::post_repository::PostRepository;

#[async_trait::async_trait]
impl Host for super::super::PluginHostState {
    async fn list_categories(&mut self) -> anyhow::Result<Vec<String>> {
        let repo = crate::repository::PostgresPostRepository::new(self.db.as_ref().clone());
        let mut categories = repo
            .get_category_stats()
            .await
            .map_err(|e| anyhow::anyhow!(e))?;

        categories.sort_by(|a, b| b.1.cmp(&a.1));
        Ok(categories
            .iter()
            .map(|(category, _count)| category.clone())
            .collect())
    }
}
