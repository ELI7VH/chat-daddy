# chat-daddy

Minimal terminal-style viewer for Claude/Cursor chat transcript JSONL. List chats from `~/.claude/projects`, open one with Enter, scroll with Up/Down/PageUp/PageDown, Escape to go back or quit.

**Env:** `CHAT_TRANSCRIPTS_ROOT` = root containing `projects` (default: `~/.claude`).

## Build

Pure Rust (minifb). No system libraries required.

```bash
cargo build --release
```

Run: `target/release/chat_daddy.exe` (Windows) or `target/release/chat-daddy` (Unix). Or `cargo run --release`.

## Controls

- **List:** Up/Down to select, Enter to open, Escape to quit.
- **View:** Up/Down/PageUp/PageDown to scroll, Escape to back to list.
