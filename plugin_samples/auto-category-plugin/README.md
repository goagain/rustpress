# Auto Category Plugin

An intelligent plugin for RustPress that automatically generates relevant categories for blog posts using AI.

## Features

- **Automatic Categorization**: Uses AI to analyze post content and generate appropriate categories
- **Smart Category Selection**: Considers existing categories and their usage frequency
- **Fallback Handling**: Creates new categories when content doesn't fit existing ones
- **Non-Intrusive**: Only processes posts without existing categories

## How It Works

1. When a post is published without a category, the plugin intercepts it
2. Analyzes the post title and content using AI
3. Considers existing categories and their post counts
4. Generates or selects the most appropriate category
5. Automatically assigns the category to the post

## AI Prompt Strategy

The plugin uses a carefully crafted prompt that:
- Analyzes both title and content
- Considers existing category distribution
- Generates categories limited to 1-3 words
- Uses proper Title Case formatting
- Ensures relevance and specificity

## Permissions Required

- `post:read` - To access post content for analysis
- `post:write` - To modify post category
- `post:list_category` - To access existing category statistics
- `ai:chat` - To perform AI-powered categorization

## Installation

1. Build the plugin:
   ```bash
   cd plugin_samples/auto-category-plugin
   cargo build --release --target wasm32-unknown-unknown
   ```

2. Package the plugin using rpk-builder

3. Upload and enable the plugin in RustPress admin panel

4. Configure OpenAI API key with `ai:chat` permission granted to this plugin

## Example Behavior

**Input Post:**
```
Title: Getting Started with Rust Web Development
Content: Learn how to build web applications using the Axum framework...
```

**Generated Category:** "Rust Tutorial"

**Input Post (with existing categories):**
```
Existing: Technology (15), Tutorial (8), News (12)
Title: Breaking: Major Security Update Released
Content: A critical security patch has been released for...
```

**Generated Category:** "News" (selected from existing categories)