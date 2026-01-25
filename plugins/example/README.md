# Example Plugin

This is an example plugin for Robsidian.

## Building

To build this plugin, you need to have Rust with the `wasm32-unknown-unknown` target installed:

```bash
rustup target add wasm32-unknown-unknown
cargo build --target wasm32-unknown-unknown --release
```

## Plugin API

Plugins can:
- React to document open/save/close events
- Register custom commands
- Access the vault filesystem (with permission)

See the main Robsidian documentation for more details on the plugin API.
