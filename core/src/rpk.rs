//! RPK (RustPress Plugin) package format handling
//!
//! RPK is a ZIP-based plugin package format that contains:
//! - manifest.toml: Plugin manifest with permissions and hooks
//! - plugin.wasm: WebAssembly plugin binary
//! - frontend/: Frontend assets (future use)
//! - admin_frontend/: Admin assets (future use)

use crate::dto::plugin::PluginManifest;
use std::collections::HashMap;
use std::fs;
use std::io::{self, Read};
use std::path::{Path, PathBuf};
use zip::ZipArchive;

/// RPK package structure
#[derive(Debug)]
pub struct RpkPackage {
    /// Plugin manifest
    pub manifest: PluginManifest,
    /// Extracted files (path -> content)
    pub files: HashMap<String, Vec<u8>>,
}

/// Installed plugin information
#[derive(Debug)]
pub struct InstalledPlugin {
    /// Plugin manifest
    pub manifest: PluginManifest,
    /// Cache directory for this plugin (determined by host)
    pub cache_dir: PathBuf,
    /// Plugin status
    pub status: String,
}

/// RPK package processor
pub struct RpkProcessor {
    /// Base directory for plugin installations
    install_dir: PathBuf,
    /// Cache directory for extracted plugins
    cache_dir: PathBuf,
}

impl RpkProcessor {
    /// Create a new RPK processor
    pub fn new(install_dir: PathBuf, cache_dir: PathBuf) -> Self {
        Self {
            install_dir,
            cache_dir,
        }
    }

    /// Initialize directories
    pub fn init(&self) -> io::Result<()> {
        fs::create_dir_all(&self.install_dir)?;
        fs::create_dir_all(&self.cache_dir)?;
        Ok(())
    }

    /// Install RPK package (only extracts and validates, caching is handled by caller)
    pub async fn install_rpk(
        &self,
        plugin_id: &str,
        rpk_data: &[u8],
    ) -> Result<RpkPackage, Box<dyn std::error::Error + Send + Sync>> {
        // 1. Save RPK file to install directory
        let rpk_path = self.install_dir.join(format!("{}.rpk", plugin_id));
        fs::write(&rpk_path, rpk_data)?;

        // 2. Extract and validate RPK
        let package = self
            .extract_and_validate(&rpk_path, Some(plugin_id))
            .await?;

        Ok(package)
    }

    /// Cache extracted package files to specified directory
    pub async fn cache_package_files(
        &self,
        package: &RpkPackage,
        cache_dir: &Path,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        // Create cache directory
        fs::create_dir_all(cache_dir)?;

        // Write manifest
        let manifest_path = cache_dir.join("manifest.toml");
        let manifest_content = toml::to_string(&package.manifest)?;
        fs::write(manifest_path, manifest_content)?;

        // Write all files
        for (file_path, content) in &package.files {
            let full_path = cache_dir.join(file_path);
            if let Some(parent) = full_path.parent() {
                fs::create_dir_all(parent)?;
            }
            fs::write(full_path, content)?;
        }

        Ok(())
    }

    /// Extract and validate RPK package
    pub async fn extract_and_validate(
        &self,
        rpk_path: &Path,
        expected_plugin_id: Option<&str>,
    ) -> Result<RpkPackage, Box<dyn std::error::Error + Send + Sync>> {
        let file = fs::File::open(rpk_path)?;
        let mut archive = ZipArchive::new(file)?;

        let mut manifest: Option<PluginManifest> = None;
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

        // Validate manifest matches expected plugin_id (if provided)
        if let Some(expected_id) = expected_plugin_id {
            if manifest.package.id != expected_id {
                return Err(format!(
                    "Manifest ID '{}' does not match expected plugin ID '{}'",
                    manifest.package.id, expected_id
                )
                .into());
            }
        }
        // Validate required files exist
        if !files.contains_key("plugin.wasm") {
            return Err("plugin.wasm not found in RPK package".into());
        }

        Ok(RpkPackage { manifest, files })
    }
}
