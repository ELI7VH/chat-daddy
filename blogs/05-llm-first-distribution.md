# LLM-First Distribution — Software That Installs Itself

## The Setup Instruction

> "ok and add to the readme: Instructions: Tell claude to clone it, build it, and set it up. and make sure the codebase is ready for those instructions."

This is a new distribution model. The README doesn't just document how to install the software — it's written so that an AI agent can read it and execute the setup autonomously.

The AGENTS.md file (deliberately not CLAUDE.md, because this isn't platform-locked):

> "make an AGENT file, it shouldn't be claude only"
> "ill never use anyone else bb, but we gotta do what's right for everyone"

## Binary-First, Build-Second

> "then add an instruction for llms to download it if their system supports it, otherwise buiild it. put it in the readme, some section for LLMS to look in the releases and do the thing."

The install flow is optimized for AI agents:
1. Check GitHub Releases for a prebuilt binary matching the platform
2. If found: download it, make executable, done
3. If not: clone, cargo build, done
4. Run it — config auto-generates

No human needs to read installation docs. No human needs to debug build issues. The AI agent reads AGENTS.md, determines the platform, picks the right strategy, and executes.

## CI for Every Platform

GitHub Actions builds Windows, Linux, macOS ARM64, and macOS x64 binaries on every tagged release. Push a tag, get four binaries. Free for public repos.

> "will the CI cost me money in the actions?"

No. Public repos get unlimited CI minutes on standard runners. The entire cross-platform build pipeline costs nothing.

> "sick make it so"

## What This Means

Software distribution is shifting. The target audience for a README is no longer just humans — it's AI agents that will read the instructions and execute them. AGENTS.md is the new install script.

The setup instruction for Chat Daddy is literally: "Tell your AI coding agent to clone it, build it, and set it up." That's the entire user-facing documentation for installation. Everything else is machine-readable.

This only works when:
- The software is self-configuring (config auto-generates)
- The build is deterministic (Rust + Cargo, no system deps beyond a font)
- The binary is portable (no installer, no runtime)
- The instructions are unambiguous (step-by-step, platform-aware)

Chat Daddy checks all four boxes. It's software designed from the ground up to be installed by an AI.
