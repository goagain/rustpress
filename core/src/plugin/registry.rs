//! Plugin Registry - Manages loaded plugins and their hook mappings

use crate::ai::AiService;
use crate::plugin::engine::PluginEngine;
use crate::plugin::exports::rustpress::plugin::event_handler::{
    OnPostPublishedData, PluginActionEvent, PluginFilterEvent,
};
use crate::plugin::loaded_plugin::LoadedPlugin;
use sea_orm::{ActiveModelTrait, ColumnTrait, EntityTrait, QueryFilter, Set};
use std::collections::{HashMap, HashSet};
use std::sync::Arc;
use tokio::sync::RwLock;

/// Plugin registry that manages loaded plugins and their hook mappings
#[derive(Clone)]
pub struct PluginRegistry {
    plugins: Arc<RwLock<HashMap<(String, String), LoadedPlugin>>>,
    hook_to_plugins: Arc<RwLock<HashMap<String, Vec<(String, String)>>>>,
    engine: Arc<PluginEngine>,
    db: Arc<sea_orm::DatabaseConnection>,
    rpk_processor: Arc<crate::rpk::RpkProcessor>,
    ai_service: Option<Arc<AiService>>,
}

impl PluginRegistry {
    /// Create a new plugin registry
    pub fn new(
        engine: Arc<PluginEngine>,
        db: Arc<sea_orm::DatabaseConnection>,
        rpk_processor: Arc<crate::rpk::RpkProcessor>,
        ai_service: Option<Arc<AiService>>,
    ) -> Self {
        Self {
            engine,
            db,
            plugins: Arc::new(RwLock::new(HashMap::new())),
            hook_to_plugins: Arc::new(RwLock::new(HashMap::new())),
            rpk_processor,
            ai_service,
        }
    }

