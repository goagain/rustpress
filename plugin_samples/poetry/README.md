# poetry

A Rustpress Wasm plugin.

## Development

1. Build the plugin:
   ```bash
   cargo build --target wasm32-unknown-unknown --release
   ```

2. Install to your Rustpress instance:
   ```bash
   # Copy the .rpk file to your plugins directory
   cp target/wasm32-unknown-unknown/release/poetry.rpk ~/rustpress/plugins/
   ```

3. Restart your Rustpress server to load the plugin.

## Plugin Features

This plugin implements the `Guest` trait and provides an `on_post_published` hook.

## License

MIT