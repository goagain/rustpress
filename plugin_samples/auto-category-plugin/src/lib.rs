// Auto Category Plugin
// Automatically generates categories for posts using AI

use std::collections::HashMap;

// Include generated WIT bindings
include!("plugin_world.rs");

struct AutoCategoryPlugin;

// Cache for category information
static mut CATEGORY_CACHE: Option<Vec<exports::rustpress::system::hooks::CategoryInfo>> = None;

impl exports::rustpress::system::hooks::Guest for AutoCategoryPlugin {
    fn on_post_published(
        mut post: exports::rustpress::system::hooks::PostData,
    ) -> Result<exports::rustpress::system::hooks::PostData, String> {
        // Check if post already has a category
        if post.category.is_some() {
            rustpress::system::logging::log_info(
                "Post already has a category, skipping auto-categorization",
            );
            return Ok(post);
        }

        rustpress::system::logging::log_info("Starting auto-categorization for new post");

        // Get available categories
        let categories = match Self::get_categories() {
            Ok(cats) => cats,
            Err(e) => {
                rustpress::system::logging::log_warn(&format!(
                    "Failed to get categories: {}, will generate new category",
                    e
                ));
                vec![]
            }
        };

        // Generate category using AI
        match Self::generate_category(&post, &categories) {
            Ok(category) => {
                post.category = Some(category.clone());
                rustpress::system::logging::log_info(&format!(
                    "Auto-generated category: {}",
                    category
                ));
                Ok(post)
            }
            Err(e) => {
                rustpress::system::logging::log_error(&format!(
                    "Failed to generate category: {}",
                    e
                ));
                // Return post unchanged if AI fails
                Ok(post)
            }
        }
    }

    fn list_categories() -> Vec<exports::rustpress::system::hooks::CategoryInfo> {
        // This method is called by the host, but we don't implement it in the plugin
        // The host provides the implementation
        vec![]
    }
}

impl AutoCategoryPlugin {
    fn get_categories() -> Result<Vec<exports::rustpress::system::hooks::CategoryInfo>, String> {
        // For now, return empty categories since we can't call host functions from plugin
        // In a real implementation, this would be provided by the host via some mechanism
        Ok(vec![])
    }

    fn generate_category(
        post: &exports::rustpress::system::hooks::PostData,
        existing_categories: &[exports::rustpress::system::hooks::CategoryInfo],
    ) -> Result<String, String> {
        // Simple categorization logic (placeholder for AI implementation)
        let title_lower = post.title.to_lowercase();
        let content_lower = post.content.to_lowercase();

        // Check for existing categories first
        if !existing_categories.is_empty() {
            for cat in existing_categories {
                let cat_lower = cat.name.to_lowercase();
                if title_lower.contains(&cat_lower) || content_lower.contains(&cat_lower) {
                    return Ok(cat.name.clone());
                }
            }
        }

        // Simple keyword-based categorization
        if title_lower.contains("tutorial")
            || title_lower.contains("guide")
            || content_lower.contains("step by step")
            || content_lower.contains("how to")
        {
            Ok("Tutorial".to_string())
        } else if title_lower.contains("news")
            || title_lower.contains("announcement")
            || content_lower.contains("released")
            || content_lower.contains("available")
        {
            Ok("News".to_string())
        } else if title_lower.contains("rust")
            || content_lower.contains("rust programming")
            || content_lower.contains("cargo")
            || content_lower.contains("crate")
        {
            Ok("Rust".to_string())
        } else if title_lower.contains("web")
            || content_lower.contains("http")
            || content_lower.contains("server")
            || content_lower.contains("api")
        {
            Ok("Web Development".to_string())
        } else {
            // Default category
            Ok("General".to_string())
        }
    }

    fn create_ai_prompt(
        post: &exports::rustpress::system::hooks::PostData,
        existing_categories: &[exports::rustpress::system::hooks::CategoryInfo],
    ) -> String {
        let mut prompt = format!(
            "Categorize this blog post into exactly ONE category.\n\nTitle: {}\n\nContent: {}\n\n",
            post.title,
            // Limit content length to avoid token limits
            if post.content.len() > 1000 {
                format!("{}...", &post.content[..1000])
            } else {
                post.content.clone()
            }
        );

        if !existing_categories.is_empty() {
            prompt.push_str("Available categories (with post counts):\n");
            for cat in existing_categories {
                prompt.push_str(&format!("- {} ({} posts)\n", cat.name, cat.count));
            }
            prompt.push_str("\nIf the content fits well into one of these categories, use it. Otherwise, create a new, relevant category.\n");
        }

        prompt.push_str("\nRequirements:\n");
        prompt.push_str("- Category must be 1-3 words maximum\n");
        prompt.push_str("- Use Title Case (capitalize first letters)\n");
        prompt.push_str("- Be specific and relevant to the content\n");
        prompt.push_str("- Return ONLY the category name, nothing else\n");

        prompt
    }
}
