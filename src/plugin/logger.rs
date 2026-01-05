use std::sync::Arc;

use async_trait::async_trait;
use tracing::*;

use crate::plugin::logger::LogLevel;
use crate::plugin::rustpress::plugin::logger::*;

#[async_trait]
impl Host for super::PluginHostState {
    async fn log(&mut self, level: LogLevel, msg: String) -> Result<(), wasmtime::Error> {
        match level {
            LogLevel::Trace => trace!(plugin_id = %self.plugin_id, "[WASM] {}", msg),
            LogLevel::Info => info!(plugin_id = %self.plugin_id,  "[WASM] {}", msg),
            LogLevel::Warn => warn!(plugin_id = %self.plugin_id,  "[WASM] {}", msg),
            LogLevel::Error => error!(plugin_id = %self.plugin_id, "[WASM] {}", msg),
            LogLevel::Debug => debug!(plugin_id = %self.plugin_id, "[WASM] {}", msg),
        }
        Ok(())
    }
}
