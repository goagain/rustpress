/// Loaded plugin instance with its hooks
#[derive(Clone)]
pub struct LoadedPlugin {
    pub plugin_id: String,
    pub version: String,
    pub registered_hooks: Vec<String>,
    pub component: Option<wasmtime::component::Component>,
    pub granted_permissions: std::collections::HashSet<String>,
}
