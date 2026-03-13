# The Origin Story — How Chat Daddy Was Born

## The Problem

It started with a crash. Claude's GUI was eating transcripts, and there was no way to get them back.

> "ok ddue. claude is crashing so much, and im losing the chats in the gui. make me a view that watches all the chat jsonl files, and I can peep the convos anytime I want. just a minimal terminal style look"

That's it. That's the whole origin. Not a roadmap, not a PRD — just frustration and a need to see your own data. The chat files were already sitting on disk as JSONL. Every conversation with Claude, Cursor, Codex — all there, all readable, all invisible behind a GUI that kept crashing.

## The Name

> "call this 'chat-daddy'"

No deliberation. No branding exercise. Just a name that stuck.

## Going Native

The first instinct was Rust + SDL2:

> "can you actually do this in rust and sdl2?"

SDL2 had build issues, so it pivoted to minifb — pure Rust, no system dependencies, pixel-buffer rendering. The philosophy was clear from the start: no frameworks, no Electron, no GPU. Just pixels on screen.

This wasn't an isolated thought. Across multiple conversations, the same anti-bloat sentiment kept surfacing:

> "I have been thinking a lot about all these fucking electron apps that I have running, and now wiht AI, I am questioning it, like I can just build some bare metal bespoke shit"

> "I think we gotta go native with this. I want this to be cross platform, my last experiments with electron just introuced all the annoyances with web."

## The First Session

Chat Daddy was born in a Cursor session — not even a Claude session. The irony of building a chat viewer inside one of the chat clients it would eventually monitor. Within that first session, the app went from nothing to a working window displaying JSONL transcripts.

> "okkk it looks like we'regetting somewhere"

That first visual — text on screen, raw chat data rendered in a terminal-style view — was the proof of concept. Everything after was refinement.

## What Made It Different

Most developer tools are built top-down: design system, plan features, build incrementally. Chat Daddy was built bottom-up: see your data, make it readable, add what you need when you need it. Every feature came from a real moment of "I need this right now."

The data was always there. The chats were always on disk. Someone just had to build the viewer.
