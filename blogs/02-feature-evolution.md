# Feature Evolution — Building in Public, One Frustration at a Time

## The Rapid Stack

Chat Daddy's feature set wasn't planned. It was demanded — by the person using it, in real time, while building it. Every feature came from a specific moment of need.

### Word Wrapping

> "can you make it so line breaks are natural? it seems like it's cut off on the right side. getting much closer."

The first thing you notice when rendering raw text to a pixel buffer: text doesn't wrap itself. You have to do it manually, character by character, respecting word boundaries.

### Search and Timestamps

> "add a header where we're going to put things. user presses S to enter a term and filter the chats. chats need timestamps"

Once you have more than a dozen chats, you need search. Once you have search, you need to know when things happened. Timestamps went from unix seconds to human-readable: "today 3:15 PM", "2 days ago", "Mar 8".

### Text Selection and Copy

> "i need to be able to highlight the text and copy paste from it. add a little gap between the header and chat, and add a line break between user / asst chunks"

A viewer that can't copy text is just a screenshot. Mouse-driven text selection with clipboard support made it actually useful for extracting information from old conversations.

### Collapsed Responses

> "ok, in the asst block, only show the last item in the group, before the next user submission. I want to zero in on just the meat of the conversation."

AI assistant responses are long. Tool calls, reasoning, code blocks — most of it is noise when you're scanning a conversation. Collapsed by default, expandable on click. Zero in on the meat.

### Markdown Rendering (Light)

> "the text should be formatted using the markdown available, but keep it light."

Not a full markdown renderer. Just enough to make code blocks readable, bold text visible, and headings distinguishable. Keep it light — the philosophy that runs through everything.

### Real-Time Updates

> "the chat should scroll to the latest message, and when a new chat is entered through one of the clients, chat-daddy should stay up to date in real time"

File mtime polling every second in view mode, transcript list refresh every 2 seconds. No filesystem watchers, no event systems — just polling. Simple, reliable, cross-platform.

### Multi-Platform Unified View

> "In the chat list it shows the C--Users-elija, but that tells us nothing, find a way to make it say 'claude' for claude chats, and we need a way to grab our cursor and antigravity chats in here. codex: ~/.codex/sessions, claude: ~/.claude/projects, cursor: ~/.cursor/projects. figure out the path on getting all the chats for the respective platforms, unified into this view."

This was the moment Chat Daddy went from "Claude transcript viewer" to "all your chats, one daddy." Three platforms, three different directory structures, three different JSONL formats — unified into a single sorted list.

> "should have some sort of mapping file, and also a place where a user can add some sort of config and their own mapping schema to add their own folders."

Config-driven sources. Users can add their own chat directories with custom formats.

### Configurable Theming

> "to the config, add 'font', 'font weight' colors: { }"

Full color palette in config.json. 18 color keys, hex values, font family and weight. The default theme is a dark blue terminal aesthetic, but every pixel color is overridable.

### LLM Auto-Naming

> "integrate this, tools/chat-daddy/LLAMA_ENDPOINTS.md - scans any chats without a name."

A local Qwen2.5-0.5B (500MB, ~40ms latency) running on llama.cpp auto-names unnamed chats every 10 seconds. First 2 + last 2 user messages, truncated to 200 chars each, sent to the model with a system prompt: "output ONLY a short title (2-6 words)."

The auto-naming idea came from a separate conversation about LM Studio configuration:

> "I currently have a program called 'chat daddy' and I want to be able to hand a chat context over to a model and auto-name the chats. find a model that is capable, figure out some params, create an md, give an edpoint for this functionality"

## The Pattern

Every feature followed the same pattern:
1. Use the tool
2. Hit a friction point
3. Fix it immediately
4. Move on

No backlog. No sprint planning. No tickets. Just build what you need when you need it.
