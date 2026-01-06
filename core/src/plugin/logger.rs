use std::sync::Arc;

use async_trait::async_trait;
use tracing::*;

use crate::plugin::logger::LogLevel;
use crate::plugin::rustpress::plugin::logger::*;

#[async_trait]
impl Host for super::PluginHostState {
    async fn log(&mut self, level: LogLevel, msg: String) -> Result<(), wasmtime::Error> {
        match level {
            LogLevel::Trace => trace!(plugin_id = %self.plugin_id, "[{}] {}", self.plugin_id, msg),
            LogLevel::Info => info!(plugin_id = %self.plugin_id,  "[{}] {}", self.plugin_id, msg),
            LogLevel::Warn => warn!(plugin_id = %self.plugin_id,  "[{}] {}", self.plugin_id, msg),
            LogLevel::Error => error!(plugin_id = %self.plugin_id, "[{}] {}", self.plugin_id, msg),
            LogLevel::Debug => debug!(plugin_id = %self.plugin_id, "[{}] {}", self.plugin_id, msg),
        }
        Ok(())
    }
}
