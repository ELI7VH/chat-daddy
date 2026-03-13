# The Daddies Find Each Other — LAN Peer Sync

## The Multi-Machine Moment

Chat Daddy started as a single-machine tool. Then came the moment it became something more:

> "okkk has chat daddy been pushed at all since we started? let's make it it's own repo. repos/chat-daddy. push as a public repo, and we are going to continue on feature dev. next: I am going to clone on my other machine and the daddies are going to find eachother"

"The daddies are going to find each other." That's the line that turned a transcript viewer into a networked tool.

## The Design

> "a chat sync, but all live over the wire, no persistance of other machines yet. remove viewing, the chats should show up inline with a machine name beside the framework name, use the machine hostname .local as the name."

The requirements were precise:
- Live over the wire — no storing remote data
- Inline display — remote chats mixed with local, sorted by time
- Machine identity — hostname next to the platform name
- Zero config — instances just find each other

No central server. No cloud. No accounts. UDP broadcast on the LAN, TCP for data exchange. Two instances on the same network discover each other within 3 seconds.

## The Handoff Dream

> "chat handoff would be sick, but we gotta brainstorm it, because currently chat daddy has no way to send chats to other machines / platform chat instances, unless you know a way around that? can we just add a row to the jsonl? probalby not eh?"

The honest answer was no — you can't cleanly inject into Claude's JSONL or Cursor's SQLite. Each tool owns its own state. But the dream is there: start a conversation on one machine, hand it off to another, keep working.

The path forward is probably through LLM summarization — generate a structured handoff document, beam it to the other machine's clipboard, paste it into a new session. The summarization endpoint is already spec'd out (CodeQwen1.5-7B on port 1234). The infrastructure is waiting for the feature.

## How It Works

The implementation is dead simple:

**Discovery:** Every 3 seconds, each instance broadcasts a UDP beacon on port 21847: `{"h":"DESKTOP-ABC","p":54321,"v":"0.1.0"}`. Every instance listens for beacons. Skip your own. Track peers. Evict anyone who goes silent for 9 seconds.

**Data exchange:** Each instance runs a TCP server on a random port. Two commands: `LIST` returns all local chats as JSON. `GET <uuid>` returns the full message content. That's the entire protocol.

**Display:** Remote chats merge into the local list, sorted by modification time. The right side of each entry shows `HOSTNAME · claude` instead of just `claude`. Open a remote chat and it fetches the content over TCP. Navigate between local and remote chats with arrow keys.

**Ephemeral by design:** When a peer goes offline, its chats disappear from the list. No stale references. No orphaned data. The network state is always true.

## What This Enables

Two developers on the same network, both running Chat Daddy, can see each other's AI conversations in real time. Not sharing a screen — sharing the actual transcript data. Review each other's approaches. See what the other machine's AI is working on. All without leaving the terminal-style viewer.

The peer sync turns Chat Daddy from a personal tool into a collaborative one. Still no cloud. Still no accounts. Just machines on a LAN, talking to each other.
