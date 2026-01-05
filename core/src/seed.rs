//! Seed data for initial database population
//!
//! This module loads sample data from configuration files to populate the database
//! when the application starts for the first time.

use serde::Deserialize;
use std::fs;

/// Sample post data for initial seeding
#[derive(Debug, Clone)]
pub struct SamplePost {
    pub title: String,
    pub content: String,
    pub category: Option<String>,
    pub description: Option<String>,
}

/// Configuration structure for seed data
#[derive(Debug, Deserialize)]
struct SeedConfig {
    sample_post: SamplePostConfig,
}

/// Sample post configuration from TOML
#[derive(Debug, Deserialize)]
struct SamplePostConfig {
    title: String,
    category: Option<String>,
    content: String,
    description: Option<String>,
}

/// Get the default sample post from configuration file
/// Falls back to default values if file cannot be read
pub fn get_sample_post() -> SamplePost {
    // Try to read from configuration file
    if let Ok(config_content) = fs::read_to_string("seed_data.toml")
        && let Ok(config) = toml::from_str::<SeedConfig>(&config_content)
    {
        return SamplePost {
            title: config.sample_post.title,
            content: config.sample_post.content,
            category: config.sample_post.category,
            description: config.sample_post.description,
        };
    }

    // Fallback to default values if config file is not found or invalid
    tracing::warn!("⚠️  Could not read seed_data.toml, using default sample post");
    SamplePost {
        title: "Welcome to RustPress".to_string(),
        content: r#"# Welcome to RustPress

This is your first sample post. You can:

- Edit this post
- Delete this post
- Create new posts

RustPress is a modern blog system built with Rust.

Enjoy!"#
            .to_string(),
        category: Some("Announcement".to_string()),
        description: Some("This is your first sample post. You can:".to_string()),
    }
}
