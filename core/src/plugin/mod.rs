//! Plugin system implementation

pub mod engine;
pub mod hook_registry;
pub mod host;
pub mod loaded_plugin;
pub mod registry;
pub mod types;

use std::sync::Arc;

use wasmtime_wasi::{ResourceTable, WasiCtx};

use crate::{ai::AiService, plugin::registry::PluginRegistry};

wasmtime::component::bindgen!({
    world: "plugin-world",
    path: "../wit",
    async: true
});

pub struct PluginHostState {
    ctx: WasiCtx,
    table: ResourceTable,
    plugin_id: String,
    granted_permissions: std::collections::HashSet<String>,
    db: Arc<sea_orm::DatabaseConnection>,
    plugin_registry: Arc<PluginRegistry>,
    ai_service: Option<Arc<AiService>>,
}

impl PluginHostState {
    /// Create a new plugin host state
    pub fn new(
        plugin_id: String,
        granted_permissions: std::collections::HashSet<String>,
        db: Arc<sea_orm::DatabaseConnection>,
        plugin_registry: &PluginRegistry,
        ai_service: Option<Arc<AiService>>,
    ) -> Self {
        let ctx = wasmtime_wasi::WasiCtxBuilder::new()
            .inherit_stderr()
            .inherit_stdout()
            .build();
        let table = ResourceTable::new();

        Self {
            ctx,
            table,
            plugin_id,
            granted_permissions,
            db,
            plugin_registry: Arc::new(plugin_registry.clone()),
            ai_service,
        }
    }

    pub fn get_plugin_registry(&self) -> Arc<crate::plugin::registry::PluginRegistry> {
        self.plugin_registry.clone()
    }
}

impl wasmtime_wasi::WasiView for PluginHostState {
    fn ctx(&mut self) -> &mut WasiCtx {
        &mut self.ctx
    }
    fn table(&mut self) -> &mut ResourceTable {
        &mut self.table
    }
}

/// Validate plugin hooks against the hook registry
fn validate_plugin_hooks(
    manifest: &crate::dto::plugin::PluginManifest,
) -> Result<Vec<String>, Box<dyn std::error::Error + Send + Sync>> {
    use crate::plugin::hook_registry::HookRegistry;
    let mut valid_hooks = Vec::new();

    for hook_name in &manifest.plugin.hooks {
        if !HookRegistry::is_valid_hook(hook_name) {
            return Err(format!("Unknown hook: {}", hook_name).into());
        }
        valid_hooks.push(hook_name.clone());
    }

    Ok(valid_hooks)
}
