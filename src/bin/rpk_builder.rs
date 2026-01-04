//! RPK Builder - Build and package RustPress plugins into RPK format

use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::process::Command;
use zip::ZipWriter;
use zip::write::FileOptions;

/// Build and package a plugin into RPK format
fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<String> = std::env::args().collect();

    if args.len() != 3 {
        eprintln!("Usage: {} <plugin-directory> <output.rpk>", args[0]);
        std::process::exit(1);
    }

    let plugin_dir = PathBuf::from(&args[1]);
    let output_file = PathBuf::from(&args[2]);

    println!("🔨 Building RPK package...");
    println!("📁 Plugin directory: {}", plugin_dir.display());
    println!("📦 Output file: {}", output_file.display());

    // Check if plugin directory exists
    if !plugin_dir.exists() {
        return Err(format!("Plugin directory does not exist: {}", plugin_dir.display()).into());
    }

    // Read and validate manifest
    let manifest_path = plugin_dir.join("manifest.toml");
    if !manifest_path.exists() {
        return Err("manifest.toml not found in plugin directory".into());
    }

    let manifest_content = fs::read_to_string(&manifest_path)?;
    let manifest: toml::Value = toml::from_str(&manifest_content)?;

    // Extract plugin ID from manifest
    let plugin_id = manifest
        .get("package")
        .and_then(|p| p.get("id"))
        .and_then(|id| id.as_str())
        .ok_or("Invalid manifest: missing package.id")?;

    println!("📋 Plugin ID: {}", plugin_id);

    // Build the plugin as WASM
    build_wasm_plugin(&plugin_dir)?;

    // Create RPK package
    create_rpk_package(&plugin_dir, &output_file, plugin_id)?;

    println!(
        "✅ RPK package created successfully: {}",
        output_file.display()
    );

    // Validate the created RPK package
    validate_rpk_package(&output_file)?;

    Ok(())
}

/// Validate the created RPK package
fn validate_rpk_package(rpk_path: &Path) -> Result<(), Box<dyn std::error::Error>> {
    println!("🔍 Validating RPK package...");

    use std::collections::HashMap;
    use std::io::Read;
    use zip::ZipArchive;

    // Open the ZIP file
    let file = std::fs::File::open(rpk_path)?;
    let mut archive = ZipArchive::new(file)?;

    let mut manifest: Option<toml::Value> = None;
    let mut files = HashMap::new();

    // Process each file in the archive
    for i in 0..archive.len() {
        let mut file = archive.by_index(i)?;
        let file_name = file.name().to_string();

        // Read file content
        let mut content = Vec::new();
        file.read_to_end(&mut content)?;

        // Special handling for manifest
        if file_name == "manifest.toml" {
            let manifest_content = String::from_utf8(content.clone())?;
            manifest = Some(toml::from_str(&manifest_content)?);
        }

        files.insert(file_name, content);
    }

    let manifest = manifest.ok_or("manifest.toml not found in RPK package")?;

    // Extract manifest data
    let manifest_id = manifest
        .get("package")
        .and_then(|p| p.get("id"))
        .and_then(|id| id.as_str())
        .ok_or("Invalid manifest: missing package.id")?;

    let plugin_name = manifest
        .get("package")
        .and_then(|p| p.get("name"))
        .and_then(|n| n.as_str())
        .unwrap_or("Unknown");

    let version = manifest
        .get("package")
        .and_then(|p| p.get("version"))
        .and_then(|v| v.as_str())
        .unwrap_or("Unknown");

    let permissions = manifest
        .get("permissions")
        .and_then(|p| p.get("required"))
        .and_then(|p| p.as_array())
        .map(|arr| arr.iter().filter_map(|v| v.as_str()).collect::<Vec<_>>())
        .unwrap_or_default();

    let hooks = manifest
        .get("hooks")
        .and_then(|h| h.get("registered"))
        .and_then(|h| h.as_array())
        .map(|arr| arr.iter().filter_map(|v| v.as_str()).collect::<Vec<_>>())
        .unwrap_or_default();

    println!("📋 Manifest ID: {}", manifest_id);
    println!("📋 Plugin Name: {}", plugin_name);
    println!("📋 Version: {}", version);
    println!("🔧 Permissions: {:?}", permissions);
    println!("🪝 Hooks: {:?}", hooks);
    println!("📄 Files in package:");
    for (filename, content) in &files {
        println!("  - {} ({} bytes)", filename, content.len());
    }

    // Verify required files
    if !files.contains_key("manifest.toml") {
        return Err("manifest.toml missing from RPK".into());
    }
    if !files.contains_key("plugin.wasm") {
        return Err("plugin.wasm missing from RPK".into());
    }

    println!("✅ RPK package validation successful!");
    Ok(())
}

