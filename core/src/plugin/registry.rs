//! Plugin Registry - Manages loaded plugins and their hook mappings

use crate::plugin::engine::PluginEngine;
use crate::plugin::exports::rustpress::plugin::post_hooks::OnPostPublishedData;
use crate::plugin::loaded_plugin::LoadedPlugin;
use sea_orm::{ActiveModelTrait, ColumnTrait, EntityTrait, QueryFilter, Set};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

/// Plugin registry that manages loaded plugins and their hook mappings
#[derive(Clone)]
pub struct PluginRegistry {
    plugins: Arc<RwLock<HashMap<(String, String), LoadedPlugin>>>,
    hook_to_plugins: Arc<RwLock<HashMap<String, Vec<(String, String)>>>>,
    engine: Arc<PluginEngine>,
    db: Arc<sea_orm::DatabaseConnection>,
}

impl PluginRegistry {
    /// Create a new plugin registry
    pub fn new(engine: Arc<PluginEngine>, db: Arc<sea_orm::DatabaseConnection>) -> Self {
        Self {
            engine,
            db,
            plugins: Arc::new(RwLock::new(HashMap::new())),
            hook_to_plugins: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Load a plugin from WASM file
    pub async fn load_plugin_async(
        &self,
        plugin_id: &str,
        version: &str,
        hooks: Vec<String>,
    ) -> Result<LoadedPlugin, Box<dyn std::error::Error + Send + Sync>> {
        // Load the plugin using the engine
        let plugin_path = format!("plugins/{}/{}.rpk", &plugin_id, &version);

        // 'rpk' is a zip format. We need to unzip it to '.cache/plugin_id-version', then use the `.wasm` file(s).
        use std::fs;
        use std::path::{Path, PathBuf};
        use tokio::task;

        let extract_dir = format!(".cache/{}/{}", plugin_id, version);

        // Ensure the extract directory exists (create if not)
        fs::create_dir_all(&extract_dir)?;

        // Unzip the .rpk file to the extract directory
        // Do the actual zip extraction in a blocking task
        let plugin_path_cloned = plugin_path.clone();
        let extract_dir_cloned = extract_dir.clone();
        task::spawn_blocking(move || {
            let file = fs::File::open(&plugin_path_cloned)?;
            let mut archive = zip::ZipArchive::new(file)?;
            for i in 0..archive.len() {
                let mut file_in_zip = archive.by_index(i)?;
                let out_path = Path::new(&extract_dir_cloned).join(file_in_zip.name());
                if file_in_zip.name().ends_with('/') {
                    fs::create_dir_all(&out_path)?;
                } else {
                    if let Some(p) = out_path.parent() {
                        if !p.exists() {
                            fs::create_dir_all(&p)?;
                        }
                    }
                    let mut out_file = fs::File::create(&out_path)?;
                    std::io::copy(&mut file_in_zip, &mut out_file)?;
                }
            }
            Ok::<_, Box<dyn std::error::Error + Send + Sync>>(())
        })
        .await??;

        // Now locate the .wasm file inside the extracted directory
        // Always use the file 'plugin.wasm' in the root of the extracted directory
        let plugin_wasm_path = std::path::Path::new(&extract_dir).join("plugin.wasm");
        if !plugin_wasm_path.exists() {
            return Err(format!(
                "plugin.wasm not found in root of extracted plugin for '{}'",
                plugin_id
            )
            .into());
        }

        let plugin = self
            .engine
            .load_plugin_async(&plugin_id, &version, &plugin_wasm_path, hooks)
            .await?;
        self.register_plugin(&plugin).await?;
        Ok(plugin)
    }

    /// Register a plugin with the registry
    pub async fn register_plugin(
        &self,
        plugin: &LoadedPlugin,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let plugin_id = plugin.plugin_id.clone();

        // Add plugin to registry
        {
            let mut plugins = self.plugins.write().await;
            plugins.insert(
                (plugin.plugin_id.clone(), plugin.version.clone()),
                plugin.clone(),
            );
        }

        // Register hooks
        {
            let mut hook_to_plugins = self.hook_to_plugins.write().await;
            for hook_name in &plugin.registered_hooks {
                hook_to_plugins
                    .entry(hook_name.clone())
                    .or_insert_with(Vec::new)
                    .push((plugin.plugin_id.clone(), plugin.version.clone()));
            }
        }

        tracing::info!(
            "✅ Registered plugin '{}' with hooks: {:?}",
            format!("{}-{}", plugin_id, plugin.version),
            plugin.registered_hooks
        );
        Ok(())
    }

    pub async fn unregister_plugin(
        &self,
        plugin_id: &str,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        self.plugins
            .write()
            .await
            .iter()
            .filter(|(id, _)| id.0 == plugin_id)
            .for_each(|(_, plugin)| {
                self.unregister_plugin_with_version(&plugin.plugin_id, &plugin.version);
            });
        Ok(())
    }
    /// Unregister a plugin from the registry
    pub async fn unregister_plugin_with_version(
        &self,
        plugin_id: &str,
        version: &str,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        // Remove from plugins
        let plugin = {
            let mut plugins = self.plugins.write().await;
            plugins.remove(&(plugin_id.to_string(), version.to_string()))
        };

        if let Some(plugin) = plugin {
            // Remove from hook mappings
            let mut hook_to_plugins = self.hook_to_plugins.write().await;
            for hook_name in &plugin.registered_hooks {
                if let Some(plugins) = hook_to_plugins.get_mut(hook_name) {
                    plugins.retain(|id_with_version| {
                        id_with_version != &(plugin_id.to_string(), version.to_string())
                    });
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
        let plugins: tokio::sync::RwLockReadGuard<'_, HashMap<(String, String), LoadedPlugin>> =
            self.plugins.read().await;

        if let Some(plugin_with_version) = hook_to_plugins.get(hook_name) {
            plugin_with_version
                .iter()
                .filter_map(|plugin_with_version| plugins.get(plugin_with_version).cloned())
                .collect()
        } else {
            Vec::new()
        }
    }

    /// Get all registered plugins
    pub async fn get_all_plugins(&self) -> HashMap<(String, String), LoadedPlugin> {
        self.plugins.read().await.clone()
    }

    /// Check if a plugin is registered
    pub async fn is_plugin_registered(&self, plugin_id: &str, version: &str) -> bool {
        self.plugins
            .read()
            .await
            .contains_key(&(plugin_id.to_string(), version.to_string()))
    }

    /// Get plugin by ID
    pub async fn get_plugin(&self, plugin_id: &str, version: &str) -> Option<LoadedPlugin> {
        self.plugins
            .read()
            .await
            .get(&(plugin_id.to_string(), version.to_string()))
            .cloned()
    }

    /// Load all enabled plugins from database
    pub async fn load_enabled_plugins(
        &self,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        // TODO: Implement loading enabled plugins from database
        // For now, this is a placeholder
        tracing::info!("Loading enabled plugins from database");
        Ok(())
    }

    /// Install a plugin from RPK data
    pub async fn install_plugin(
        &self,
        rpk_data: &[u8],
        permission_grants: &std::collections::HashMap<String, bool>,
    ) -> Result<
        (serde_json::Value, crate::dto::plugin::PluginUpdateAnalysis),
        Box<dyn std::error::Error + Send + Sync>,
    > {
        use sea_orm::{ActiveModelTrait, Set};

        // Create temporary directory for extraction
        let temp_dir = std::env::temp_dir().join(format!(
            "rustpress_plugin_install_{}",
            chrono::Utc::now().timestamp()
        ));
        std::fs::create_dir_all(&temp_dir)?;

        // Extract and validate RPK
        let mut rpk_processor =
            crate::rpk::RpkProcessor::new(temp_dir.clone(), temp_dir.join("cache"));
        rpk_processor.init()?;
        let package = rpk_processor
            .extract_and_validate(&std::path::Path::new("temp.rpk"), None)
            .await?;
        let plugin_id = &package.manifest.package.id;

        // Check if plugin already exists
        let existing_plugin = crate::entity::plugins::Entity::find()
            .filter(crate::entity::plugins::Column::Name.eq(plugin_id))
            .one(&*self.db)
            .await?;

        if existing_plugin.is_some() {
            return Err("Plugin already exists".into());
        }

        // Save RPK file
        let rpk_path = temp_dir.join(format!("{}.rpk", plugin_id));
        std::fs::write(&rpk_path, rpk_data)?;

        // Cache package files
        rpk_processor
            .cache_package_files(&package, &temp_dir.join("cache"))
            .await?;

        // Create plugin record in database
        let manifest_json = serde_json::to_value(&package.manifest)?;
        let version = package.manifest.package.version.clone();
        let plugin_model = crate::entity::plugins::ActiveModel {
            name: Set(plugin_id.clone()),
            description: Set(package.manifest.package.description),
            version: Set(version.clone()),
            enabled: Set(false),
            status: Set("disabled".to_string()),
            config: Set(None),
            manifest: Set(Some(manifest_json.clone())),
            created_at: Set(chrono::Utc::now().into()),
            updated_at: Set(chrono::Utc::now().into()),
            ..Default::default()
        };

        plugin_model.insert(&*self.db).await?;

        // Analyze update (new installation)
        let analysis = crate::dto::plugin::PluginUpdateAnalysis {
            plugin_id: plugin_id.clone(),
            current_version: "0.0.0".to_string(),
            new_version: version,
            status: crate::dto::plugin::PluginUpdateStatus::NeedsReview,
            new_required_permissions: package.manifest.permissions.required,
            new_optional_permissions: Vec::new(), // Optional permissions not used in new format
            message: "New plugin installation requires permission review".to_string(),
        };

        // Cleanup
        let _ = std::fs::remove_dir_all(&temp_dir);

        Ok((manifest_json, analysis))
    }

    /// Uninstall a plugin
    pub async fn uninstall_plugin(
        &self,
        plugin_name: &str,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        // Remove from registry first
        self.unregister_plugin(plugin_name).await?;

        // Remove from database
        crate::entity::plugins::Entity::delete_many()
            .filter(crate::entity::plugins::Column::Name.eq(plugin_name))
            .exec(&*self.db)
            .await?;

        Ok(())
    }

    /// Get current permissions for a plugin
    pub async fn get_plugin_permissions(
        &self,
        plugin_id: &str,
    ) -> Result<std::collections::HashMap<String, bool>, Box<dyn std::error::Error + Send + Sync>>
    {
        // For now, return all permissions as granted
        // TODO: Implement proper permission tracking
        let mut permissions = std::collections::HashMap::new();
        permissions.insert("post:read".to_string(), true);
        permissions.insert("post:write".to_string(), true);
        permissions.insert("ai:chat".to_string(), true);
        Ok(permissions)
    }

    /// Update plugin permissions
    pub async fn update_plugin_permissions(
        &self,
        plugin_id: &str,
        permissions: &std::collections::HashMap<String, bool>,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        // TODO: Implement permission persistence
        tracing::info!(
            "Updated permissions for plugin {}: {:?}",
            plugin_id,
            permissions
        );
        Ok(())
    }

    /// Analyze permissions when enabling a plugin
    pub async fn analyze_enable_permissions(
        &self,
        plugin_name: &str,
    ) -> Result<std::collections::HashMap<String, String>, Box<dyn std::error::Error + Send + Sync>>
    {
        // Get plugin from database
        let plugin = crate::entity::plugins::Entity::find()
            .filter(crate::entity::plugins::Column::Name.eq(plugin_name))
            .one(&*self.db)
            .await?
            .ok_or("Plugin not found")?;

        let mut new_permissions = std::collections::HashMap::new();

        if let Some(manifest_value) = plugin.manifest {
            // Extract required permissions
            if let Some(required_perms) = manifest_value
                .get("permissions")
                .and_then(|p| p.get("required"))
                .and_then(|r| r.as_array())
            {
                for perm in required_perms {
                    if let Some(perm_str) = perm.as_str() {
                        new_permissions.insert(perm_str.to_string(), "required".to_string());
                    }
                }
            }
        }

        Ok(new_permissions)
    }
}

pub struct PluginExecuter {
    registry: Arc<PluginRegistry>,
    ai_client: Option<Arc<super::ai::AiHelper>>,
    db: Arc<sea_orm::DatabaseConnection>,
}

impl PluginExecuter {
    pub fn new(
        registry: Arc<PluginRegistry>,
        ai_client: Option<Arc<super::ai::AiHelper>>,
        db: Arc<sea_orm::DatabaseConnection>,
    ) -> Self {
        Self {
            registry,
            ai_client,
            db,
        }
    }

    fn new_state(&self, plugin: &LoadedPlugin) -> anyhow::Result<super::PluginHostState> {
        Ok(super::PluginHostState::new(
            plugin.plugin_id.clone(),
            plugin.granted_permissions.clone(),
        ))
    }

    pub async fn get_bindings(
        &self,
        plugin: &LoadedPlugin,
    ) -> anyhow::Result<(wasmtime::Store<super::PluginHostState>, super::PluginWorld)> {
        let engine = self.registry.engine.get_engine();
        let linker = self.registry.engine.get_linker();
        let state = self.new_state(plugin)?;
        let component = plugin.component.as_ref().unwrap();
        let mut store = wasmtime::Store::new(engine, state);
        let (bindings, _) =
            super::PluginWorld::instantiate_async(&mut store, component, &linker).await?;

        Ok((store, bindings))
    }
    pub async fn post_published_filter(
        &self,
        data: crate::dto::post::CreatePostRequest,
    ) -> anyhow::Result<crate::dto::post::CreatePostRequest> {
        let plugins = self
            .registry
            .get_plugins_for_hook("post_published_filter")
            .await;

        let mut modified_data: OnPostPublishedData = data.into();

        for plugin in plugins {
            let (store, bindings) = self.get_bindings(&plugin).await?;
            modified_data = match bindings
                .rustpress_plugin_post_hooks()
                .call_on_post_published(store, &modified_data)
                .await
            {
                Ok(new_result) => match new_result {
                    Ok(new_result) => new_result,
                    Err(e) => {
                        tracing::error!(
                            "Error calling on_post_published_filter for plugin {}: {:?}",
                            plugin.plugin_id,
                            e
                        );
                        modified_data
                    }
                },
                Err(e) => {
                    return Err(e);
                }
            }
        }
        Ok(modified_data.into())
    }
}
