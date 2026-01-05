// This module provides the runtime implementation for the plugin permissions WIT interface.
// All code, comments, and documentation are in English as per project rules.

use axum::async_trait;

use crate::plugin::rustpress::plugin::permissions::*;

#[async_trait]
impl Host for super::PluginHostState {
    async fn list_permissions(&mut self) -> Result<Vec<String>, wasmtime::Error> {
        Ok(self.granted_permissions.iter().cloned().collect())
    }

    async fn permission_granted(&mut self, permission: String) -> Result<bool, wasmtime::Error> {
        Ok(self.granted_permissions.contains(&permission))
    }
}
