use log::*;

mod logger;

wit_bindgen::generate!({
    world: "plugin-world",
    path: "./wit",
    pub_export_macro: true
});

use crate::rustpress::plugin::ai::*;
use crate::exports::rustpress::plugin::event_handler::*;
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
    if post.description.is_some() {
        info!("Post already has description, skipping auto-summary");
        return Ok(post);
    }

    info!("Generating AI summary for post: {}", post.title);

    // Generate summary using AI
    match generate_summary(&post) {
        Ok(summary) => {
            post.description = Some(summary.clone());
            info!("Generated summary: {}", summary);
            Ok(post)
        }
        Err(e) => {
            error!("Failed to generate summary: {}", e);
            // Return post unchanged if AI fails
            Ok(post)
        }
    }
}

fn generate_summary(post: &OnPostPublishedData) -> anyhow::Result<String> {
    // Create the AI prompt in English
    let prompt = format!(
        "Please provide a concise summary of this blog post in less than 50 words.\n\nTitle: {}\n\nContent: {}\n\nSummary:",
        post.title,
        // Limit content to first 1000 characters to avoid token limits
        if post.content.len() > 1000 {
            format!("{}...", &post.content[..1000])
        } else {
            post.content.clone()
        }
    );

    // Create chat completion request
    let messages = vec![ChatMessage {
        role: "user".to_string(),
        content: prompt,
    }];

    let request = ChatCompletionRequest {
        model: None, // Use default model
        messages,
        max_tokens: Some(100), // Limit response length
    };

    // Call AI chat completion
    match chat_completion(&request) {
        Ok(response) => {
            if let Some(choice) = response.choices.first() {
                let summary = choice.message.content.trim();

                // Ensure summary is under 50 words
                let word_count = summary.split_whitespace().count();
                if word_count > 50 {
                    // Truncate to first 50 words
                    let words: Vec<&str> = summary.split_whitespace().take(50).collect();
                    Ok(format!("{}...", words.join(" ")))
                } else {
                    Ok(summary.to_string())
                }
            } else {
                Err(anyhow::anyhow!("No response choices returned from AI"))
            }
        }
        Err(e) => Err(anyhow::anyhow!("AI chat completion failed: {}", e)),
    }
}

export!(AutoSummaryPlugin);
