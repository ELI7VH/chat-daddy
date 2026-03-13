<p align="center">
  <img src="assets/banner.png" alt="Chat Daddy — All Your Chats, One Daddy" width="100%" />
</p>

<p align="center">
  <img src="assets/icon.png" alt="Chat Daddy" width="120" />
</p>

# Chat Daddy

Minimal, keyboard-driven chat transcript viewer for AI coding assistants. Pixel-buffer rendered with zero GPU dependencies. Reads transcripts from Claude Code, Cursor, and Codex — all in one place.

## Features

- **Multi-source** — Claude Code, Cursor, Codex transcripts unified in a single list
- **LLM auto-naming** — local Qwen2.5-0.5B names unnamed chats automatically
- **Configurable theme** — full color palette, font family, and weight via `~/.chat-daddy/config.json`
- **Keyboard-driven** — search, star, rename, navigate between chats without touching the mouse
- **LAN peer sync** — instances on different machines discover each other and show remote chats inline

## Setup

> Tell your AI coding agent to clone it, build it, and set it up. It will read `AGENTS.md` and handle the rest.

```
git clone https://github.com/ELI7VH/chat-daddy.git
cd chat-daddy
cargo build --release
./target/release/chat-daddy      # Linux/macOS
target\release\chat-daddy.exe    # Windows
```

**Requirements:**
- Rust toolchain (`rustup` / `cargo`)
- A monospace TTF font installed (defaults to [Fira Code](https://github.com/tonsky/FiraCode) Light — configurable in `~/.chat-daddy/config.json`)

Config, sources, and chat metadata auto-generate on first run. No manual setup needed.

## Controls

Press `?` in-app for the full hotkey overlay.

### List View
| Key | Action |
|-----|--------|
| Up / Down | Select chat |
| Enter | Open chat |
| F | Star / unstar |
| Shift+F | Toggle favorites filter |
| / | Search |
| N | Rename chat |
| ? | Help overlay |
| Escape | Quit |

### Chat View
| Key | Action |
|-----|--------|
| Up / Down | Scroll |
| PageUp / PageDown | Fast scroll |
| Left / Right | Previous / next chat |
| Space | Expand / collapse message group |
| C | Copy selected text |
| ? | Help overlay |
| Escape | Back to list |

## Config

Stored at `~/.chat-daddy/config.json`. Auto-generated on first run with detected sources.

```json
{
  "font": "Fira Code",
  "font_weight": 300,
  "llm_endpoint": "http://localhost:1235",
  "colors": {
    "bg": "#0d1117",
    "text": "#c9d1d9",
    "user": "#58c4dc",
    "assistant": "#e6b34d"
  },
  "sources": [
    { "name": "claude", "format": "claude", "root": "~/.claude", "layout": "projects" },
    { "name": "cursor", "format": "cursor", "root": "~/.cursor", "layout": "agent-transcripts" },
    { "name": "codex",  "format": "codex",  "root": "~/.codex",  "layout": "sessions" }
  ]
}
```

See the full color key list in [config defaults](src/main.rs).

## LLM Auto-Naming

Requires a local llama.cpp server running Qwen2.5-0.5B on port 1235. See [LLAMA_ENDPOINTS.md](LLAMA_ENDPOINTS.md) for setup.

## License

MIT
