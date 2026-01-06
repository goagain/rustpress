use log::*;

mod logger;

wit_bindgen::generate!({
    world: "plugin-world",
    path: "./wit",
    pub_export_macro: true
});

use crate::exports::rustpress::plugin::event_handler::*;
use crate::rustpress::plugin::ai::*;
use crate::rustpress::plugin::posts::*;
struct AutoSummaryPlugin;

impl Guest for AutoSummaryPlugin {
    fn handle_filter(event: PluginFilterEvent) -> anyhow::Result<PluginFilterEvent, String> {
        logger::Logger::init();

        info!("Processing event for auto-summary plugin");
        match event {
            PluginFilterEvent::OnPostPublishedFilter(data) => {
                let processed_data = on_post_published(data).map_err(|e| e.to_string())?;
                Ok(PluginFilterEvent::OnPostPublishedFilter(processed_data))
            }
            _ => Ok(event),
        }
    }
    fn handle_action(event: PluginActionEvent) {
        logger::Logger::init();

        info!("Processing action for auto-summary plugin");
        // For now, just log the event
        match event {
            PluginActionEvent::Unknown => {
                // Do nothing for unknown events
            }
        }
    }
}

fn on_post_published(
    mut post: OnPostPublishedData,
) -> anyhow::Result<OnPostPublishedData, anyhow::Error> {
    // Skip if description is already set
    if let Some(description) = post.description.clone() {
        if !description.is_empty() {
            info!("Post already has description, skipping auto-summary");
            return Ok(post);
        }
    }

    info!("Generating AI summary for post: {}", post.title);

    // Generate summary using AI
    match generate_summary(&post) {
        Ok(summary) => {
            post.description = Some(summary.clone());
            info!("Generated summary: {}", summary);
        }
        Err(e) => {
            error!("Failed to generate summary: {}", e);
            // Return post unchanged if AI fails
            return Err(anyhow::anyhow!("Failed to generate summary: {}", e));
        }
    }

    match generate_category(&post) {
        Ok(category) => {
            info!("Generated category: {}", category);
            post.category = Some(category);
        }
        Err(e) => {
            error!("Failed to generate category: {}", e);
            return Err(anyhow::anyhow!("Failed to generate category: {}", e));
        }
    }
    Ok(post)
}

fn generate_summary(post: &OnPostPublishedData) -> anyhow::Result<String> {
    let system_prompt = "You are a helpful assistant that generates summaries of blog posts. 
        Your task is to generate a summary of the given blog post in less than 50 words.
        The summary should be in the same language as the blog post. If the language is not detected, use English.
        The summary should be concise and to the point.
        The summary should be in markdown format.

        Please only return the summary, no other text.
    ";

    let user_prompt = format!(
        "Title: {}\n\nContent: {}",
        post.title,
        post.content
    );

    // Create chat completion request
    let messages = vec![ChatMessage {
        role: "system".to_string(),
        content: system_prompt.to_string(),
    }, ChatMessage {
        role: "user".to_string(),
        content: user_prompt,
    }];

    // Call AI chat completion
    match chat_completion(&ChatCompletionRequest {
        model: None, // Use default model
        messages,
        max_tokens: None,
    }) {
        Ok(response) => {
            if let Some(choice) = response.choices.first() {
                let summary = choice.message.content.trim();
                info!("Generated summary: {}", summary);
                Ok(summary.to_string())
            } else {
                Err(anyhow::anyhow!("No response choices returned from AI"))
            }
        },
        Err(e) => Err(anyhow::anyhow!("AI chat completion failed: {}", e)),
    }
}

fn generate_category(post: &OnPostPublishedData) -> anyhow::Result<String> {
    if let Some(post) = post.category.clone() {
        info!("Post already has category, skipping auto-label");
        return Ok(post);
    }

    let categories = list_categories();

    let system_prompt = format!("You are a helpful assistant that generates labels for blog posts.
        Your task is to generate a label for the given blog post.
        The label should be a single word or phrase that describes the post.
        The label should be in the same language as the blog post. If the language is not detected, use English.
        The label should be concise and to the point.

        The following is a list of categories, ranked by frequency (categories appearing more often are listed first).
        Categories: {:?}.
        If the post is not related to any of the categories, generate a new category.
    ", categories.join(", "));

    let user_prompt = format!(
        "Title: {}\n\nContent: {}",
        post.title,
        post.content
    );

    let messages = vec![ChatMessage {
        role: "system".to_string(),
        content: system_prompt.to_string(),
    }, ChatMessage {
        role: "user".to_string(),
        content: user_prompt,
    }];

    match chat_completion(&ChatCompletionRequest {
        model: None, // Use default model
        messages,
        max_tokens: None,
    }) {
        Ok(response) => {
            if let Some(choice) = response.choices.first() {
                let category = choice.message.content.trim();
                info!("Generated category: {}", category);
                Ok(category.to_string())
            } else {
                Err(anyhow::anyhow!("No response choices returned from AI"))
            }
        },
        Err(e) => Err(anyhow::anyhow!("AI chat completion failed: {}", e)),
    }
}

export!(AutoSummaryPlugin);