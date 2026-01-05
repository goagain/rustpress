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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::dto::plugin::{HooksConfig, PackageInfo, PermissionsConfig, PluginManifest};

    #[test]
    fn test_hook_registry_definitions() {
        let definitions = HookRegistry::get_hook_definitions();

        // Test that sensitive hooks require permissions
        assert!(definitions.contains_key("action_post_published"));
        assert_eq!(
            HookRegistry::get_hook_permission("action_post_published"),
            Some("post:read".to_string())
        );

        // Test that system hooks don't require permissions
        assert!(definitions.contains_key("action_system_startup"));
        assert_eq!(
            HookRegistry::get_hook_permission("action_system_startup"),
            None
        );
    }

    #[test]
    fn test_valid_hook_check() {
        assert!(HookRegistry::is_valid_hook("action_post_published"));
        assert!(HookRegistry::is_valid_hook("action_system_startup"));
        assert!(!HookRegistry::is_valid_hook("nonexistent_hook"));
    }

    #[test]
    fn test_hook_validation_edge_cases() {
        // Test empty hook list
        let manifest = create_test_manifest(vec![]);
        let granted_perms = std::collections::HashMap::new();
        let result = validate_hooks_for_test(&manifest, &granted_perms);
        assert!(result.is_ok());
        assert_eq!(result.unwrap().len(), 0);

        // Test single valid hook without permission
        let manifest = create_test_manifest(vec!["action_system_startup".to_string()]);
        let granted_perms = std::collections::HashMap::new();
        let result = validate_hooks_for_test(&manifest, &granted_perms);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), vec!["action_system_startup"]);

        // Test single hook requiring permission (granted)
        let manifest = create_test_manifest(vec!["action_post_published".to_string()]);
        let mut granted_perms = std::collections::HashMap::new();
        granted_perms.insert("post:read".to_string(), true);
        let result = validate_hooks_for_test(&manifest, &granted_perms);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), vec!["action_post_published"]);

        // Test single hook requiring permission (not granted)
        let manifest = create_test_manifest(vec!["action_post_published".to_string()]);
        let granted_perms = std::collections::HashMap::new();
        let result = validate_hooks_for_test(&manifest, &granted_perms);
        assert!(result.is_err());
        match result.unwrap_err() {
            HookValidationError::PluginSecurityViolation {
                plugin_id,
                hook,
                required,
            } => {
                assert_eq!(plugin_id, "test.plugin");
                assert_eq!(hook, "action_post_published");
                assert_eq!(required, "post:read");
            }
            _ => panic!("Expected PluginSecurityViolation"),
        }
    }

    #[test]
    fn test_unknown_hook_validation() {
        let manifest = create_test_manifest(vec!["unknown_hook".to_string()]);
        let granted_perms = std::collections::HashMap::new();
        let result = validate_hooks_for_test(&manifest, &granted_perms);
        assert!(result.is_err());
        match result.unwrap_err() {
            HookValidationError::UnknownHook(hook) => {
                assert_eq!(hook, "unknown_hook");
            }
            _ => panic!("Expected UnknownHook error"),
        }
    }

    #[test]
    fn test_mixed_valid_invalid_hooks() {
        let manifest = create_test_manifest(vec![
            "action_system_startup".to_string(), // Valid, no permission required
            "action_post_published".to_string(), // Valid, requires post:read
            "unknown_hook".to_string(),          // Invalid, unknown hook
        ]);

        let mut granted_perms = std::collections::HashMap::new();
        granted_perms.insert("post:read".to_string(), true);

        let result = validate_hooks_for_test(&manifest, &granted_perms);
        assert!(result.is_err());
        match result.unwrap_err() {
            HookValidationError::UnknownHook(hook) => {
                assert_eq!(hook, "unknown_hook");
            }
            _ => panic!("Expected UnknownHook error"),
        }
    }

    #[test]
    fn test_hook_validation_permission_denied() {
        let manifest = create_test_manifest(vec![
            "action_post_published".to_string(), // Requires post:read
            "filter_post_published".to_string(), // Requires post:write
            "action_user_created".to_string(),   // Requires user:read
        ]);

        let mut granted_perms = std::collections::HashMap::new();
        granted_perms.insert("post:read".to_string(), true); // Only grant post:read

        let result = validate_hooks_for_test(&manifest, &granted_perms);
        assert!(result.is_err());
        match result.unwrap_err() {
            HookValidationError::PluginSecurityViolation { hook, required, .. } => {
                // Should fail on the first missing permission
                assert!(hook == "filter_post_published" || hook == "action_user_created");
                assert!(required == "post:write" || required == "user:read");
            }
            _ => panic!("Expected PluginSecurityViolation"),
        }
    }

    #[test]
    fn test_hook_validation_partial_success() {
        // This test ensures that if we have a mix of valid and invalid hooks,
        // we fail fast on the first invalid one, not continue processing
        let manifest = create_test_manifest(vec![
            "action_system_startup".to_string(), // Valid
            "unknown_hook".to_string(),          // Invalid - should fail here
            "action_post_published".to_string(), // Valid but never reached
        ]);

        let granted_perms = std::collections::HashMap::new();
        let result = validate_hooks_for_test(&manifest, &granted_perms);
        assert!(result.is_err());
        match result.unwrap_err() {
            HookValidationError::UnknownHook(hook) => {
                assert_eq!(hook, "unknown_hook");
            }
            _ => panic!("Expected UnknownHook error"),
        }
    }

    #[test]
    fn test_hook_error_classification() {
        let unknown_hook_err = HookValidationError::UnknownHook("test_hook".to_string());
        assert!(!unknown_hook_err.is_security_violation());

        let security_err = HookValidationError::PluginSecurityViolation {
            plugin_id: "test.plugin".to_string(),
            hook: "test_hook".to_string(),
            required: "test:perm".to_string(),
        };
        assert!(security_err.is_security_violation());

        let missing_perm_err = HookValidationError::MissingPermissionForHook {
            hook: "test_hook".to_string(),
            required: "test:perm".to_string(),
        };
        assert!(!missing_perm_err.is_security_violation());
    }

    #[test]
    fn test_hook_registry_completeness() {
        let definitions = HookRegistry::get_hook_definitions();

        // Ensure we have a reasonable number of hooks defined
        assert!(
            definitions.len() >= 5,
            "Should have at least 5 hook definitions"
        );

        // Ensure all hooks have valid names (no empty strings)
        for (name, def) in &definitions {
            assert!(!name.is_empty(), "Hook name should not be empty");
            assert_eq!(def.name, name, "Hook definition name should match key");
            assert!(!def.description.is_empty(), "Hook should have description");
        }

        // Test specific hook properties
        let post_published = definitions.get("action_post_published").unwrap();
        assert_eq!(post_published.required_perm, Some("post:read"));
        assert!(post_published.description.contains("post"));

        let system_startup = definitions.get("action_system_startup").unwrap();
        assert_eq!(system_startup.required_perm, None);
        assert!(system_startup.description.contains("system"));
    }

    // Helper function to create test manifest
    fn create_test_manifest(hooks: Vec<String>) -> PluginManifest {
        PluginManifest {
            package: PackageInfo {
                id: "test.plugin".to_string(),
                name: "Test Plugin".to_string(),
                version: "1.0.0".to_string(),
                description: Some("Test plugin".to_string()),
                author: Some("Test Author".to_string()),
            },
            permissions: PermissionsConfig { required: vec![] },
            optional_permissions: std::collections::HashMap::new(),
            hooks: HooksConfig { registered: hooks },
        }
    }

    // Helper function to validate hooks (extracted from RpkProcessor for testing)
    fn validate_hooks_for_test(
        manifest: &PluginManifest,
        granted_permissions: &std::collections::HashMap<String, bool>,
    ) -> Result<Vec<String>, HookValidationError> {
        let hook_defs = HookRegistry::get_hook_definitions();
        let mut valid_hooks = Vec::new();

        // Validate each hook the plugin wants to register
        for hook_name in &manifest.hooks.registered {
            // 1. Check if hook is defined in host registry
            let hook_def = match hook_defs.get(hook_name) {
                Some(def) => def,
                None => {
                    return Err(HookValidationError::UnknownHook(hook_name.clone()));
                }
            };

            // 2. Check permission requirements
            if let Some(required_perm) = hook_def.required_perm {
                let has_permission = granted_permissions
                    .get(required_perm)
                    .copied()
                    .unwrap_or(false);

                if !has_permission {
                    // SECURITY VIOLATION: Plugin wants to register a hook that requires permissions it doesn't have
                    return Err(HookValidationError::PluginSecurityViolation {
                        plugin_id: manifest.package.id.clone(),
                        hook: hook_name.clone(),
                        required: required_perm.to_string(),
                    });
                }
            }

            // Hook passed validation
            valid_hooks.push(hook_name.clone());
        }

        Ok(valid_hooks)
    }
}