/// Build the plugin as WASM
fn build_wasm_plugin(plugin_dir: &Path) -> Result<(), Box<dyn std::error::Error>> {
    println!("🏗️  Building WASM plugin...");
    println!("Plugin dir: {}", plugin_dir.display());

    let output = Command::new("cargo")
        .args(&["build", "--target", "wasm32-unknown-unknown"])
        .current_dir(plugin_dir)
        .output()?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(format!("Failed to build WASM plugin:\n{}", stderr).into());
    }

    // Convert to WASM component
    println!("🔧 Converting to WASM component...");
    let target_dir = plugin_dir.join("target/wasm32-unknown-unknown/debug");
    let wasm_file = target_dir.join("poetry_plugin.wasm");

    println!("WASM file path: {}", wasm_file.display());
    println!("Current dir: {}", plugin_dir.display());

    // Use absolute path and convert to forward slashes for wasm-tools
    let abs_wasm_file = std::fs::canonicalize(&wasm_file)?;
    let abs_wasm_path = abs_wasm_file.to_string_lossy().replace("\\", "/");
    let wit_path = plugin_dir.join("../../wit/plugin.wit");
    println!("WIT path: {}", wit_path.display());
    let abs_wit_path = std::fs::canonicalize(&wit_path)?
        .to_string_lossy()
        .replace("\\", "/");
    println!("Absolute WIT path: {}", abs_wit_path);

    let component_output = Command::new("wasm-tools")
        .args(&[
            "component",
            "embed",
            "--world",
            "plugin-world",
            &abs_wit_path,
            &abs_wasm_path,
            "-o",
            "plugin.wasm",
        ])
        .current_dir(plugin_dir)
        .output()?;

    if !component_output.status.success() {
        let stderr = String::from_utf8_lossy(&component_output.stderr);
        let stdout = String::from_utf8_lossy(&component_output.stdout);
        println!("wasm-tools stdout: {}", stdout);
        println!("wasm-tools stderr: {}", stderr);
        return Err(format!("Failed to create WASM component:\n{}", stderr).into());
    }

    println!("✅ WASM plugin built successfully");
    Ok(())
}

/// Create RPK package (ZIP file)
fn create_rpk_package(
    plugin_dir: &Path,
    output_file: &Path,
    plugin_id: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    println!("📦 Creating RPK package...");

    // Create output file
    let file = fs::File::create(output_file)?;
    let mut zip = ZipWriter::new(file);
    let options = FileOptions::default()
        .compression_method(zip::CompressionMethod::Deflated)
        .unix_permissions(0o755);

    // Add manifest.toml
    let manifest_path = plugin_dir.join("manifest.toml");
    let manifest_content = fs::read(&manifest_path)?;
    zip.start_file("manifest.toml", options)?;
    zip.write_all(&manifest_content)?;
    println!("📝 Added manifest.toml");

    // Add plugin.wasm
    let wasm_path = plugin_dir.join("target/wasm32-unknown-unknown/release/poetry_plugin.wasm");
    if !wasm_path.exists() {
        return Err("WASM file not found. Make sure the plugin built successfully.".into());
    }

    let wasm_content = fs::read(&wasm_path)?;
    zip.start_file("plugin.wasm", options)?;
    zip.write_all(&wasm_content)?;
    println!("🔗 Added plugin.wasm ({} bytes)", wasm_content.len());

    // Add any additional files (frontend, admin_frontend, etc.)
    add_additional_files(&mut zip, plugin_dir, options)?;

    zip.finish()?;
    println!("📦 RPK package created");

    Ok(())
}

/// Add any additional files to the RPK package
fn add_additional_files(
    zip: &mut ZipWriter<fs::File>,
    plugin_dir: &Path,
    options: FileOptions,
) -> Result<(), Box<dyn std::error::Error>> {
    let additional_dirs = ["frontend", "admin_frontend", "assets"];

    for dir_name in &additional_dirs {
        let dir_path = plugin_dir.join(dir_name);
        if dir_path.exists() && dir_path.is_dir() {
            add_directory_to_zip(zip, &dir_path, Path::new(dir_name), options)?;
        }
    }

    Ok(())
}

/// Recursively add directory contents to ZIP
fn add_directory_to_zip(
    zip: &mut ZipWriter<fs::File>,
    source_dir: &Path,
    relative_path: &Path,
    options: FileOptions,
) -> Result<(), Box<dyn std::error::Error>> {
    for entry in fs::read_dir(source_dir)? {
        let entry = entry?;
        let path = entry.path();
        let relative_entry_path = relative_path.join(path.file_name().unwrap());

        if path.is_dir() {
            add_directory_to_zip(zip, &path, &relative_entry_path, options)?;
        } else {
            let content = fs::read(&path)?;
            zip.start_file(relative_entry_path.to_string_lossy(), options)?;
            zip.write_all(&content)?;
            println!("📄 Added {}", relative_entry_path.display());
        }
    }

    Ok(())
}
