//! Plugin Executor - Handles WASM plugin loading and execution

use std::path::Path;
use std::sync::Arc;

/// Plugin executor that manages WASM plugin execution
#[derive(Clone)]
pub struct PluginExecutor {
    plugin_id: String,
    ai_helper: Option<Arc<super::ai::AiHelper>>,
    post_repo: Option<Arc<dyn crate::repository::PostRepository>>,
}

/// Loaded plugin instance with its hooks
#[derive(Clone)]
pub struct LoadedPlugin {
    pub plugin_id: String,
    pub registered_hooks: Vec<String>,
}

impl PluginExecutor {
    /// Create a new plugin executor for a specific plugin
    pub fn new(plugin_id: String) -> Self {
        Self {
            plugin_id,
            ai_helper: None,
            post_repo: None,
        }
    }

    /// Set the AI helper for this executor
    pub fn with_ai_helper(mut self, ai_helper: Arc<super::ai::AiHelper>) -> Self {
        self.ai_helper = Some(ai_helper);
        self
    }

    /// Set the post repository for this executor
    pub fn with_post_repo(mut self, post_repo: Arc<dyn crate::repository::PostRepository>) -> Self {
        self.post_repo = Some(post_repo);
        self
    }

    /// Log info message with plugin prefix
    pub fn log_info(&self, message: &str) {
        tracing::info!("[{}] {}", self.plugin_id, message);
    }

    /// Log warn message with plugin prefix
    pub fn log_warn(&self, message: &str) {
        tracing::warn!("[{}] {}", self.plugin_id, message);
    }

    /// Log error message with plugin prefix
    pub fn log_error(&self, message: &str) {
        tracing::error!("[{}] {}", self.plugin_id, message);
    }

    /// Log debug message with plugin prefix
    pub fn log_debug(&self, message: &str) {
        tracing::debug!("[{}] {}", self.plugin_id, message);
    }

    /// List categories (for host-side use)
    pub fn list_categories(
        &self,
    ) -> Vec<crate::plugin::plugin_world::exports::rustpress::system::hooks::CategoryInfo> {
        // Query database for real category statistics
        if let Some(repo) = &self.post_repo {
            match futures::executor::block_on(repo.get_category_stats()) {
                Ok(stats) => stats
                    .into_iter()
                    .map(|(name, count)| {
                        crate::plugin::plugin_world::exports::rustpress::system::hooks::CategoryInfo {
                            name,
                            count,
                        }
                    })
                    .collect(),
                Err(e) => {
                    tracing::error!("[{}] Failed to get category stats: {}", self.plugin_id, e);
                    vec![]
                }
            }
        } else {
            tracing::warn!(
                "[{}] No post repository available for category listing",
                self.plugin_id
            );
            vec![]
        }
    }

    /// Load a plugin from WASM file
    pub async fn load_plugin(
        &self,
        plugin_id: &str,
        _wasm_path: &Path,
        hooks: Vec<String>,
    ) -> Result<LoadedPlugin, Box<dyn std::error::Error + Send + Sync>> {
        tracing::info!("🔄 Loading plugin '{}' (simulated)", plugin_id);

        // TODO: Implement actual WASM loading
        // For now, just simulate loading
        tracing::info!(
            "✅ Successfully loaded plugin '{}' with hooks: {:?}",
            plugin_id,
            hooks
        );

        Ok(LoadedPlugin {
            plugin_id: plugin_id.to_string(),
            registered_hooks: hooks,
        })
    }

    /// Execute a filter hook
    pub async fn execute_filter_hook(
        &self,
        plugin: &LoadedPlugin,
        hook_name: &str,
        mut data: serde_json::Value,
    ) -> Result<serde_json::Value, Box<dyn std::error::Error + Send + Sync>> {
        match hook_name {
            "filter_post_published" => self.execute_filter_post_published(plugin, data).await,
            _ => {
                tracing::warn!("⚠️ Unknown filter hook: {}", hook_name);
                Ok(data)
            }
        }
    }

    /// Execute action hook
    pub async fn execute_action_hook(
        &self,
        plugin: &LoadedPlugin,
        hook_name: &str,
        data: serde_json::Value,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        match hook_name {
            "action_post_published" => self.execute_action_post_published(plugin, data).await,
            _ => {
                tracing::warn!("⚠️ Unknown action hook: {}", hook_name);
                Ok(())
            }
        }
    }

    /// Execute filter_post_published hook
    async fn execute_filter_post_published(
        &self,
        plugin: &LoadedPlugin,
        mut data: serde_json::Value,
    ) -> Result<serde_json::Value, Box<dyn std::error::Error + Send + Sync>> {
        tracing::info!(
            "🔄 Executing filter_post_published for plugin '{}'",
            plugin.plugin_id
        );
        tracing::debug!(
            "📝 Original post content: {}",
            data["content"].as_str().unwrap_or("")
        );

        // Simulate plugin execution - modify post content
        if let Some(content) = data["content"].as_str() {
            // Add poetry to the beginning of the post content
            let poetry_line = "> *Shall I compare thee to a summer's day?*\n\n";
            let new_content = format!("{}{}", poetry_line, content);
            data["content"] = serde_json::Value::String(new_content);
            tracing::debug!("✨ Added poetry to post content");
        }

        tracing::debug!(
            "📝 Modified post content: {}",
            data["content"].as_str().unwrap_or("")
        );
        tracing::info!(
            "✅ Plugin '{}' successfully processed post",
            plugin.plugin_id
        );
        Ok(data)
    }

    /// Execute action_post_published hook
    async fn execute_action_post_published(
        &self,
        plugin: &LoadedPlugin,
        _data: serde_json::Value,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        tracing::info!(
            "🔄 Executing action_post_published for plugin '{}'",
            plugin.plugin_id
        );
        // Action hooks typically don't return data, just perform side effects
        tracing::info!("✅ Plugin '{}' completed action", plugin.plugin_id);
        Ok(())
    }
}