    /// Load a plugin from RPK data
    pub async fn load_plugin_from_rpk_data(
        &self,
        rpk_data: &[u8],
        plugin_model: &crate::entity::plugins::Model,
    ) -> Result<LoadedPlugin, Box<dyn std::error::Error + Send + Sync>> {
        use std::fs;
        use tokio::task;

        // Create a temporary directory for extraction
        let temp_id = format!("temp_{}", chrono::Utc::now().timestamp());
        let extract_dir = self.rpk_processor.cache_dir().join(&temp_id);

        // Clone data for the blocking task
        let rpk_data_clone = rpk_data.to_vec();
        let extract_dir_clone = extract_dir.clone();

        // Extract the RPK (zip) data to the temporary directory
        task::spawn_blocking(move || {
            fs::create_dir_all(&extract_dir_clone)?;
            let cursor = std::io::Cursor::new(rpk_data_clone);
            let mut archive = zip::ZipArchive::new(cursor)?;
            for i in 0..archive.len() {
                let mut file_in_zip = archive.by_index(i)?;
                let out_path = extract_dir_clone.join(file_in_zip.name());
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

        // Read the manifest to get plugin info
        let manifest_path = extract_dir.join("manifest.toml");
        let manifest_content = fs::read_to_string(&manifest_path)?;
        let manifest: crate::dto::plugin::PluginManifest = toml::from_str(&manifest_content)?;

        // Locate the .wasm file
        let plugin_wasm_path = extract_dir.join("plugin.wasm");
        if !plugin_wasm_path.exists() {
            let _ = fs::remove_dir_all(&extract_dir);
            return Err("plugin.wasm not found in RPK package".into());
        }

        // Load the WASM plugin
        let plugin_data = fs::read(&plugin_wasm_path)?;
        let component =
            wasmtime::component::Component::from_binary(self.engine.get_engine(), &plugin_data)?;

        // Get registered hooks
        let registered_hooks = manifest.plugin.hooks.clone();

        // Get granted permissions (for now, include all required permissions)
        // TODO: Implement proper permission granting based on user approval
        let mut granted_permissions = std::collections::HashSet::new();

        // Add required permissions from manifest (for plugins without stored permissions)
        // If plugin has stored granted_permissions in database, use those instead
        if let Some(stored_permissions) = &plugin_model.granted_permissions {
            if let Ok(perms) = serde_json::from_value::<Vec<String>>(stored_permissions.clone()) {
                granted_permissions.extend(perms);
            } else {
                // Fallback to manifest permissions if stored permissions are invalid
                granted_permissions.extend(manifest.permissions.required.iter().cloned());
            }
        } else {
            // No stored permissions, use manifest permissions
            granted_permissions.extend(manifest.permissions.required.iter().cloned());
        }

        // Create LoadedPlugin
        let loaded_plugin = LoadedPlugin {
            plugin_id: manifest.package.id.clone(),
            version: manifest.package.version.clone(),
            registered_hooks: registered_hooks.clone(),
            component: Some(component),
            granted_permissions,
        };

        // Cleanup temporary directory
        let _ = fs::remove_dir_all(&extract_dir);

        Ok(loaded_plugin)
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
        let plugins_to_unregister: Vec<_> = self
            .plugins
            .write()
            .await
            .iter()
            .filter(|(id, _)| id.0 == plugin_id)
            .map(|(_, plugin)| (plugin.plugin_id.clone(), plugin.version.clone()))
            .collect();

        for (plugin_id, version) in plugins_to_unregister {
            self.unregister_plugin_with_version(&plugin_id, &version)
                .await?;
        }
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
        // Load all enabled plugins from database
        let enabled_plugins = crate::entity::plugins::Entity::find()
            .filter(crate::entity::plugins::Column::Enabled.eq(true))
            .filter(crate::entity::plugins::Column::Status.eq("enabled"))
            .all(&*self.db)
            .await?;

        tracing::info!(
            "Found {} enabled plugins in database",
            enabled_plugins.len()
        );

        for plugin_model in enabled_plugins {
            tracing::info!(
                "Loading plugin: {} v{}",
                plugin_model.name,
                plugin_model.version
            );
            if let Err(e) = self
                .load_plugin_from_database(&plugin_model.plugin_id, &plugin_model.version)
                .await
            {
                tracing::error!("Failed to load plugin {}: {}", plugin_model.plugin_id, e);

                // Disable the plugin if loading failed
                if let Err(disable_err) = self
                    .disable_plugin(&plugin_model.plugin_id, &plugin_model.version)
                    .await
                {
                    tracing::error!(
                        "Failed to disable broken plugin {}: {}",
                        plugin_model.plugin_id,
                        disable_err
                    );
                } else {
                    tracing::warn!(
                        "Disabled broken plugin: {} v{}",
                        plugin_model.plugin_id,
                        plugin_model.version
                    );
                }

                // Continue loading other plugins even if one fails
            }
        }

        Ok(())
    }

    /// Load a plugin from database into registry
    pub async fn load_plugin_from_database(
        &self,
        plugin_id: &str,
        version: &str,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        // Find plugin in database
        let plugin_model = crate::entity::plugins::Entity::find()
            .filter(crate::entity::plugins::Column::PluginId.eq(plugin_id))
            .filter(crate::entity::plugins::Column::Version.eq(version))
            .one(&*self.db)
            .await?
            .ok_or_else(|| format!("Plugin {}-{} not found in database", plugin_id, version))?;

        // Load plugin from installed directory
        let plugin_path = self
            .rpk_processor
            .install_dir()
            .join(format!("{}-{}.rpk", plugin_id, version));
        if !plugin_path.exists() {
            return Err(format!("Plugin file not found: {:?}", plugin_path).into());
        }

        // Load and instantiate the plugin
        let plugin_data = std::fs::read(&plugin_path)?;
        let loaded_plugin = self
            .load_plugin_from_rpk_data(&plugin_data, &plugin_model)
            .await?;

        // Register the plugin
        self.register_plugin(&loaded_plugin).await?;

        Ok(())
    }

    /// Disable a plugin in the database
    pub async fn disable_plugin(
        &self,
        plugin_id: &str,
        version: &str,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        // Find and update the plugin
        let mut plugin: crate::entity::plugins::ActiveModel =
            crate::entity::plugins::Entity::find()
                .filter(crate::entity::plugins::Column::PluginId.eq(plugin_id))
                .filter(crate::entity::plugins::Column::Version.eq(version))
                .one(&*self.db)
                .await?
                .ok_or_else(|| format!("Plugin {}-{} not found", plugin_id, version))?
                .into();

        plugin.enabled = sea_orm::Set(false);
        plugin.status = sea_orm::Set("disabled".to_string());
        plugin.updated_at = sea_orm::Set(chrono::Utc::now().into());

        plugin.update(&*self.db).await?;

        // Unload from registry if it's currently loaded
        if let Err(e) = self.unload_plugin(plugin_id, version).await {
            tracing::warn!("Failed to unload plugin {} from registry: {}", plugin_id, e);
        }

        Ok(())
    }

    /// Unload a plugin from registry
    pub async fn unload_plugin(
        &self,
        plugin_name: &str,
        version: &str,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        self.unregister_plugin_with_version(plugin_name, version)
            .await
    }

    /// Install a plugin from RPK data
    pub async fn install_plugin(
        &self,
        rpk_data: &[u8],
    ) -> Result<
        (serde_json::Value, crate::dto::plugin::PluginUpdateAnalysis),
        Box<dyn std::error::Error + Send + Sync>,
    > {
        use sea_orm::{ActiveModelTrait, Set};

        // Read the RPK archive and extract the plugin manifest
        let mut cursor = std::io::Cursor::new(rpk_data.to_vec());
        let package = self
            .rpk_processor
            .read_package_from_reader(&mut cursor)
            .await?;

        // Extract plugin_id and version from the package manifest
        let plugin_id = package.manifest.package.id.clone();
        let version = package.manifest.package.version.clone();
        let manifest_plugin_id = package.manifest.package.id.clone();

        // Check if plugin already exists
        let existing_plugin = crate::entity::plugins::Entity::find()
            .filter(crate::entity::plugins::Column::PluginId.eq(&plugin_id))
            .one(&*self.db)
            .await?;

        if existing_plugin.is_some() {
            return Err("Plugin already exists".into());
        }

        // Save RPK file
        self.rpk_processor
            .install_rpk(&plugin_id, &version, rpk_data)
            .await?;

        // Create plugin record in database
        let manifest_json = serde_json::to_value(&package.manifest)?;
        let version = package.manifest.package.version.clone();
        let plugin_model = crate::entity::plugins::ActiveModel {
            name: Set(plugin_id.to_string()),
            plugin_id: Set(manifest_plugin_id),
            description: Set(package.manifest.package.description.clone()),
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
            plugin_id: plugin_id.to_string(),
            current_version: "0.0.0".to_string(),
            new_version: version,
            status: crate::dto::plugin::PluginUpdateStatus::NeedsReview,
            new_required_permissions: package.manifest.permissions.required.clone(),
            new_optional_permissions: Vec::new(), // Optional permissions not used in new format
            message: "New plugin installation requires permission review".to_string(),
        };

        Ok((manifest_json, analysis))
    }

    /// Uninstall a plugin
    pub async fn uninstall_plugin(
        &self,
        plugin_id: &str,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        // Remove from registry first
        self.unregister_plugin(plugin_id).await?;

        // Remove from database
        crate::entity::plugins::Entity::delete_many()
            .filter(crate::entity::plugins::Column::Name.eq(plugin_id))
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
        // Find the plugin
        let plugin = crate::entity::plugins::Entity::find()
            .filter(crate::entity::plugins::Column::Name.eq(plugin_id))
            .one(&*self.db)
            .await?
            .ok_or_else(|| format!("Plugin '{}' not found", plugin_id))?;

        // Convert permissions map to list of granted permissions
        let granted_permissions: Vec<String> = permissions
            .iter()
            .filter_map(|(perm, &granted)| if granted { Some(perm.clone()) } else { None })
            .collect();

        // Update the plugin's granted_permissions field
        let mut plugin_model: crate::entity::plugins::ActiveModel = plugin.into();
        plugin_model.granted_permissions =
            sea_orm::Set(Some(serde_json::json!(granted_permissions)));
        plugin_model.updated_at = sea_orm::Set(chrono::Utc::now().into());

        plugin_model.update(&*self.db).await?;

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

    fn new_state(&self, plugin: &LoadedPlugin) -> anyhow::Result<super::PluginHostState> {
        Ok(super::PluginHostState::new(
            plugin.plugin_id.clone(),
            plugin.granted_permissions.clone(),
            self.db.clone(),
            &self,
            self.ai_service.clone(),
        ))
    }

    pub async fn get_bindings(
        &self,
        plugin: &LoadedPlugin,
    ) -> anyhow::Result<(wasmtime::Store<super::PluginHostState>, super::PluginWorld)> {
        let engine = self.engine.get_engine();
        let linker = self.engine.get_linker();
        let state = self.new_state(plugin)?;
        let component = plugin.component.as_ref().unwrap();
        let mut store = wasmtime::Store::new(engine, state);
        let (bindings, _) =
            super::PluginWorld::instantiate_async(&mut store, component, &linker).await?;

        Ok((store, bindings))
    }
    pub async fn call_filter_hook(
        &self,
        hook_name: &str,
        data: &PluginFilterEvent,
    ) -> anyhow::Result<PluginFilterEvent> {
        let plugins = self.get_plugins_for_hook(hook_name).await;
        tracing::info!(
            "Calling filter hook {} with data: {:?} and {} plugins",
            hook_name,
            data,
            &plugins.len()
        );
        for plugin in &plugins {
            tracing::info!("Plugin: {:?}", plugin.plugin_id);
        }

        let mut modified_data = data.clone();
        for plugin in plugins {
            let (store, bindings) = self.get_bindings(&plugin).await?;
            modified_data = match bindings
                .rustpress_plugin_event_handler()
                .call_handle_filter(store, &modified_data)
                .await
            {
                Ok(plugin_result) => match plugin_result {
                    Ok(plugin_result) => plugin_result,
                    Err(e) => {
                        tracing::error!(
                            "Plugin {} returned error for {} filter hook: {}",
                            plugin.plugin_id,
                            hook_name,
                            e
                        );
                        // Return a 500 error if a plugin returned an error
                        return Err(anyhow::anyhow!(
                            "Plugin {} returned error for {} filter hook: {}",
                            plugin.plugin_id,
                            hook_name,
                            e
                        ));
                    }
                },
                Err(e) => {
                    tracing::error!(
                        "Error calling {} filter hook for plugin {}: {:?}",
                        hook_name,
                        plugin.plugin_id,
                        e
                    );
                    // Return a 500 error if a plugin returned an error
                    return Err(anyhow::anyhow!(
                        "Plugin {} returned error: {}",
                        plugin.plugin_id,
                        e
                    ));
                }
            }
        }
        Ok(modified_data)
    }

    pub async fn call_action_hook(
        &self,
        hook_name: &str,
        data: &PluginActionEvent,
    ) -> anyhow::Result<()> {
        let plugins = self.get_plugins_for_hook(hook_name).await;
        tracing::info!(
            "Calling action hook {} with data: {:?} and {} plugins",
            hook_name,
            data,
            &plugins.len()
        );

        for plugin in &plugins {
            tracing::info!("Plugin: {:?}", plugin.plugin_id);
        }

        for plugin in plugins {
            let (store, bindings) = self.get_bindings(&plugin).await?;
            match bindings
                .rustpress_plugin_event_handler()
                .call_handle_action(store, data)
                .await
            {
                Ok(_) => {
                    // Action hooks don't return data, just log success
                    tracing::debug!(
                        "Successfully called {} action hook for plugin {}",
                        hook_name,
                        plugin.plugin_id
                    );
                }
                Err(e) => {
                    tracing::error!(
                        "Error calling {} action hook for plugin {}: {:?}",
                        hook_name,
                        plugin.plugin_id,
                        e
                    );
                    // For action hooks, we don't return errors, just log them
                    // This allows other plugins to still process the hook
                }
            }
        }
        Ok(())
    }
}
