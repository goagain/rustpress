//! Plugin system implementation

use crate::dto::plugin::{PluginExecutionContext, PluginExecutionResult};
use sea_orm::{
    ActiveModelTrait, ColumnTrait, DatabaseConnection, DeleteResult, EntityTrait, QueryFilter,
};
use std::sync::Arc;

/// Plugin manager that handles plugin lifecycle and execution
#[derive(Clone)]
pub struct PluginManager {
    /// Database connection
    db: Arc<DatabaseConnection>,
}

impl PluginManager {
    /// Create a new plugin manager
    pub fn new(db: Arc<DatabaseConnection>) -> Self {
        Self { db }
    }

    /// Load all enabled plugins from database
    pub async fn load_enabled_plugins(
        &self,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        use crate::entity::plugins;

        println!("🔄 Loading enabled plugins...");

        // Query enabled plugins from database
        let enabled_plugins = plugins::Entity::find()
            .filter(plugins::Column::Enabled.eq(true))
            .filter(plugins::Column::Status.eq("enabled"))
            .all(&*self.db)
            .await
            .map_err(|e| format!("Failed to query plugins: {}", e))?;

        println!("📦 Found {} enabled plugins", enabled_plugins.len());

        for plugin in enabled_plugins {
            println!("✅ Loaded plugin: {} (ID: {})", plugin.name, plugin.id);

            // TODO: Load WASM modules and register hooks
            // For now, just log that the plugin would be loaded
        }

        println!("✅ Plugin system initialized");
        Ok(())
    }

    /// Execute a hook for a specific plugin
    pub async fn execute_plugin_hook(
        &self,
        _plugin_id: &str,
        _context: PluginExecutionContext,
    ) -> PluginExecutionResult {
        // Mock implementation
        PluginExecutionResult {
            success: true,
            data: Some(serde_json::json!({
                "content": "> *Shall I compare thee to a summer's day?*\n\nPlugin executed successfully!"
            })),
            error: None,
        }
    }

    /// Execute a filter hook
    pub async fn execute_filter_hook(
        &self,
        _hook: crate::dto::plugin::PluginHook,
        data: serde_json::Value,
    ) -> (serde_json::Value, Vec<PluginExecutionResult>) {
        // Mock implementation
        (
            data,
            vec![PluginExecutionResult {
                success: true,
                data: Some(serde_json::json!({
                    "content": "> *Shall I compare thee to a summer's day?*\n\nPlugin executed successfully!"
                })),
                error: None,
            }],
        )
    }

    /// Execute an action hook
    pub async fn execute_action_hook(
        &self,
        _hook: crate::dto::plugin::PluginHook,
        _data: serde_json::Value,
    ) -> Vec<PluginExecutionResult> {
        // Mock implementation
        vec![PluginExecutionResult {
            success: true,
            data: Some(serde_json::json!({
                "content": "> *Shall I compare thee to a summer's day?*\n\nPlugin executed successfully!"
            })),
            error: None,
        }]
    }

    /// Install a plugin
    pub async fn install_plugin(
        &self,
        rpk_data: &[u8],
        permission_grants: &std::collections::HashMap<String, bool>,
    ) -> Result<(serde_json::Value, serde_json::Value), Box<dyn std::error::Error + Send + Sync>>
    {
        use crate::entity::plugins;

        // 1. Parse RPK data
        let rpk_processor = crate::rpk::RpkProcessor::new(
            std::path::PathBuf::from("plugins"),
            std::path::PathBuf::from("plugin_cache"),
        );
        rpk_processor
            .init()
            .map_err(|e| format!("Failed to initialize RPK processor: {}", e))?;

        // Extract plugin information (we need to parse the manifest)
        let temp_dir =
            tempfile::tempdir().map_err(|e| format!("Failed to create temp dir: {}", e))?;
        let temp_rpk_path = temp_dir.path().join("temp.rpk");
        std::fs::write(&temp_rpk_path, rpk_data)
            .map_err(|e| format!("Failed to write temp RPK: {}", e))?;

        let package = rpk_processor
            .extract_and_validate(&temp_rpk_path, None)
            .await
            .map_err(|e| format!("Failed to validate RPK: {}", e))?;

        // 2. Extract plugin information from manifest
        let plugin_manifest = &package.manifest;
        let plugin_id = &plugin_manifest.package.id;
        let name = &plugin_manifest.package.name;
        let version = &plugin_manifest.package.version;
        let description = plugin_manifest.package.description.clone();

        // 3. Check if plugin already exists
        let existing_plugin = plugins::Entity::find()
            .filter(plugins::Column::Name.eq(plugin_id))
            .one(&*self.db)
            .await
            .map_err(|e| format!("Database error: {}", e))?;

        if existing_plugin.is_some() {
            return Err("Plugin already exists".into());
        }

        // 4. Serialize manifest for database storage and return
        let manifest_json = serde_json::to_value(plugin_manifest)
            .map_err(|e| format!("Failed to serialize manifest: {}", e))?;

        // 5. Create new plugin record
        let now =
            chrono::Utc::now().with_timezone(&chrono::FixedOffset::east_opt(8 * 3600).unwrap());
        let new_plugin = plugins::ActiveModel {
            name: sea_orm::ActiveValue::Set(plugin_id.to_string()),
            description: sea_orm::ActiveValue::Set(description),
            version: sea_orm::ActiveValue::Set(version.to_string()),
            enabled: sea_orm::ActiveValue::Set(false), // Default to disabled
            config: sea_orm::ActiveValue::Set(Some(serde_json::json!({}))),
            manifest: sea_orm::ActiveValue::Set(Some(manifest_json.clone())),
            status: sea_orm::ActiveValue::Set("disabled".to_string()),
            created_at: sea_orm::ActiveValue::Set(now),
            updated_at: sea_orm::ActiveValue::Set(now),
            ..Default::default()
        };

        // 5. Insert into database
        let plugin = new_plugin
            .insert(&*self.db)
            .await
            .map_err(|e| format!("Failed to insert plugin: {}", e))?;

        tracing::info!(
            "Plugin '{}' installed successfully with ID {}",
            plugin_id,
            plugin.id
        );

        // 6. Create update analysis (for new installation)
        let update_analysis = serde_json::json!({
            "type": "new_installation",
            "plugin_id": plugin_id,
            "version": version
        });

        Ok((manifest_json, update_analysis))
    }

