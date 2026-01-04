# Poetry Plugin

A sample RustPress plugin that adds Shakespearean sonnet lines to blog posts.

## Overview

This plugin demonstrates the RustPress plugin system by:

- Adding a line from Shakespeare's Sonnet 18 to the beginning of each published post
- Implementing the `filter_post_published` hook
- Requesting necessary permissions (`post:read`, `post:write`)

## Files

- `src/lib.rs` - Main plugin implementation (compiled to WebAssembly)
- `manifest.toml` - Plugin manifest with metadata and permissions
- `Cargo.toml` - Rust project configuration

## Building

To build this plugin into an RPK package:

```bash
# From the project root
cargo run --bin rpk-builder plugins/poetry-plugin poetry-plugin.rpk
```

This will:
1. Compile the plugin to WebAssembly (`plugin.wasm`)
2. Package it with the manifest into `poetry-plugin.rpk`

## Installation

Upload the `poetry-plugin.rpk` file through the RustPress admin panel under Plugins > Install Plugin.

## Permissions Required

- `post:read` - To read post content
- `post:write` - To modify post content

## Hooks Used

- `filter_post_published` - Modifies post content when published

## Example Output

Before:
```
This is my blog post content.
```

After:
```
> *Shall I compare thee to a summer's day?*

This is my blog post content.
```

The plugin selects different lines from the sonnet based on the post ID to provide variety.