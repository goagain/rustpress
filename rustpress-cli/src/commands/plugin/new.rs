use anyhow::{Context, Result};
use colored::Colorize;
use rust_embed::RustEmbed;
use std::collections::HashMap;
use std::fs;
use std::path::Path;

// Templates embedded using include_str!
const CARGO_TOML_TEMPLATE: &str = include_str!("../../../templates/Cargo.toml.tmpl");
const LIB_RS_TEMPLATE: &str = include_str!("../../../templates/src/lib.rs.tmpl");
const LOGGER_RS_TEMPLATE: &str = include_str!("../../../templates/src/logger.rs.tmpl");
const GITIGNORE_TEMPLATE: &str = include_str!("../../../templates/.gitignore.tmpl");
const README_TEMPLATE: &str = include_str!("../../../templates/README.md.tmpl");

// WIT files embedded using rust-embed
#[derive(RustEmbed)]
#[folder = "../wit/"]
struct WitAssets;

fn to_pascal_case(s: &str) -> String {
    s.split(['_', '-'])
        .map(|word| {
            let mut chars = word.chars();
            match chars.next() {
                None => String::new(),
                Some(first) => {
                    first.to_uppercase().collect::<String>()
                        + chars.as_str().to_lowercase().as_str()
                }
            }
        })
        .collect()
}

fn render_template(template_name: &str, replacements: &HashMap<&str, &str>) -> Result<String> {
    let template_content = match template_name {
        "Cargo.toml.tmpl" => CARGO_TOML_TEMPLATE,
        "src/lib.rs.tmpl" => LIB_RS_TEMPLATE,
        "src/logger.rs.tmpl" => LOGGER_RS_TEMPLATE,
        ".gitignore.tmpl" => GITIGNORE_TEMPLATE,
        "README.md.tmpl" => README_TEMPLATE,
        _ => return Err(anyhow::anyhow!("Unknown template: {}", template_name)),
    };

    let mut result = template_content.to_string();
    for (key, value) in replacements {
        let placeholder = format!("{{{{{}}}}}", key);
        result = result.replace(&placeholder, value);
    }

    Ok(result)
}

pub fn create_new_plugin(plugin_name: &str) -> Result<()> {
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
    fs::create_dir(&src_dir).with_context(|| "Failed to create src directory")?;

    // Create wit directory
    let wit_dir = plugin_dir.join("wit");
    fs::create_dir(&wit_dir).with_context(|| "Failed to create wit directory")?;

    // Prepare template replacements
    let mut replacements = HashMap::new();
    replacements.insert("plugin_name", plugin_name);

    // Convert plugin_name to PascalCase for struct names
    let plugin_name_pascal = to_pascal_case(plugin_name);
    replacements.insert("plugin_name_pascal", &plugin_name_pascal);

    // Create Cargo.toml from template
    let cargo_toml_content = render_template("Cargo.toml.tmpl", &replacements)?;
    fs::write(plugin_dir.join("Cargo.toml"), cargo_toml_content)
        .with_context(|| "Failed to create Cargo.toml")?;

    // Create src/lib.rs from template
    let lib_rs_content = render_template("src/lib.rs.tmpl", &replacements)?;
    fs::write(src_dir.join("lib.rs"), lib_rs_content)
        .with_context(|| "Failed to create src/lib.rs")?;

    // Create .gitignore from template
    let gitignore_content = render_template(".gitignore.tmpl", &replacements)?;
    fs::write(plugin_dir.join(".gitignore"), gitignore_content)
        .with_context(|| "Failed to create .gitignore")?;

    // Create README.md from template
    let readme_content = render_template("README.md.tmpl", &replacements)?;
    fs::write(plugin_dir.join("README.md"), readme_content)
        .with_context(|| "Failed to create README.md")?;

    // Create WIT files from embedded assets
    for file_path in WitAssets::iter() {
        let file_name = file_path.as_ref();
        let asset = WitAssets::get(&file_path)
            .with_context(|| format!("Failed to get embedded asset: {}", file_name))?;
        let content = std::str::from_utf8(&asset.data)
            .with_context(|| format!("Failed to decode UTF-8 content for: {}", file_name))?;

        fs::write(wit_dir.join(file_name), content)
            .with_context(|| format!("Failed to create wit/{}", file_name))?;
    }
    // Create src/logger.rs from template
    let logger_rs_content = render_template("src/logger.rs.tmpl", &replacements)?;
    fs::write(src_dir.join("logger.rs"), logger_rs_content)
        .with_context(|| "Failed to create src/logger.rs")?;

    println!("{}", "✅ Plugin created successfully!".green().bold());
    println!("📁 Directory: {}", plugin_name.cyan());
    println!();
    println!("Next steps:");
    println!("  1. {}", format!("cd {}", plugin_name).yellow());
    println!(
        "  2. {}",
        "cargo build --target wasm32-unknown-unknown --release".yellow()
    );
    println!("  3. Copy the .rpk file to your Rustpress plugins directory");

    Ok(())
}
