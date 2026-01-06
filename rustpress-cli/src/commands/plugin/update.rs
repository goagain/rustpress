use anyhow::{Context, Result};
use colored::Colorize;
use rust_embed::RustEmbed;
use std::fs;
use std::path::Path;

// WIT files embedded using rust-embed
#[derive(RustEmbed)]
#[folder = "../wit/"]
struct WitAssets;

pub fn update_plugin_wit(force: &bool) -> Result<()> {
    println!("{}", "Updating Rustpress plugin WIT files...".cyan().bold());

    // Check if we're in a plugin directory (has Cargo.toml and wit directory)
    if !Path::new("Cargo.toml").exists() {
        anyhow::bail!("Not in a plugin directory. Make sure you're in a directory with Cargo.toml");
    }

    if !Path::new("wit").exists() {
        anyhow::bail!(
            "No wit directory found. This doesn't appear to be a Rustpress plugin directory."
        );
    }

    let wit_dir = Path::new("wit");

    println!("  📁 Found WIT directory: {}", wit_dir.display());

    let mut updated_count = 0;
    let mut created_count = 0;

    // Update WIT files from embedded assets
    for file_path in WitAssets::iter() {
        let file_name = file_path.as_ref();
        let asset = WitAssets::get(&file_path)
            .with_context(|| format!("Failed to get embedded asset: {}", file_name))?;

        let content = std::str::from_utf8(&asset.data)
            .with_context(|| format!("Failed to decode UTF-8 content for: {}", file_name))?;

        let wit_file_path = wit_dir.join(file_name);

        let should_update = if wit_file_path.exists() {
            if *force {
                true
            } else {
                // Check if content is different
                match fs::read_to_string(&wit_file_path) {
                    Ok(existing_content) => existing_content != content,
                    Err(_) => true, // If we can't read, assume update is needed
                }
            }
        } else {
            true
        };

        if should_update {
            fs::write(&wit_file_path, content).with_context(|| {
                format!("Failed to write WIT file: {}", wit_file_path.display())
            })?;

            if wit_file_path.exists() && !*force {
                updated_count += 1;
                println!("  🔄 Updated: {}", file_name);
            } else {
                created_count += 1;
                println!("  ➕ Created: {}", file_name);
            }
        } else {
            println!("  ✓ Unchanged: {}", file_name);
        }
    }

    // Check for obsolete files (files in wit directory that are not in embedded assets)
    let mut obsolete_files = Vec::new();
    if let Ok(entries) = fs::read_dir(wit_dir) {
        for entry in entries.filter_map(Result::ok) {
            let path = entry.path();
            if path.is_file()
                && let Some(file_name) = path.file_name().and_then(|n| n.to_str())
                && let Some(ext) = path.extension()
                && ext == "wit"
            {
                // Check if this file exists in embedded assets
                let exists_in_assets =
                    WitAssets::iter().any(|asset_path| asset_path.as_ref() == file_name);

                if !exists_in_assets {
                    obsolete_files.push(file_name.to_string());
                }
            }
        }
    }

    if !obsolete_files.is_empty() {
        println!();
        println!("{}", "⚠️  Found obsolete WIT files:".yellow());
        for file in &obsolete_files {
            println!("    {}", file);
        }
        println!("  These files are no longer part of the WIT interface.");
        println!("  Consider removing them if they're not needed for your plugin.");
    }

    println!();
    println!(
        "{}",
        "✅ Plugin WIT files updated successfully!".green().bold()
    );

    if updated_count > 0 || created_count > 0 {
        println!(
            "  📊 Summary: {} updated, {} created",
            updated_count, created_count
        );
    } else {
        println!("  📊 All WIT files are already up to date!");
    }

    if !obsolete_files.is_empty() {
        println!("  ⚠️  {} obsolete files found", obsolete_files.len());
    }

    Ok(())
}
