use std::hash::{Hash, Hasher};
use std::collections::hash_map::DefaultHasher;

use log::*;

mod logger;

wit_bindgen::generate!({
    world: "plugin-world",
    path: "./wit",
    pub_export_macro: true
});

use crate::exports::rustpress::plugin::event_handler::*;
struct PoetryPlugin;

impl Guest for PoetryPlugin {
    fn handle_filter(event: PluginFilterEvent) -> anyhow::Result<PluginFilterEvent, String> {
        crate::logger::init();

        info!("Processing post for poetry enhancement");
        match event {
            PluginFilterEvent::OnPostPublishedFilter(data) => {
                let processed_data = on_post_published(data).map_err(|e| e.to_string())?;
                Ok(PluginFilterEvent::OnPostPublishedFilter(processed_data))
            },
            _ => Ok(event),
        }
    }
    fn handle_action(event: PluginActionEvent) {
        crate::logger::init();

        info!("Processing action for poetry enhancement");
        // For now, just log the event
        match event {
            PluginActionEvent::Unknown => {
                // Do nothing for unknown events
            }
        }
    }
}
fn on_post_published(mut post: OnPostPublishedData) -> anyhow::Result<OnPostPublishedData, anyhow::Error> {
    info!("Processing post for poetry enhancement");

    let mut hasher = DefaultHasher::new();
    post.title.to_string().hash(&mut hasher);

    let hash = hasher.finish() as usize;
    let line_index = hash % SONNET_LINES.len();
    let poetry_line = SONNET_LINES[line_index];
    let new_content = format!("> *{}*<br/><br/>{}", poetry_line, post.content);
    post.content = new_content;

    Ok(post)
}
// Static sonnet lines
static SONNET_LINES: &[&str] = &[
    "Shall I compare thee to a summer's day?",
    "Thou art more lovely and more temperate:",
    "Rough winds do shake the darling buds of May,",
    "And summer's lease hath all too short a date;",
    "Sometimes too hot the eye of heaven shines,",
    "And often is his gold complexion dimmed;",
    "And every fair from fair sometime declines,",
    "By chance or nature's changing course untrimmed;",
    "But thy eternal summer shall not fade,",
    "Nor lose possession of that fair thou ow'st;",
    "Nor shall death brag thou wander'st in his shade,",
    "When in eternal lines to time thou grow'st:",
    "So long as men can breathe or eyes can see,",
    "So long lives this, and this gives life to thee.",
];

export!(PoetryPlugin);
