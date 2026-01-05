use anyhow::{Context, Result};
use colored::*;
use std::fs;
use std::path::Path;
use std::process::Command;

pub fn build_plugin(release: &bool) -> Result<String> {
    println!("{}", "Building Rustpress plugin...".cyan().bold());

    // Check if we're in a plugin directory (has Cargo.toml)
    if !Path::new("Cargo.toml").exists() {
        anyhow::bail!("Not in a plugin directory. Make sure you're in a directory with Cargo.toml");
    }

    // Read plugin name from Cargo.toml
    let plugin_name = get_plugin_name()?;

    // Build the plugin for wasm32-wasip2 target
    let mut cargo_cmd = Command::new("cargo");
    cargo_cmd.arg("build");
    cargo_cmd.arg("--target").arg("wasm32-wasip2");

    if *release {
        cargo_cmd.arg("--release");
        println!("  Building in release mode...");
    } else {
        println!("  Building in debug mode...");
    }

    let status = cargo_cmd
        .status()
        .with_context(|| "Failed to run cargo build")?;

    if !status.success() {
        anyhow::bail!("Cargo build failed");
    }

    // Find the built wasm file
    let target_dir = if *release {
        "target/wasm32-wasip2/release"
    } else {
        "target/wasm32-wasip2/debug"
    };
    let wasm_files = find_wasm_files(target_dir)?;

    if wasm_files.is_empty() {
        anyhow::bail!("No .wasm files found in target directory");
    }

    // Assume the first wasm file is our plugin
    let wasm_file = &wasm_files[0];
    println!("  Found plugin binary: {}", wasm_file.display());

    // Copy to plugin.wasm in current directory
    let plugin_wasm_path = Path::new("plugin.wasm");
    fs::copy(wasm_file, plugin_wasm_path)
        .with_context(|| format!("Failed to copy {} to plugin.wasm", wasm_file.display()))?;

    println!(
        "  ✅ Plugin binary copied to: {}",
        plugin_wasm_path.display()
    );

    // Generate manifest.toml from Cargo.toml
    generate_manifest()?;

    println!("{}", "✅ Plugin built successfully!".green().bold());
    println!("  📦 Binary: plugin.wasm");
    println!("  📋 Manifest: manifest.toml");

    Ok(plugin_name)
}

fn get_plugin_name() -> Result<String> {
    // Read Cargo.toml
    let cargo_toml_content =
        fs::read_to_string("Cargo.toml").with_context(|| "Failed to read Cargo.toml")?;

    // Parse Cargo.toml
    let cargo_toml: toml::Value =
        toml::from_str(&cargo_toml_content).with_context(|| "Failed to parse Cargo.toml")?;

    // Extract package name
    let package = cargo_toml
        .get("package")
        .and_then(|p| p.as_table())
        .with_context(|| "No [package] section found in Cargo.toml")?;

    let name = package
        .get("name")
        .and_then(|n| n.as_str())
        .with_context(|| "No name field in [package] section")?;

    Ok(name.to_string())
}

fn find_wasm_files(dir: &str) -> Result<Vec<std::path::PathBuf>> {
    let mut wasm_files = Vec::new();
    let target_path = Path::new(dir);

    if target_path.exists() {
        for entry in fs::read_dir(target_path)
            .with_context(|| format!("Failed to read directory {}", dir))?
        {
            let entry = entry?;
            let path = entry.path();
            if let Some(ext) = path.extension()
                && ext == "wasm"
            {
                wasm_files.push(path);
            }
        }
    }

    Ok(wasm_files)
}

