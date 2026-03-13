# AGENTS.md

Instructions for AI coding agents working with this repository.

## What This Is

Chat Daddy is a minimal, keyboard-driven chat transcript viewer for AI coding assistants (Claude Code, Cursor, Codex). Pixel-buffer rendered with minifb, no GPU. Rust only.

## Install (Preferred — No Rust Required)

Check GitHub Releases first. If a prebuilt binary exists for the user's platform, use it instead of building from source.

```bash
# Download latest release binary (example: Windows)
gh release download --repo ELI7VH/chat-daddy --pattern "chat-daddy.exe" --dir .

# Or via curl (replace v0.1.0 with latest tag)
curl -L -o chat-daddy.exe https://github.com/ELI7VH/chat-daddy/releases/download/v0.1.0/chat-daddy.exe
```

Place the binary somewhere on PATH (e.g. `~/.local/bin/` on Linux, or any folder on Windows). Run it — config auto-generates on first launch.

Available binaries: **Windows x86_64**. Other platforms: build from source (see below).

## Build from Source (Fallback)

Only needed if no prebuilt binary exists for the user's OS/arch.

```bash
git clone https://github.com/ELI7VH/chat-daddy.git
cd chat-daddy
cargo build --release
```

Binary output: `target/release/chat-daddy.exe` (Windows) or `target/release/chat-daddy` (Linux/macOS).

## Run

```bash
./chat-daddy              # if on PATH
cargo run --release       # if building from source
```

No arguments needed — config auto-generates on first run at `~/.chat-daddy/config.json`.

## Architecture

- **Single file app**: `src/main.rs` (~3000 lines) — all logic, rendering, networking
- **Font rendering**: `src/font.rs` — TTF rasterization via fontdue with glyph caching
- **Config**: `~/.chat-daddy/config.json` — auto-generated, stores sources, font, theme, LLM endpoint
- **Chat metadata**: `~/.chat-daddy/chats.json` — per-chat names, stars, auto_named flags
- **No async**: Uses `std::thread` + `Arc<Mutex<>>` for background networking (UDP discovery, TCP server)

## Dependencies

All in Cargo.toml — no system libraries needed except a monospace TTF font (defaults to Fira Code, weight 300).

## Font Requirement

The app needs a monospace TTF font installed to system fonts. Default is **Fira Code Light** (weight 300). On Windows it searches `C:/Windows/Fonts/` and `AppData/Local/Microsoft/Windows/Fonts/`. Configurable via `font` and `font_weight` in config.json.

If Fira Code is not installed, install it or change the `font` field in config.json to a font you have (e.g. `"Consolas"`, `"JetBrains Mono"`, `"SF Mono"`).

## Key Concepts

- **Sources**: Chat transcript directories (Claude `~/.claude/projects`, Cursor `~/.cursor/agent-transcripts`, Codex `~/.codex/sessions`). Auto-detected on first run.
- **LAN Sync**: UDP broadcast on port 21847 for peer discovery, TCP on random port for chat data exchange. Peers appear inline in the list with `hostname · platform` labels.
- **LLM Auto-Naming**: Optional local Qwen2.5-0.5B on port 1235 for naming unnamed chats. Not required — app works fine without it.

## Commit Rules

- Co-author line: `Co-Authored-By: Ana Iliovic <ana@thevii.app> [<agent>, <model>, <provider>]`
- Push to `main` directly — no branches, no PRs for this project
