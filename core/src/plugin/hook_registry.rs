//! Hook Registry - Gatekeeper for Plugin Security
//!
//! This module defines the "Host Side Truth" for hook permissions.
//! Each hook is explicitly defined with its required permissions to prevent
//! data leakage vulnerabilities.

use std::collections::HashMap;

/// Hook definition with permission requirements
#[derive(Debug, Clone)]
pub struct HookDef {
    pub name: &'static str,
    pub required_perm: Option<&'static str>, // Required permission for this hook
    pub description: &'static str,
}

/// Hook registry that defines all valid hooks and their permission requirements
pub struct HookRegistry;

impl HookRegistry {
    /// Get all valid hook definitions (Host Side Truth)
    pub fn get_hook_definitions() -> HashMap<String, HookDef> {
        let mut map = HashMap::new();

        // === Content-Related Hooks ===
        // These hooks carry sensitive post content data
        map.insert(
            "post_published_filter".into(),
            HookDef {
                name: "post_published_filter",
                required_perm: Some("post:write"), // Must be able to read posts to receive content
                description: "Triggered after a post is published (receives full post content)",
            },
        );

        map.insert(
            "list_categories".into(),
            HookDef {
                name: "list_categories",
                required_perm: Some("post:list_category"), // Must be able to list categories
                description: "Allows listing of all categories with their post counts",
            },
        );

        // === User-Related Hooks ===
        // These hooks carry user data
        map.insert(
            "action_user_created".into(),
            HookDef {
                name: "action_user_created",
                required_perm: Some("user:read"), // Must be able to read users
                description: "Triggered when a new user is created",
            },
        );

        map.insert(
            "action_user_login".into(),
            HookDef {
                name: "action_user_login",
                required_perm: Some("user:read"), // Must be able to read users
                description: "Triggered when a user logs in",
            },
        );

        map.insert(
            "filter_authenticate".into(),
            HookDef {
                name: "filter_authenticate",
                required_perm: Some("user:write"), // Must be able to modify user data
                description: "Allows modification of authentication process",
            },
        );

        // === System Hooks (No sensitive data) ===
        // These are pure notification hooks without sensitive data
        map.insert(
            "action_system_startup".into(),
            HookDef {
                name: "action_system_startup",
                required_perm: None, // No sensitive data, no permission required
                description: "Triggered when the system starts up",
            },
        );

        map.insert(
            "action_system_shutdown".into(),
            HookDef {
                name: "action_system_shutdown",
                required_perm: None, // No sensitive data, no permission required
                description: "Triggered when the system shuts down",
            },
        );

        // === Future hooks can be added here ===

        map
    }

    /// Check if a hook name is valid (defined in registry)
    pub fn is_valid_hook(hook_name: &str) -> bool {
        Self::get_hook_definitions().contains_key(hook_name)
    }

    /// Get the permission required for a specific hook
    pub fn get_hook_permission(hook_name: &str) -> Option<String> {
        Self::get_hook_definitions()
            .get(hook_name)
            .and_then(|def| def.required_perm)
            .map(|s| s.to_string())
    }

    /// Get hook definition for a specific hook
    pub fn get_hook_def(hook_name: &str) -> Option<HookDef> {
        Self::get_hook_definitions().get(hook_name).cloned()
    }
}

/// Hook validation errors
#[derive(Debug, thiserror::Error)]
pub enum HookValidationError {
    #[error("Unknown hook '{0}' - not defined in host registry")]
    UnknownHook(String),

    #[error("Hook '{hook}' requires permission '{required}' but plugin does not have it")]
    MissingPermissionForHook { hook: String, required: String },

    #[error(
        "Plugin '{plugin_id}' attempted to register hook '{hook}' without required permission '{required}'"
    )]
    PluginSecurityViolation {
        plugin_id: String,
        hook: String,
        required: String,
    },
}

impl HookValidationError {
    /// Check if this is a security violation that should be logged as a warning
    pub fn is_security_violation(&self) -> bool {
        matches!(self, HookValidationError::PluginSecurityViolation { .. })
    }
}
