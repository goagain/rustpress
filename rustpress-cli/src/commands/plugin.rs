use anyhow::{Context, Result};
use clap::Args;
use colored::*;
use std::fs;
use std::path::Path;

/// Plugin-related commands
#[derive(Args)]
pub struct PluginArgs {
    #[command(subcommand)]
    pub command: PluginCommand,
}

#[derive(clap::Subcommand)]
pub enum PluginCommand {
    /// Create a new Rustpress plugin
    New {
        /// Name of the plugin to create
        plugin_name: String,
    },
}

pub fn handle_plugin_command(args: &PluginArgs) -> Result<()> {
    match &args.command {
        PluginCommand::New { plugin_name } => create_new_plugin(plugin_name),
    }
}

fn create_new_plugin(plugin_name: &str) -> Result<()> {
    // Check if directory already exists
    let plugin_dir = Path::new(plugin_name);
    if plugin_dir.exists() {
        anyhow::bail!("Directory '{}' already exists!", plugin_name);
    }

    println!("{}", "Creating new Rustpress plugin...".cyan().bold());

    // Create the main directory
    fs::create_dir(plugin_dir)
        .with_context(|| format!("Failed to create directory '{}'", plugin_name))?;

    // Create src directory
    let src_dir = plugin_dir.join("src");
    fs::create_dir(&src_dir)
        .with_context(|| "Failed to create src directory")?;

    // Create Cargo.toml
    let cargo_toml_content = format!(
        r#"[package]
name = "{}"
version = "0.1.0"
edition = "2021"

[lib]
crate-type = ["cdylib"]

[dependencies]
anyhow = "1.0"
# Assuming the user will replace this with the real version or path later
rustpress-sdk = {{ path = "../rustpress-sdk" }}
wit-bindgen = "0.36.0"
"#,
        plugin_name
    );

    fs::write(plugin_dir.join("Cargo.toml"), cargo_toml_content)
        .with_context(|| "Failed to create Cargo.toml")?;

    // Create src/lib.rs
    let lib_rs_content = format!(
        r#"use rustpress_sdk::{{Guest, PostData}};

struct MyPlugin;

impl Guest for MyPlugin {{
    fn on_post_published(post: PostData) -> Result<PostData, String> {{
        // Example: Add a prefix to the title
        let mut new_post = post;
        println!("Plugin {} is processing post: {{}}", new_post.title);
        Ok(new_post)
    }}
}}

rustpress_sdk::export!(MyPlugin);
"#,
        plugin_name
    );

    fs::write(src_dir.join("lib.rs"), lib_rs_content)
        .with_context(|| "Failed to create src/lib.rs")?;

    // Create .gitignore
    let gitignore_content = r#"# Rust
target/
Cargo.lock

# IDE
.vscode/
.idea/

# OS
.DS_Store
Thumbs.db

# Logs
*.log

# Plugin output
*.rpk
"#;

    fs::write(plugin_dir.join(".gitignore"), gitignore_content)
        .with_context(|| "Failed to create .gitignore")?;

    // Create README.md
    let readme_content = format!(
        r#"# {}

A Rustpress Wasm plugin.

## Development

1. Build the plugin:
   ```bash
   cargo build --target wasm32-unknown-unknown --release
   ```

2. Install to your Rustpress instance:
   ```bash
   # Copy the .rpk file to your plugins directory
   cp target/wasm32-unknown-unknown/release/{}.rpk ~/rustpress/plugins/
   ```

3. Restart your Rustpress server to load the plugin.

## Plugin Features

This plugin implements the `Guest` trait and provides an `on_post_published` hook.

## License

MIT
"#,
        plugin_name, plugin_name
    );

    fs::write(plugin_dir.join("README.md"), readme_content)
        .with_context(|| "Failed to create README.md")?;

    println!("{}", "✅ Plugin created successfully!".green().bold());
    println!("📁 Directory: {}", plugin_name.cyan());
    println!();
    println!("Next steps:");
    println!("  1. {}", format!("cd {}", plugin_name).yellow());
    println!("  2. {}", "cargo build --target wasm32-unknown-unknown --release".yellow());
    println!("  3. Copy the .rpk file to your Rustpress plugins directory");

    Ok(())
}