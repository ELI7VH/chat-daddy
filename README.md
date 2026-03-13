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

## For AI Agents — Quick Install

<!-- AGENT INSTRUCTIONS: Read this section if you are an AI coding agent setting up chat-daddy. -->

Download a prebuilt binary from GitHub Releases instead of building from source. Use the **exact asset names** below — do not guess with glob patterns.

### Binary Asset Names

| Platform              | Asset name                  |
|-----------------------|-----------------------------|
| macOS ARM64 (M1/M2+)  | `chat-daddy-macos-arm64`    |
| macOS x64 (Intel)      | `chat-daddy-macos-x64`     |
| Linux x86_64           | `chat-daddy-linux`         |
| Windows x86_64         | `chat-daddy.exe`           |

**Steps:**
1. Detect OS and arch (`uname -s` / `uname -m` on Unix, or check environment on Windows)
2. Download the matching binary using the exact asset name from the table — both methods always fetch the **latest** release:
   ```bash
   # via gh CLI (always fetches latest)
   gh release download --repo ELI7VH/chat-daddy --pattern "ASSET" --dir .
   chmod +x ASSET

   # via curl (GitHub /latest redirect resolves automatically)
   curl -L -o chat-daddy https://github.com/ELI7VH/chat-daddy/releases/latest/download/ASSET
   chmod +x chat-daddy
   ```
   Replace `ASSET` with the exact name from the table. Do not hardcode version tags.
3. Place it somewhere on PATH (e.g. `~/.local/bin/`)
4. If no binary exists for the platform, fall back to `cargo build --release`
5. Run it once — config auto-generates at `~/.chat-daddy/config.json`

Binaries are built automatically for **Windows x86_64**, **Linux x86_64**, **macOS ARM64**, and **macOS x64** on every tagged release.

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
