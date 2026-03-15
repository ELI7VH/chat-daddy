# Chat Daddy — Backlog

## Upcoming

- **Background transcript scan** — move `get_all_transcripts()` off the main thread to eliminate input lag (especially on macOS where filesystem metadata calls are slower)
- **Push-based LAN updates** — peers notify each other on rename/star instead of polling every 5s, for instant name sync
- **Remote chat caching** — cache remote chats locally since they don't delete; use a "last index" watcher for incremental inserts instead of full LIST refetch
- **GroundControl API handoff** — configurable external endpoint for `dump` (initial state) and `update` (individual chats); create API spec for server-side ingestion

## Ideas

- Custom keybindings via config
- Export chat to markdown
- Mouse scroll support
- Multi-select bulk rename/star
