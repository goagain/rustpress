//! Poetry Plugin - Adds Shakespearean sonnet lines to posts

// Include generated WIT bindings
include!("plugin_world.rs");

struct PoetryPlugin;

// Implement the generated Guest trait
impl exports::blog::system::hooks::Guest for PoetryPlugin {
    fn on_post_published(
        mut post: exports::blog::system::hooks::PostData,
    ) -> Result<exports::blog::system::hooks::PostData, String> {
        // Log that we're processing a post
        blog::system::logging::log_info("Processing post for poetry enhancement");

        // Get post ID to select different poetry lines
        let post_id = post.id as usize;

        // Select poetry line based on post ID
        let line_index = post_id % SONNET_LINES.len();
        let poetry_line = SONNET_LINES[line_index];

        blog::system::logging::log_debug(&format!("Adding poetry line: {}", poetry_line));

        // Modify content - prepend poetry line
        let new_content = format!("> *{}*\n\n{}", poetry_line, post.content);

        post.content = new_content;

        blog::system::logging::log_info("Successfully enhanced post with poetry");

        Ok(post)
    }
}

// Export the plugin
export!(PoetryPlugin);

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
