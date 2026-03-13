---
date: 2026-03-13
purpose: llama.cpp server endpoints for chat-daddy features
---

# Chat Daddy — Local LLM Endpoints

## Auto-Name Chats

**Endpoint:** `POST http://localhost:1235/v1/chat/completions`
**Model:** Qwen2.5-0.5B-Instruct Q8 (507MB, ~500 VRAM)
**Latency:** ~40ms total (instant for UX)

### Request

```json
{
  "model": "qwen2.5-0.5b",
  "messages": [
    {
      "role": "system",
      "content": "You are a chat naming assistant. Given a conversation, output ONLY a short title (2-6 words). No quotes, no explanation, no punctuation at the end."
    },
    {
      "role": "user",
      "content": "Name this conversation:\n\n<truncated chat transcript here>"
    }
  ],
  "max_tokens": 20,
  "temperature": 0.3
}
```

### Parameters

| Param | Value | Why |
|-------|-------|-----|
| temperature | 0.3 | Low creativity — we want consistent, descriptive names |
| max_tokens | 20 | Titles are 2-6 words, this caps runaway output |
| top_p | 0.9 | (optional) Slight nucleus sampling to avoid degenerate repeats |

### Transcript Truncation Strategy

The model has a 2048 token context. A chat transcript can be huge. Truncate before sending:

1. Take the **first 2 user messages** (establishes topic)
2. Take the **last 2 user messages** (captures where it ended up)
3. For each message, truncate to **200 chars**
4. Total prompt stays well under 1K tokens

```js
function buildNamingPrompt(messages) {
  const userMsgs = messages.filter(m => m.role === 'user')
  const selected = [
    ...userMsgs.slice(0, 2),
    ...userMsgs.slice(-2)
  ].filter((v, i, a) => a.indexOf(v) === i) // dedupe if < 4 msgs

  const transcript = selected
    .map(m => `User: ${m.content.slice(0, 200)}`)
    .join('\n')

  return `Name this conversation:\n\n${transcript}`
}
```

### Response

```json
{
  "choices": [{ "message": { "content": "Disk Cleanup and LLM Setup" } }]
}
```

Just read `choices[0].message.content` — that's the title.

---

## Handoff / Summarize Chat

**Endpoint:** `POST http://localhost:1234/v1/chat/completions`
**Model:** CodeQwen1.5-7B-Chat Q4_K_M (4.5GB)
**Use case:** Generate a structured summary of a chat for handoff to a new session

### Request

```json
{
  "model": "codeqwen",
  "messages": [
    {
      "role": "system",
      "content": "You are a technical summarizer. Given a chat transcript, produce a structured handoff summary in markdown with sections: ## What Was Done, ## Open Items, ## Key Decisions. Be concise and specific."
    },
    {
      "role": "user",
      "content": "<full or truncated transcript>"
    }
  ],
  "max_tokens": 512,
  "temperature": 0.2
}
```

### Parameters

| Param | Value | Why |
|-------|-------|-----|
| temperature | 0.2 | Very deterministic — summaries should be factual |
| max_tokens | 512 | Enough for a thorough handoff doc |

---

## Server Launch Commands

### Naming server (port 1235) — lightweight, always-on
```bash
D:\llama-cpp\llama-server.exe \
  -m "D:\.lmstudio\models\lmstudio-community\Qwen2.5-0.5B-Instruct-GGUF\Qwen2.5-0.5B-Instruct-Q8_0.gguf" \
  --port 1235 -ngl 99 -c 2048 -fa on
```
VRAM: ~825 MB

### Code/summarization server (port 1234) — heavier
```bash
D:\llama-cpp\llama-server.exe \
  -m "D:\.lmstudio\models\Qwen\CodeQwen1.5-7B-Chat-GGUF\codeqwen-1_5-7b-chat-q4_k_m.gguf" \
  --port 1234 -ngl 99 -c 8192 -fa on
```
VRAM: ~5 GB

### Both fit simultaneously on the 5060 Ti (16GB VRAM)
Combined: ~5.8 GB VRAM, leaving ~10 GB free.

---

## Health Check

```bash
curl http://localhost:1235/health  # naming
curl http://localhost:1234/health  # code/summarization
```

Both return `{"status":"ok"}` when ready.
