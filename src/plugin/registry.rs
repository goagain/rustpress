//! Plugin Registry - Manages loaded plugins and their hook mappings

use crate::plugin::executor::{LoadedPlugin, PluginExecutor};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

/// Plugin registry that manages loaded plugins and their hook mappings
#[derive(Clone)]
pub struct PluginRegistry {
    executors: Arc<RwLock<HashMap<String, PluginExecutor>>>,
    plugins: Arc<RwLock<HashMap<String, LoadedPlugin>>>,
    hook_to_plugins: Arc<RwLock<HashMap<String, Vec<String>>>>, // hook_name -> plugin_ids
}

impl PluginRegistry {
    /// Create a new plugin registry
    pub fn new() -> Self {
        Self {
            executors: Arc::new(RwLock::new(HashMap::new())),
            plugins: Arc::new(RwLock::new(HashMap::new())),
            hook_to_plugins: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Get executor for a specific plugin
    pub async fn get_executor(&self, plugin_id: &str) -> Option<PluginExecutor> {
        let executors = self.executors.read().await;
        executors.get(plugin_id).cloned()
    }

    /// Load a plugin from WASM file
    pub async fn load_plugin(
        &self,
        plugin_id: &str,
        wasm_path: &std::path::Path,
        hooks: Vec<String>,
    ) -> Result<LoadedPlugin, Box<dyn std::error::Error + Send + Sync>> {
        // Create executor for this plugin
        let executor = PluginExecutor::new(plugin_id.to_string());

        // Load the plugin using the executor
        executor.load_plugin(plugin_id, wasm_path, hooks).await
    }

    /// Register a plugin with the registry
    pub async fn register_plugin(
        &self,
        plugin: LoadedPlugin,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let plugin_id = plugin.plugin_id.clone();

        // Create executor for this plugin
        let executor = PluginExecutor::new(plugin_id.clone());
        {
            let mut executors = self.executors.write().await;
            executors.insert(plugin_id.clone(), executor);
        }

        // Add plugin to registry
        {
            let mut plugins = self.plugins.write().await;
            plugins.insert(plugin_id.clone(), plugin.clone());
        }

        // Register hooks
        {
            let mut hook_to_plugins = self.hook_to_plugins.write().await;
            for hook_name in &plugin.registered_hooks {
                hook_to_plugins
                    .entry(hook_name.clone())
                    .or_insert_with(Vec::new)
                    .push(plugin_id.clone());
            }
        }

        tracing::info!(
            "✅ Registered plugin '{}' with hooks: {:?}",
            plugin_id,
            plugin.registered_hooks
        );
        Ok(())
    }

    /// Unregister a plugin from the registry
    pub async fn unregister_plugin(
        &self,
        plugin_id: &str,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        // Remove from plugins
        let plugin = {
            let mut plugins = self.plugins.write().await;
            plugins.remove(plugin_id)
        };

        if let Some(plugin) = plugin {
            // Remove from hook mappings
            let mut hook_to_plugins = self.hook_to_plugins.write().await;
            for hook_name in &plugin.registered_hooks {
                if let Some(plugins) = hook_to_plugins.get_mut(hook_name) {
                    plugins.retain(|id| id != plugin_id);
                    // Remove empty hook entries
                    if plugins.is_empty() {
                        hook_to_plugins.remove(hook_name);
                    }
                }
            }

            tracing::info!("✅ Unregistered plugin '{}'", plugin_id);
            Ok(())
        } else {
            Err(format!("Plugin '{}' not found", plugin_id).into())
        }
    }

    /// Get all plugins registered for a specific hook
    pub async fn get_plugins_for_hook(&self, hook_name: &str) -> Vec<LoadedPlugin> {
        let hook_to_plugins = self.hook_to_plugins.read().await;
        let plugins = self.plugins.read().await;

        if let Some(plugin_ids) = hook_to_plugins.get(hook_name) {
            plugin_ids
                .iter()
                .filter_map(|plugin_id| plugins.get(plugin_id).cloned())
                .collect()
        } else {
            Vec::new()
        }
    }

    /// Execute a filter hook across all registered plugins
    pub async fn execute_filter_hook(
        &self,
        hook_name: &str,
        mut data: serde_json::Value,
    ) -> Result<serde_json::Value, Box<dyn std::error::Error + Send + Sync>> {
        let plugins = self.get_plugins_for_hook(hook_name).await;

        if plugins.is_empty() {
            return Ok(data);
        }

        tracing::info!(
            "🔄 Executing filter hook '{}' on {} plugins",
            hook_name,
            plugins.len()
        );

        for plugin in plugins {
            if let Some(executor) = self.get_executor(&plugin.plugin_id).await {
                data = executor.execute_filter_hook(&plugin, hook_name, data).await?;
            }
        }

        Ok(data)
    }

    /// Execute an action hook across all registered plugins
    pub async fn execute_action_hook(
        &self,
        hook_name: &str,
        data: serde_json::Value,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let plugins = self.get_plugins_for_hook(hook_name).await;

        if plugins.is_empty() {
            return Ok(());
        }

        tracing::info!(
            "🔄 Executing action hook '{}' on {} plugins",
            hook_name,
            plugins.len()
        );

        for plugin in plugins {
            if let Some(executor) = self.get_executor(&plugin.plugin_id).await {
                executor.execute_action_hook(&plugin, hook_name, data.clone()).await?;
            }
        }

        Ok(())
    }

    /// Get all registered plugins
    pub async fn get_all_plugins(&self) -> HashMap<String, LoadedPlugin> {
        self.plugins.read().await.clone()
    }

    /// Check if a plugin is registered
    pub async fn is_plugin_registered(&self, plugin_id: &str) -> bool {
        self.plugins.read().await.contains_key(plugin_id)
    }

    /// Get plugin by ID
    pub async fn get_plugin(&self, plugin_id: &str) -> Option<LoadedPlugin> {
        self.plugins.read().await.get(plugin_id).cloned()
    }
}