fn generate_manifest() -> Result<()> {
    // Read Cargo.toml
    let cargo_toml_content =
        fs::read_to_string("Cargo.toml").with_context(|| "Failed to read Cargo.toml")?;

    // Parse Cargo.toml
    let cargo_toml: toml::Value =
        toml::from_str(&cargo_toml_content).with_context(|| "Failed to parse Cargo.toml")?;

    // Extract package information
    let package = cargo_toml
        .get("package")
        .and_then(|p| p.as_table())
        .with_context(|| "No [package] section found in Cargo.toml")?;

    let name = package
        .get("name")
        .and_then(|n| n.as_str())
        .with_context(|| "No name field in [package] section")?;

    let version = package
        .get("version")
        .and_then(|v| v.as_str())
        .unwrap_or("0.1.0");

    let description = package
        .get("description")
        .and_then(|d| d.as_str())
        .unwrap_or("");

    // Extract rustpress metadata
    let mut manifest = toml::value::Table::new();

    // Initialize tables
    let mut package_table = toml::value::Table::new();
    package_table.insert("name".to_string(), toml::Value::String(name.to_string()));
    package_table.insert(
        "version".to_string(),
        toml::Value::String(version.to_string()),
    );
    package_table.insert(
        "description".to_string(),
        toml::Value::String(description.to_string()),
    );

    let mut plugin_table = toml::value::Table::new();
    plugin_table.insert(
        "binary".to_string(),
        toml::Value::String("plugin.wasm".to_string()),
    );

    // Extract metadata from [package.metadata.rustpress]
    if let Some(metadata) = cargo_toml
        .get("package")
        .and_then(|p| p.get("metadata"))
        .and_then(|m| m.get("rustpress"))
        .and_then(|r| r.as_table())
    {
        // Title
        if let Some(title) = metadata.get("title").and_then(|t| t.as_str()) {
            package_table.insert("title".to_string(), toml::Value::String(title.to_string()));
        }

        // Icon
        if let Some(icon) = metadata.get("icon").and_then(|i| i.as_str()) {
            package_table.insert("icon".to_string(), toml::Value::String(icon.to_string()));
        }

        // Package ID
        if let Some(package_id) = metadata.get("package-id").and_then(|p| p.as_str()) {
            package_table.insert(
                "id".to_string(),
                toml::Value::String(package_id.to_string()),
            );
        }

        // Hooks
        if let Some(hooks) = metadata.get("hooks").and_then(|h| h.as_array()) {
            let hook_strings: Vec<toml::Value> = hooks
                .iter()
                .filter_map(|h| h.as_str())
                .map(|s| toml::Value::String(s.to_string()))
                .collect();
            if !hook_strings.is_empty() {
                plugin_table.insert("hooks".to_string(), toml::Value::Array(hook_strings));
            }
        }

        // Permissions
        if let Some(permissions) = metadata.get("permissions").and_then(|p| p.as_table()) {
            let mut perms_table = toml::value::Table::new();

            // Required permissions
            if let Some(required) = permissions.get("required").and_then(|r| r.as_array()) {
                let required_strings: Vec<toml::Value> = required
                    .iter()
                    .filter_map(|p| p.as_str())
                    .map(|s| toml::Value::String(s.to_string()))
                    .collect();
                if !required_strings.is_empty() {
                    perms_table
                        .insert("required".to_string(), toml::Value::Array(required_strings));
                }
            }

            // Optional permissions
            if let Some(optional) = permissions.get("optional").and_then(|o| o.as_array()) {
                let optional_strings: Vec<toml::Value> = optional
                    .iter()
                    .filter_map(|p| p.as_str())
                    .map(|s| toml::Value::String(s.to_string()))
                    .collect();
                if !optional_strings.is_empty() {
                    perms_table
                        .insert("optional".to_string(), toml::Value::Array(optional_strings));
                }
            }

            if !perms_table.is_empty() {
                manifest.insert("permissions".to_string(), toml::Value::Table(perms_table));
            }
        }
    }

    // Dependencies section (always include rustpress-sdk)
    let mut deps_table = toml::value::Table::new();
    deps_table.insert(
        "rustpress-sdk".to_string(),
        toml::Value::String("*".to_string()),
    );
    manifest.insert("dependencies".to_string(), toml::Value::Table(deps_table));

    // Add tables to manifest
    manifest.insert("package".to_string(), toml::Value::Table(package_table));
    manifest.insert("plugin".to_string(), toml::Value::Table(plugin_table));

    // Generate TOML content
    let manifest_content = toml::to_string_pretty(&manifest)
        .with_context(|| "Failed to serialize manifest to TOML")?;

    // Write manifest.toml
    fs::write("manifest.toml", manifest_content)
        .with_context(|| "Failed to write manifest.toml")?;

    println!("  📋 Generated manifest.toml");

    Ok(())
}
