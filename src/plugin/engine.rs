use std::sync::Arc;

use sea_orm::DatabaseConnection;
use std::collections::HashSet;
use std::fs;
use wasmtime::component::Component;
use wasmtime::component::Linker;
use wasmtime::{Config, Engine};

use crate::plugin::loaded_plugin::LoadedPlugin;
pub struct PluginEngine {
    engine: Engine,
    linker: Linker<super::PluginHostState>,
}

impl PluginEngine {
    pub fn new() -> anyhow::Result<Self> {
        // 1. initialize Engine (very expensive operation)
        let mut config = Config::new();
        config.wasm_component_model(true); // enable component model
        config.async_support(true); // enable async support
        let engine = Engine::new(&config)?;

        // 2. initialize Linker (define which functions the Host provides)
        let mut linker = Linker::<super::PluginHostState>::new(&engine);

        // 🔥 critical: add WASI standard interface
        // 🔥 critical: add your own Host interface (logger, db, etc.)
        // so that WASM can import them
        wasmtime_wasi::add_to_linker_async(&mut linker)?;
        super::PluginWorld::add_to_linker(&mut linker, |state: &mut super::PluginHostState| state)?;
        Ok(Self { engine, linker })
    }

    // pub fn new_state(&self, plugin: &LoadedPlugin) -> anyhow::Result<super::PluginHostState> {
    //     Ok(super::PluginHostState::new(
    //         plugin.plugin_id.clone(),
    //         plugin.granted_permissions.clone(),
    //         self.ai_client.clone(),
    //         self.db.clone(),
    //     ))
    // }

    pub fn get_engine(&self) -> &Engine {
        &self.engine
    }

    pub fn get_linker(&self) -> Linker<super::PluginHostState> {
        self.linker.clone()
    }

    pub async fn load_plugin_async(
        &self,
        plugin_id: &str,
        version: &str,
        wasm_path: &std::path::Path,
        hooks: Vec<String>,
    ) -> Result<LoadedPlugin, Box<dyn std::error::Error + Send + Sync>> {
        // Read the WASM file from the path, then construct and return a LoadedPlugin instance.

        // Read the plugin WASM file into memory
        let wasm_bytes = fs::read(wasm_path)?;
        // Use engine to parse the component (WASM module)
        let component = Component::from_binary(&self.engine, &wasm_bytes)?;

        // By default, we grant all hooks as permissions—can refine as needed
        let granted_permissions: HashSet<String> = hooks.iter().cloned().collect();

        let loaded_plugin = LoadedPlugin {
            plugin_id: plugin_id.to_string(),
            version: version.to_string(),
            registered_hooks: hooks.clone(),
            component: Some(component),
            granted_permissions,
        };

        Ok(loaded_plugin)
    }
}
