use anyhow::{Context, Result};
use colored::*;
use std::fs;
use std::io::Write;
use std::path::Path;
use tempfile::TempDir;
use zip::ZipWriter;
use zip::write::FileOptions;

use super::build;

pub fn pack_plugin(release: &bool, output_dir: &Option<String>) -> Result<()> {
    println!("{}", "Packing Rustpress plugin...".cyan().bold());

    // Create a temporary directory for build artifacts
    let temp_dir = TempDir::new()
        .with_context(|| "Failed to create temporary directory for plugin build")?;
    let temp_path = temp_dir.path();

    println!("  📁 Using temporary directory: {}", temp_path.display());

    // Build the plugin in the temporary directory
    let plugin_name =
        build::build_plugin(release, temp_path).with_context(|| "Failed to build plugin before packing")?;

    println!("  📦 Creating RPK package...");

    // Check that required files exist in temp directory
    let plugin_wasm_path = temp_path.join("plugin.wasm");
    let manifest_toml_path = temp_path.join("manifest.toml");

    if !plugin_wasm_path.exists() {
        anyhow::bail!("plugin.wasm not found in temporary directory. Build failed.");
    }

    if !manifest_toml_path.exists() {
        anyhow::bail!("manifest.toml not found in temporary directory. Build failed.");
    }

    // Determine output path
    let output_dir = output_dir.as_deref().unwrap_or(".");
    let rpk_filename = format!("{}.rpk", plugin_name);
    let rpk_path = Path::new(output_dir).join(rpk_filename);

    // Ensure output directory exists
    if let Some(parent) = rpk_path.parent() {
        fs::create_dir_all(parent)
            .with_context(|| format!("Failed to create output directory: {}", parent.display()))?;
    }

    // Create the RPK file (ZIP archive)
    let rpk_file = fs::File::create(&rpk_path)
        .with_context(|| format!("Failed to create RPK file: {}", rpk_path.display()))?;

    let mut zip = ZipWriter::new(rpk_file);
    let options = FileOptions::default()
        .compression_method(zip::CompressionMethod::Deflated)
        .unix_permissions(0o644);

    // Add manifest.toml
    zip.start_file("manifest.toml", options)?;
    let manifest_content = fs::read(&manifest_toml_path)
        .with_context(|| format!("Failed to read manifest.toml from {}", manifest_toml_path.display()))?;
    zip.write_all(&manifest_content)?;

    // Add plugin.wasm
    zip.start_file("plugin.wasm", options)?;
    let wasm_content = fs::read(&plugin_wasm_path)
        .with_context(|| format!("Failed to read plugin.wasm from {}", plugin_wasm_path.display()))?;
    zip.write_all(&wasm_content)?;

    // Finish the ZIP file
    zip.finish()
        .with_context(|| "Failed to finalize RPK package")?;

    // Temporary directory will be automatically cleaned up when temp_dir goes out of scope

    println!("{}", "✅ Plugin packed successfully!".green().bold());
    println!("  📦 RPK file: {}", rpk_path.display());

    // Display file size
    if let Ok(metadata) = fs::metadata(&rpk_path) {
        let size_kb = metadata.len() / 1024;
        println!("  📏 Size: {} KB", size_kb);
    }

    println!();
    println!("Next steps:");
    println!(
        "  1. Upload {} to your Rustpress instance",
        rpk_path.display()
    );
    println!("  2. Install via admin panel or API");

    Ok(())
}