    /// Uninstall a plugin
    pub async fn uninstall_plugin(
        &self,
        plugin_id: &str,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        use crate::entity::{plugin_permissions, plugins};

        tracing::info!(
            "PluginManager::uninstall_plugin called with plugin_id: {}",
            plugin_id
        );

        // 1. Delete plugin permissions first (foreign key constraint)
        let delete_permissions_result = plugin_permissions::Entity::delete_many()
            .filter(plugin_permissions::Column::PluginId.eq(plugin_id))
            .exec(&*self.db)
            .await;

        if let Err(e) = delete_permissions_result {
            tracing::error!(
                "Failed to delete plugin permissions for {}: {}",
                plugin_id,
                e
            );
            return Err(format!("Failed to delete plugin permissions: {}", e).into());
        }

        // 2. Delete the plugin record
        let delete_plugin_result = plugins::Entity::delete_many()
            .filter(plugins::Column::Name.eq(plugin_id))
            .exec(&*self.db)
            .await;

        match delete_plugin_result {
            Ok(delete_result) => {
                if delete_result.rows_affected == 0 {
                    tracing::warn!("Plugin '{}' not found for uninstallation", plugin_id);
                    return Err(format!("Plugin '{}' not found", plugin_id).into());
                }
                tracing::info!("Successfully deleted plugin '{}' from database", plugin_id);
            }
            Err(e) => {
                tracing::error!(
                    "Failed to delete plugin '{}' from database: {}",
                    plugin_id,
                    e
                );
                return Err(format!("Failed to delete plugin from database: {}", e).into());
            }
        }

        // 3. TODO: Clean up cache files and unload from memory
        // For now, we don't have cache file management implemented yet

        tracing::info!("Plugin '{}' uninstalled successfully", plugin_id);
        Ok(())
    }

    /// Get plugin permissions
    pub async fn get_plugin_permissions(
        &self,
        _plugin_id: &str,
    ) -> Result<std::collections::HashMap<String, bool>, Box<dyn std::error::Error + Send + Sync>>
    {
        Ok(std::collections::HashMap::new())
    }

    /// Update plugin permissions
    pub async fn update_plugin_permissions(
        &self,
        _plugin_id: &str,
        _permissions: &std::collections::HashMap<String, bool>,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        Ok(())
    }

    /// Analyze enable permissions
    pub async fn analyze_enable_permissions(
        &self,
        _plugin_id: &str,
    ) -> Result<std::collections::HashMap<String, String>, Box<dyn std::error::Error + Send + Sync>>
    {
        Ok(std::collections::HashMap::new())
    }
}

/// Plugin instance in memory
#[derive(Clone, Debug)]
pub struct PluginInstance {
    /// Plugin ID
    pub id: String,
}

/// Plugin host API
#[derive(Clone)]
pub struct PluginHostApi {
    plugin_manager: Arc<PluginManager>,
    plugin_id: String,
}

impl PluginHostApi {
    pub fn new(plugin_manager: Arc<PluginManager>, plugin_id: String) -> Self {
        Self {
            plugin_manager,
            plugin_id,
        }
    }

    pub fn log_error(&self, message: &str) {
        println!("[{}] ERROR: {}", self.plugin_id, message);
    }

    pub fn log_warn(&self, message: &str) {
        println!("[{}] WARN: {}", self.plugin_id, message);
    }

    pub fn log_info(&self, message: &str) {
        println!("[{}] INFO: {}", self.plugin_id, message);
    }

    pub fn log_debug(&self, message: &str) {
        println!("[{}] DEBUG: {}", self.plugin_id, message);
    }
}
