# AGENTS.md

Instructions for AI coding agents working with this repository.

## What This Is

Chat Daddy is a minimal, keyboard-driven chat transcript viewer for AI coding assistants (Claude Code, Cursor, Codex). Pixel-buffer rendered with minifb, no GPU. Rust only.

## One-Shot Setup Script

Copy-paste this entire block. It detects the platform, downloads the correct binary, handles the font requirement, and launches. **No decisions required.**

```bash
#!/usr/bin/env bash
set -euo pipefail

# ── 1. Determine platform and asset name ──
OS="$(uname -s)"
ARCH="$(uname -m)"
case "$OS-$ARCH" in
  Darwin-arm64)  ASSET="chat-daddy-macos-arm64" ;;
  Darwin-x86_64) ASSET="chat-daddy-macos-x64"   ;;
  Linux-x86_64)  ASSET="chat-daddy-linux"        ;;
  *)             echo "No prebuilt binary for $OS-$ARCH — fall back to cargo build"; ASSET="" ;;
esac

# ── 2. Install directory ──
INSTALL_DIR="$HOME/.local/bin"
mkdir -p "$INSTALL_DIR"

# ── 3. Download or build ──
if [ -n "$ASSET" ]; then
  if command -v gh &>/dev/null; then
    gh release download --repo ELI7VH/chat-daddy --pattern "$ASSET" --dir /tmp --clobber
  else
    curl -fSL -o "/tmp/$ASSET" "https://github.com/ELI7VH/chat-daddy/releases/latest/download/$ASSET"
  fi
  chmod +x "/tmp/$ASSET"
  mv "/tmp/$ASSET" "$INSTALL_DIR/chat-daddy"
else
  # Requires Rust toolchain
  REPO_DIR="${REPO_DIR:-$(mktemp -d)}"
  git clone https://github.com/ELI7VH/chat-daddy.git "$REPO_DIR/chat-daddy" 2>/dev/null || true
  cd "$REPO_DIR/chat-daddy"
  cargo build --release
  cp target/release/chat-daddy "$INSTALL_DIR/chat-daddy"
fi

# ── 4. Ensure ~/.local/bin is on PATH ──
if ! echo "$PATH" | tr ':' '\n' | grep -qx "$INSTALL_DIR"; then
  echo "export PATH=\"$INSTALL_DIR:\$PATH\"" >> "$HOME/.zshrc" 2>/dev/null \
    || echo "export PATH=\"$INSTALL_DIR:\$PATH\"" >> "$HOME/.bashrc" 2>/dev/null
  export PATH="$INSTALL_DIR:$PATH"
fi

# ── 5. Font check ──
# Chat Daddy needs a monospace TTF font. Default is Fira Code (weight 300).
# If missing, it falls back to whatever monospace font is available via config.
FONT_OK=false
if [ "$OS" = "Darwin" ]; then
  find ~/Library/Fonts /Library/Fonts /System/Library/Fonts -iname "*firacode*" -print -quit 2>/dev/null | grep -q . && FONT_OK=true
elif [ "$OS" = "Linux" ]; then
  fc-list 2>/dev/null | grep -qi "fira code" && FONT_OK=true
fi

if ! $FONT_OK; then
  echo "⚠ Fira Code font not found."
  echo "  Option A: Install it — https://github.com/tonsky/FiraCode/releases"
  echo "  Option B: Edit ~/.chat-daddy/config.json after first run and set"
  echo "            \"font\" to a monospace font you have (e.g. \"Menlo\", \"Consolas\", \"DejaVu Sans Mono\")"
fi

# ── 6. Launch ──
echo "chat-daddy installed at $INSTALL_DIR/chat-daddy"
chat-daddy &
```

For **Windows**, download `chat-daddy.exe` from the latest release and run it. Font detection and PATH setup are automatic on Windows.

## Manual Install (if the script above doesn't suit the situation)

### Binary Asset Names

Use **exact** asset names — do not guess patterns like `*darwin*` or `*aarch64*`.

| Platform              | Asset name                  |
|-----------------------|-----------------------------|
| macOS ARM64 (M1/M2+)  | `chat-daddy-macos-arm64`    |
| macOS x64 (Intel)      | `chat-daddy-macos-x64`     |
| Linux x86_64           | `chat-daddy-linux`         |
| Windows x86_64         | `chat-daddy.exe`           |

### Download Commands

Both commands always fetch the **latest** release — no version hardcoding.

```bash
# via gh CLI (recommended)
gh release download --repo ELI7VH/chat-daddy --pattern "ASSET" --dir .
chmod +x ASSET

# via curl
curl -fSL -o chat-daddy https://github.com/ELI7VH/chat-daddy/releases/latest/download/ASSET
chmod +x chat-daddy
```

Replace `ASSET` with the exact name from the table.

### Platform Detection (for agents)

```
uname -s  → Darwin  = macOS
uname -s  → Linux   = Linux
uname -m  → arm64   = ARM (Apple Silicon)
uname -m  → x86_64  = Intel/AMD
```

## Build from Source (Fallback)

Only needed if no prebuilt binary exists for the user's OS/arch.

```bash
git clone https://github.com/ELI7VH/chat-daddy.git
cd chat-daddy
cargo build --release
```

Binary output: `target/release/chat-daddy` (Linux/macOS) or `target/release/chat-daddy.exe` (Windows).

## Troubleshooting

| Symptom | Cause | Fix |
|---------|-------|-----|
| `error: font not found` or blank window | Fira Code not installed | Install Fira Code **or** edit `~/.chat-daddy/config.json` → set `"font"` to any installed monospace font |
| `Permission denied` | Binary not executable | `chmod +x chat-daddy` |
| `gh: command not found` | GitHub CLI not installed | Use the `curl` method instead |
| `cargo: command not found` | Rust not installed | Use the binary download method instead of building |
| Window opens but no chats | No transcript sources found | Check that `~/.claude/`, `~/.cursor/`, or `~/.codex/` exist with chat data |

## Run

```bash
chat-daddy              # if on PATH
./chat-daddy            # if in current directory
cargo run --release     # if building from source
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

## Key Concepts

- **Sources**: Chat transcript directories (Claude `~/.claude/projects`, Cursor `~/.cursor/agent-transcripts`, Codex `~/.codex/sessions`). Auto-detected on first run.
- **LAN Sync**: UDP broadcast on port 21847 for peer discovery, TCP on random port for chat data exchange. Peers appear inline in the list with `hostname · platform` labels.
- **LLM Auto-Naming**: Optional local Qwen2.5-0.5B on port 1235 for naming unnamed chats. Not required — app works fine without it.

## Commit Rules

- Co-author line: `Co-Authored-By: Ana Iliovic <ana@thevii.app> [<agent>, <model>, <provider>]`
- Push to `main` directly — no branches, no PRs for this project
