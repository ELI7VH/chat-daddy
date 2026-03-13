//! chat-daddy — unified chat transcript viewer for Claude, Cursor, Codex, and custom sources.
//! Config: ~/.chat-daddy/config.json (auto-generated on first run)

mod font;

use arboard::Clipboard;
use chrono::{Local, TimeZone};
use font::FontAtlas;
use image::{save_buffer, ColorType};
use minifb::{Key, KeyRepeat, MouseButton, MouseMode, Window, WindowOptions};
use serde_json::Value;
use std::collections::{HashMap, HashSet};
use std::env;
use std::fs;
use std::io::{BufRead, BufReader, Read as IoRead, Write as IoWrite};
use std::net::{TcpListener, TcpStream, SocketAddr, UdpSocket};
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Instant;

const WIN_W: usize = 1000;
const WIN_H: usize = 700;
const PAD: i32 = 12;

const FONT_SIZES: [f32; 4] = [11.0, 14.0, 18.0, 24.0];
const DEFAULT_SIZE_IDX: usize = 1; // 14px

// --- default colors (used as fallbacks, overridable via config) ---
// These constants are used by helper functions (parse_inline_markdown, etc.)
// and get shadowed by theme values inside main()'s render loop.

const COL_BG: (u8, u8, u8) = (0x0d, 0x11, 0x17);
const COL_DIM: (u8, u8, u8) = (0x58, 0x58, 0x5a);
const COL_USER: (u8, u8, u8) = (0x58, 0xc4, 0xdc);
const COL_ASST: (u8, u8, u8) = (0xe6, 0xb3, 0x4d);
const COL_TEXT: (u8, u8, u8) = (0xc9, 0xd1, 0xd9);
const COL_SEL: (u8, u8, u8) = (0x21, 0x3d, 0x5a);
const COL_SEP: (u8, u8, u8) = (0x1a, 0x1e, 0x26);
const COL_HEADER_BG: (u8, u8, u8) = (0x14, 0x1a, 0x22);
const COL_ACCENT: (u8, u8, u8) = (0x58, 0xc4, 0xdc);
const COL_SEARCH_BG: (u8, u8, u8) = (0x1a, 0x24, 0x30);
const COL_TIMESTAMP: (u8, u8, u8) = (0x44, 0x6e, 0x7a);
const COL_SELECT_BG: (u8, u8, u8) = (0x26, 0x4f, 0x78);
const COL_CODE: (u8, u8, u8) = (0x9d, 0xc8, 0x9d);
const COL_CODE_BG: (u8, u8, u8) = (0x12, 0x1a, 0x14);
const COL_BOLD: (u8, u8, u8) = (0xf0, 0xf4, 0xf8);
const COL_TOGGLE: (u8, u8, u8) = (0x44, 0x88, 0xaa);
const COL_HEADING: (u8, u8, u8) = (0x6a, 0xd0, 0xe8);
const COL_MSG_TIME: (u8, u8, u8) = (0x3a, 0x5a, 0x66);

#[derive(Clone)]
struct Theme {
    bg: (u8, u8, u8),
    dim: (u8, u8, u8),
    user: (u8, u8, u8),
    asst: (u8, u8, u8),
    text: (u8, u8, u8),
    sel: (u8, u8, u8),
    sep: (u8, u8, u8),
    header_bg: (u8, u8, u8),
    accent: (u8, u8, u8),
    search_bg: (u8, u8, u8),
    timestamp: (u8, u8, u8),
    select_bg: (u8, u8, u8),
    code: (u8, u8, u8),
    code_bg: (u8, u8, u8),
    bold: (u8, u8, u8),
    toggle: (u8, u8, u8),
    heading: (u8, u8, u8),
    msg_time: (u8, u8, u8),
}

impl Default for Theme {
    fn default() -> Self {
        Theme {
            bg: COL_BG,
            dim: COL_DIM,
            user: COL_USER,
            asst: COL_ASST,
            text: COL_TEXT,
            sel: COL_SEL,
            sep: COL_SEP,
            header_bg: COL_HEADER_BG,
            accent: COL_ACCENT,
            search_bg: COL_SEARCH_BG,
            timestamp: COL_TIMESTAMP,
            select_bg: COL_SELECT_BG,
            code: COL_CODE,
            code_bg: COL_CODE_BG,
            bold: COL_BOLD,
            toggle: COL_TOGGLE,
            heading: COL_HEADING,
            msg_time: COL_MSG_TIME,
        }
    }
}

/// Parse "#RRGGBB" or "RRGGBB" hex string to (r, g, b)
fn parse_hex_color(s: &str) -> Option<(u8, u8, u8)> {
    let hex = s.strip_prefix('#').unwrap_or(s);
    if hex.len() != 6 {
        return None;
    }
    let r = u8::from_str_radix(&hex[0..2], 16).ok()?;
    let g = u8::from_str_radix(&hex[2..4], 16).ok()?;
    let b = u8::from_str_radix(&hex[4..6], 16).ok()?;
    Some((r, g, b))
}

fn color_to_hex(c: (u8, u8, u8)) -> String {
    format!("#{:02x}{:02x}{:02x}", c.0, c.1, c.2)
}

fn load_theme_from_value(val: &Value) -> Theme {
    let mut t = Theme::default();
    if let Some(colors) = val.get("colors").and_then(|c| c.as_object()) {
        macro_rules! col {
            ($key:expr, $field:ident) => {
                if let Some(s) = colors.get($key).and_then(|v| v.as_str()) {
                    if let Some(c) = parse_hex_color(s) {
                        t.$field = c;
                    }
                }
            };
        }
        col!("bg", bg);
        col!("dim", dim);
        col!("user", user);
        col!("assistant", asst);
        col!("text", text);
        col!("selection", sel);
        col!("separator", sep);
        col!("header_bg", header_bg);
        col!("accent", accent);
        col!("search_bg", search_bg);
        col!("timestamp", timestamp);
        col!("select_bg", select_bg);
        col!("code", code);
        col!("code_bg", code_bg);
        col!("bold", bold);
        col!("toggle", toggle);
        col!("heading", heading);
        col!("msg_time", msg_time);
    }
    t
}

fn theme_to_json(t: &Theme) -> Value {
    serde_json::json!({
        "bg": color_to_hex(t.bg),
        "dim": color_to_hex(t.dim),
        "user": color_to_hex(t.user),
        "assistant": color_to_hex(t.asst),
        "text": color_to_hex(t.text),
        "selection": color_to_hex(t.sel),
        "separator": color_to_hex(t.sep),
        "header_bg": color_to_hex(t.header_bg),
        "accent": color_to_hex(t.accent),
        "search_bg": color_to_hex(t.search_bg),
        "timestamp": color_to_hex(t.timestamp),
        "select_bg": color_to_hex(t.select_bg),
        "code": color_to_hex(t.code),
        "code_bg": color_to_hex(t.code_bg),
        "bold": color_to_hex(t.bold),
        "toggle": color_to_hex(t.toggle),
        "heading": color_to_hex(t.heading),
        "msg_time": color_to_hex(t.msg_time),
    })
}

fn rgb(r: u8, g: u8, b: u8) -> u32 {
    ((r as u32) << 16) | ((g as u32) << 8) | (b as u32)
}

fn home_dir() -> PathBuf {
    let home = env::var("USERPROFILE")
        .or_else(|_| env::var("HOME"))
        .unwrap_or_else(|_| ".".into());
    PathBuf::from(home)
}

fn chat_daddy_dir() -> PathBuf {
    let dir = home_dir().join(".chat-daddy");
    if !dir.exists() {
        let _ = fs::create_dir_all(&dir);
    }
    dir
}

fn config_path() -> PathBuf {
    chat_daddy_dir().join("config.json")
}

fn favorites_path() -> PathBuf {
    chat_daddy_dir().join("favorites.json")
}

fn names_path() -> PathBuf {
    chat_daddy_dir().join("names.json")
}

// --- source config ---

#[derive(Clone, Debug, PartialEq)]
enum SourceFormat {
    Claude,  // event-based: type/message.role/message.content, queue-operation
    Cursor,  // simple: role/message.content per line, <user_query> wrappers
    Codex,   // session_meta + response_item events with payload wrapper
    Generic, // try Claude-style, fall back to Cursor-style
}

#[derive(Clone)]
struct SourceConfig {
    name: String,       // display name: "claude", "cursor", "codex"
    format: SourceFormat,
    root: PathBuf,      // base directory to scan
    /// How to find JSONL files within root:
    /// "projects" — root/projects/<dir>/UUID.jsonl (Claude-style)
    /// "agent-transcripts" — root/projects/<dir>/agent-transcripts/UUID/UUID.jsonl (Cursor)
    /// "sessions" — root/sessions/**/*.jsonl (Codex date-tree)
    /// "flat" — root/**/*.jsonl (generic recursive)
    layout: String,
}

fn default_sources() -> Vec<SourceConfig> {
    let home = home_dir();
    let mut sources = Vec::new();

    // Claude
    let claude_root = home.join(".claude");
    if claude_root.join("projects").exists() {
        sources.push(SourceConfig {
            name: "claude".into(),
            format: SourceFormat::Claude,
            root: claude_root,
            layout: "projects".into(),
        });
    }

    // Cursor
    let cursor_root = home.join(".cursor-server");
    let cursor_root2 = home.join(".cursor");
    let cursor_base = if cursor_root.join("projects").exists() {
        Some(cursor_root)
    } else if cursor_root2.join("projects").exists() {
        Some(cursor_root2)
    } else {
        None
    };
    if let Some(cr) = cursor_base {
        sources.push(SourceConfig {
            name: "cursor".into(),
            format: SourceFormat::Cursor,
            root: cr,
            layout: "agent-transcripts".into(),
        });
    }

    // Codex
    let codex_root = home.join(".codex");
    if codex_root.join("sessions").exists() {
        sources.push(SourceConfig {
            name: "codex".into(),
            format: SourceFormat::Codex,
            root: codex_root,
            layout: "sessions".into(),
        });
    }

    sources
}

struct AppConfig {
    sources: Vec<SourceConfig>,
    font: String,       // font family name, e.g. "Fira Code"
    font_weight: u16,   // 300 = light, 400 = regular, 500 = medium
    theme: Theme,
    llm_endpoint: String, // local LLM for auto-naming, e.g. "http://localhost:1235"
}

fn parse_sources(val: &Value) -> Vec<SourceConfig> {
    let Some(arr) = val.get("sources").and_then(|s| s.as_array()) else {
        return vec![];
    };
    let mut sources = Vec::new();
    for item in arr {
        let name = item
            .get("name")
            .and_then(|n| n.as_str())
            .unwrap_or("custom")
            .to_string();
        let format_str = item
            .get("format")
            .and_then(|f| f.as_str())
            .unwrap_or("generic");
        let format = match format_str {
            "claude" => SourceFormat::Claude,
            "cursor" => SourceFormat::Cursor,
            "codex" => SourceFormat::Codex,
            _ => SourceFormat::Generic,
        };
        let root_str = item
            .get("root")
            .and_then(|r| r.as_str())
            .unwrap_or("");
        let root = if root_str.starts_with("~/") || root_str.starts_with("~\\") {
            home_dir().join(&root_str[2..])
        } else {
            PathBuf::from(root_str)
        };
        let layout = item
            .get("layout")
            .and_then(|l| l.as_str())
            .unwrap_or("flat")
            .to_string();
        sources.push(SourceConfig {
            name,
            format,
            root,
            layout,
        });
    }
    sources
}

fn load_config() -> AppConfig {
    let path = config_path();

    // migrate from old ~/.claude/ location if new path doesn't exist
    if !path.exists() {
        let old = home_dir().join(".claude").join("chat-daddy.json");
        if old.exists() {
            let _ = fs::copy(&old, &path);
        }
        let old_favs = home_dir().join(".claude").join("chat-daddy-favorites.json");
        let new_favs = favorites_path();
        if old_favs.exists() && !new_favs.exists() {
            let _ = fs::copy(&old_favs, &new_favs);
        }
    }

    if let Ok(data) = fs::read_to_string(&path) {
        if let Ok(val) = serde_json::from_str::<Value>(&data) {
            let sources = parse_sources(&val);
            let font = val
                .get("font")
                .and_then(|f| f.as_str())
                .unwrap_or("Fira Code")
                .to_string();
            let font_weight = val
                .get("font_weight")
                .and_then(|w| w.as_u64())
                .unwrap_or(300) as u16;
            let theme = load_theme_from_value(&val);
            let llm_endpoint = val
                .get("llm_endpoint")
                .and_then(|e| e.as_str())
                .unwrap_or("http://localhost:1235")
                .to_string();

            if !sources.is_empty() {
                return AppConfig {
                    sources,
                    font,
                    font_weight,
                    theme,
                    llm_endpoint,
                };
            }
        }
    }

    // auto-detect defaults and write config
    let sources = default_sources();
    let cfg = AppConfig {
        sources,
        font: "Fira Code".into(),
        font_weight: 300,
        theme: Theme::default(),
        llm_endpoint: "http://localhost:1235".into(),
    };
    save_config(&cfg);
    cfg
}

fn sources_to_json(sources: &[SourceConfig]) -> Vec<Value> {
    sources
        .iter()
        .map(|s| {
            let format_str = match s.format {
                SourceFormat::Claude => "claude",
                SourceFormat::Cursor => "cursor",
                SourceFormat::Codex => "codex",
                SourceFormat::Generic => "generic",
            };
            let home = home_dir();
            let root_str = if let Ok(rel) = s.root.strip_prefix(&home) {
                format!("~/{}", rel.to_string_lossy().replace('\\', "/"))
            } else {
                s.root.to_string_lossy().into_owned()
            };
            serde_json::json!({
                "name": s.name,
                "format": format_str,
                "root": root_str,
                "layout": s.layout,
            })
        })
        .collect()
}

fn save_config(cfg: &AppConfig) {
    let config = serde_json::json!({
        "font": cfg.font,
        "font_weight": cfg.font_weight,
        "llm_endpoint": cfg.llm_endpoint,
        "colors": theme_to_json(&cfg.theme),
        "sources": sources_to_json(&cfg.sources),
    });
    if let Ok(json) = serde_json::to_string_pretty(&config) {
        let _ = fs::write(config_path(), json);
    }
}

// --- per-chat metadata (persisted in chats.json) ---

#[derive(Clone, Default)]
struct ChatMeta {
    name: Option<String>,   // user-given or LLM-generated name
    starred: bool,
    auto_named: bool,       // true if name came from LLM (user rename clears this)
}

fn chats_path() -> PathBuf {
    chat_daddy_dir().join("chats.json")
}

fn load_chats() -> HashMap<String, ChatMeta> {
    let path = chats_path();

    // migrate from old separate files if chats.json doesn't exist
    if !path.exists() {
        let mut map = HashMap::new();

        // migrate favorites
        let fav_path = favorites_path();
        if let Ok(data) = fs::read_to_string(&fav_path) {
            if let Ok(arr) = serde_json::from_str::<Vec<String>>(&data) {
                for uuid in arr {
                    map.entry(uuid).or_insert_with(ChatMeta::default).starred = true;
                }
            }
        }

        // migrate names
        let names_p = names_path();
        if let Ok(data) = fs::read_to_string(&names_p) {
            if let Ok(names) = serde_json::from_str::<HashMap<String, String>>(&data) {
                for (uuid, name) in names {
                    let meta = map.entry(uuid).or_insert_with(ChatMeta::default);
                    meta.name = Some(name);
                }
            }
        }

        if !map.is_empty() {
            save_chats(&map);
        }
        return map;
    }

    let Ok(data) = fs::read_to_string(&path) else {
        return HashMap::new();
    };
    let Ok(val) = serde_json::from_str::<Value>(&data) else {
        return HashMap::new();
    };
    let Some(obj) = val.as_object() else {
        return HashMap::new();
    };
    let mut map = HashMap::new();
    for (uuid, meta_val) in obj {
        let meta = ChatMeta {
            name: meta_val.get("name").and_then(|n| n.as_str()).map(|s| s.to_string()),
            starred: meta_val.get("starred").and_then(|s| s.as_bool()).unwrap_or(false),
            auto_named: meta_val.get("auto_named").and_then(|a| a.as_bool()).unwrap_or(false),
        };
        map.insert(uuid.clone(), meta);
    }
    map
}

fn save_chats(chats: &HashMap<String, ChatMeta>) {
    let mut obj = serde_json::Map::new();
    for (uuid, meta) in chats {
        let mut entry = serde_json::Map::new();
        if let Some(ref name) = meta.name {
            entry.insert("name".into(), Value::String(name.clone()));
        }
        if meta.starred {
            entry.insert("starred".into(), Value::Bool(true));
        }
        if meta.auto_named {
            entry.insert("auto_named".into(), Value::Bool(true));
        }
        // only write entry if it has data
        if !entry.is_empty() {
            obj.insert(uuid.clone(), Value::Object(entry));
        }
    }
    if let Ok(json) = serde_json::to_string_pretty(&Value::Object(obj)) {
        let _ = fs::write(chats_path(), json);
    }
}

// --- LLM auto-naming ---

/// Build a naming prompt from user messages: first 2 + last 2, truncated to 200 chars each
fn build_naming_prompt(msgs: &[MessageLine]) -> String {
    let user_msgs: Vec<&MessageLine> = msgs.iter().filter(|m| m.role == "user").collect();
    if user_msgs.is_empty() {
        return String::new();
    }
    let mut selected: Vec<&MessageLine> = Vec::new();
    // first 2
    for m in user_msgs.iter().take(2) {
        selected.push(m);
    }
    // last 2 (deduped by pointer)
    let len = user_msgs.len();
    for m in user_msgs.iter().skip(len.saturating_sub(2)) {
        if !selected.iter().any(|s| std::ptr::eq(*s, *m)) {
            selected.push(m);
        }
    }
    let transcript: String = selected
        .iter()
        .map(|m| {
            let trunc: String = m.text.chars().take(200).collect();
            format!("User: {}", trunc)
        })
        .collect::<Vec<_>>()
        .join("\n");
    format!("Name this conversation:\n\n{}", transcript)
}

/// Ask local LLM to name a chat. Returns None if server is down or error.
fn llm_auto_name(endpoint: &str, msgs: &[MessageLine]) -> Option<String> {
    let prompt = build_naming_prompt(msgs);
    if prompt.is_empty() {
        return None;
    }
    let url = format!("{}/v1/chat/completions", endpoint.trim_end_matches('/'));
    let body = serde_json::json!({
        "model": "qwen2.5-0.5b",
        "messages": [
            {
                "role": "system",
                "content": "You are a chat naming assistant. Given a conversation, output ONLY a short title (2-6 words). No quotes, no explanation, no punctuation at the end."
            },
            {
                "role": "user",
                "content": prompt
            }
        ],
        "max_tokens": 20,
        "temperature": 0.3
    });
    let resp = ureq::post(&url)
        .set("Content-Type", "application/json")
        .timeout(std::time::Duration::from_secs(5))
        .send_string(&body.to_string())
        .ok()?;
    let resp_body = resp.into_string().ok()?;
    let val: Value = serde_json::from_str(&resp_body).ok()?;
    let name = val
        .get("choices")
        .and_then(|c: &Value| c.get(0))
        .and_then(|c: &Value| c.get("message"))
        .and_then(|m: &Value| m.get("content"))
        .and_then(|c: &Value| c.as_str())
        .map(|s: &str| s.trim().to_string())?;
    if name.is_empty() {
        None
    } else {
        Some(name)
    }
}

/// Check LLM health (returns true if server responds)
fn llm_is_healthy(endpoint: &str) -> bool {
    let url = format!("{}/health", endpoint.trim_end_matches('/'));
    ureq::get(&url)
        .timeout(std::time::Duration::from_secs(1))
        .call()
        .is_ok()
}

// --- transcript data ---

#[derive(Clone)]
struct RemoteOrigin {
    hostname: String,
    tcp_addr: SocketAddr,
}

#[derive(Clone)]
struct TranscriptEntry {
    path: PathBuf,
    uuid: String,
    project: String, // display label: "claude", "cursor", "codex", or custom name
    source_format: SourceFormat,
    mtime_secs: u64,
    preview: String,
    timestamp: String,
    remote: Option<RemoteOrigin>,
}

// --- LAN peer sync ---

const LAN_PORT: u16 = 21847;

struct PeerInfo {
    hostname: String,
    #[allow(dead_code)]
    tcp_port: u16,
    last_seen: Instant,
}

struct PeerState {
    peers: HashMap<SocketAddr, PeerInfo>,
    remote_entries: Vec<TranscriptEntry>,
    dirty: bool,
}

fn format_timestamp(secs: u64) -> String {
    let Some(dt) = Local.timestamp_opt(secs as i64, 0).single() else {
        return String::new();
    };
    let now = Local::now();
    let today = now.date_naive();
    let entry_date = dt.date_naive();
    let days_ago = (today - entry_date).num_days();
    if days_ago == 0 {
        dt.format("today %l:%M %p").to_string().trim().to_string()
    } else if days_ago == 1 {
        dt.format("yesterday %l:%M %p")
            .to_string()
            .trim()
            .to_string()
    } else if days_ago < 7 {
        dt.format("%a %l:%M %p").to_string().trim().to_string()
    } else {
        dt.format("%b %d, %l:%M %p").to_string().trim().to_string()
    }
}

/// Format a UNIX timestamp suitable for per-message display in view mode
fn format_message_time(secs: u64) -> String {
    let Some(dt) = Local.timestamp_opt(secs as i64, 0).single() else {
        return String::new();
    };
    let now = Local::now();
    let today = now.date_naive();
    let entry_date = dt.date_naive();
    let days_ago = (today - entry_date).num_days();
    if days_ago == 0 {
        dt.format("%l:%M %p").to_string().trim().to_string()
    } else if days_ago == 1 {
        format!("yesterday {}", dt.format("%l:%M %p").to_string().trim())
    } else if days_ago < 7 {
        dt.format("%a %l:%M %p").to_string().trim().to_string()
    } else {
        dt.format("%b %d %l:%M %p").to_string().trim().to_string()
    }
}

fn find_all_jsonl(dir: &Path, acc: &mut Vec<PathBuf>) {
    let Ok(entries) = fs::read_dir(dir) else {
        return;
    };
    for e in entries.flatten() {
        let path = e.path();
        if path.is_dir() {
            find_all_jsonl(&path, acc);
        } else if path.extension().map_or(false, |e| e == "jsonl") {
            acc.push(path);
        }
    }
}

fn is_top_level_session(base: &Path, path: &Path) -> bool {
    path.parent()
        .and_then(|p| p.strip_prefix(base).ok())
        .map(|rel| rel.components().count() == 1)
        .unwrap_or(false)
}

/// Discover JSONL files for a Claude-layout source: root/projects/<dir>/UUID.jsonl
fn find_claude_transcripts(src: &SourceConfig) -> Vec<TranscriptEntry> {
    let base = src.root.join("projects");
    if !base.exists() {
        return vec![];
    }
    let mut files = Vec::new();
    find_all_jsonl(&base, &mut files);
    files
        .into_iter()
        .filter(|p| is_top_level_session(&base, p))
        .filter_map(|path| make_entry(path, &src.name, src.format.clone()))
        .collect()
}

/// Discover JSONL files for Cursor: root/projects/<dir>/agent-transcripts/UUID/UUID.jsonl
fn find_cursor_transcripts(src: &SourceConfig) -> Vec<TranscriptEntry> {
    let base = src.root.join("projects");
    if !base.exists() {
        return vec![];
    }
    let mut out = Vec::new();
    // iterate project dirs
    let Ok(proj_dirs) = fs::read_dir(&base) else {
        return vec![];
    };
    for proj in proj_dirs.flatten() {
        if !proj.path().is_dir() {
            continue;
        }
        let at_dir = proj.path().join("agent-transcripts");
        if !at_dir.exists() {
            continue;
        }
        let Ok(sessions) = fs::read_dir(&at_dir) else {
            continue;
        };
        for session in sessions.flatten() {
            if !session.path().is_dir() {
                continue;
            }
            // look for UUID.jsonl inside the session dir
            let mut files = Vec::new();
            find_all_jsonl(&session.path(), &mut files);
            for path in files {
                if let Some(entry) = make_entry(path, &src.name, src.format.clone()) {
                    out.push(entry);
                }
            }
        }
    }
    out
}

/// Discover JSONL files for Codex: root/sessions/YYYY/MM/DD/*.jsonl
fn find_codex_transcripts(src: &SourceConfig) -> Vec<TranscriptEntry> {
    let base = src.root.join("sessions");
    if !base.exists() {
        return vec![];
    }
    let mut files = Vec::new();
    find_all_jsonl(&base, &mut files);
    files
        .into_iter()
        .filter_map(|path| make_entry(path, &src.name, src.format.clone()))
        .collect()
}

/// Discover JSONL files with flat recursive scan
fn find_flat_transcripts(src: &SourceConfig) -> Vec<TranscriptEntry> {
    if !src.root.exists() {
        return vec![];
    }
    let mut files = Vec::new();
    find_all_jsonl(&src.root, &mut files);
    files
        .into_iter()
        .filter_map(|path| make_entry(path, &src.name, src.format.clone()))
        .collect()
}

fn make_entry(path: PathBuf, source_name: &str, format: SourceFormat) -> Option<TranscriptEntry> {
    let meta = fs::metadata(&path).ok()?;
    let uuid = path.file_stem()?.to_string_lossy().into_owned();
    let mtime_secs = meta
        .modified()
        .ok()
        .and_then(|t| t.duration_since(std::time::UNIX_EPOCH).ok())
        .map(|d| d.as_secs())
        .unwrap_or(0);
    let preview = preview_first_line(&path, &format).unwrap_or_default();
    let timestamp = format_timestamp(mtime_secs);
    Some(TranscriptEntry {
        path,
        uuid,
        project: source_name.to_string(),
        source_format: format,
        mtime_secs,
        preview,
        timestamp,
        remote: None,
    })
}

fn get_all_transcripts(sources: &[SourceConfig]) -> Vec<TranscriptEntry> {
    let mut all = Vec::new();
    for src in sources {
        let entries = match src.layout.as_str() {
            "projects" => find_claude_transcripts(src),
            "agent-transcripts" => find_cursor_transcripts(src),
            "sessions" => find_codex_transcripts(src),
            _ => find_flat_transcripts(src),
        };
        all.extend(entries);
    }
    all.sort_by(|a, b| b.mtime_secs.cmp(&a.mtime_secs));
    all
}

/// Extract preview text from a JSONL value (format-aware)
fn preview_from_value(v: &Value, format: &SourceFormat) -> String {
    match format {
        SourceFormat::Codex => {
            // Codex: payload.content[].text or payload.content[].input_text
            let content = v
                .get("payload")
                .and_then(|p| p.get("content"))
                .or_else(|| v.get("message").and_then(|m| m.get("content")));
            let Some(c) = content else {
                return String::new();
            };
            if let Some(arr) = c.as_array() {
                return arr
                    .iter()
                    .filter_map(|x| {
                        x.get("text")
                            .or_else(|| x.get("input_text"))
                            .and_then(|t| t.as_str())
                    })
                    .collect::<Vec<_>>()
                    .join(" ")
                    .trim()
                    .to_string();
            }
            String::new()
        }
        _ => {
            // Claude + Cursor: message.content (string or array)
            let content = v.get("message").and_then(|m| m.get("content"));
            let Some(c) = content else {
                return String::new();
            };
            if let Some(s) = c.as_str() {
                return s.to_string();
            }
            if let Some(arr) = c.as_array() {
                return arr
                    .iter()
                    .filter_map(|x| x.get("text").and_then(|t| t.as_str()))
                    .collect::<Vec<_>>()
                    .join(" ")
                    .trim()
                    .to_string();
            }
            String::new()
        }
    }
}

fn preview_first_line(path: &Path, format: &SourceFormat) -> Option<String> {
    let file = fs::File::open(path).ok()?;
    let reader = BufReader::new(file);
    for line in reader.lines().take(20) {
        let line = line.ok()?;
        let line = line.trim().to_string();
        if line.is_empty() {
            continue;
        }
        let j: Value = serde_json::from_str(&line).ok()?;
        // skip non-content entries
        let entry_type = j.get("type").and_then(|t| t.as_str()).unwrap_or("");
        if entry_type == "queue-operation" || entry_type == "session_meta" {
            continue;
        }
        let text = strip_query_wrapper(&preview_from_value(&j, format));
        if !text.is_empty() {
            let short: String = text.chars().take(120).collect();
            let short = if text.chars().count() > 120 {
                format!("{}..", short)
            } else {
                short
            };
            return Some(short.replace('\n', " ").trim().to_string());
        }
    }
    None
}

// --- message parsing ---

#[derive(Clone)]
struct MessageLine {
    role: String,
    text: String,
    timestamp: u64, // unix seconds from JSONL timestamp field
}

fn extract_text(v: &Value) -> String {
    if v.get("type").and_then(|t| t.as_str()) == Some("queue-operation") {
        return String::new();
    }
    let content = v.get("message").and_then(|m| m.get("content"));
    let Some(c) = content else {
        return String::new();
    };
    if let Some(s) = c.as_str() {
        return s.to_string();
    }
    if let Some(arr) = c.as_array() {
        return arr
            .iter()
            .filter_map(|x| {
                let t = x.get("type").and_then(|t| t.as_str()).unwrap_or("");
                match t {
                    "text" => x.get("text").and_then(|t| t.as_str()),
                    "thinking" => x.get("thinking").and_then(|t| t.as_str()),
                    _ => None, // skip tool_use, tool_result, etc
                }
            })
            .collect::<Vec<_>>()
            .join("\n");
    }
    String::new()
}

fn get_role(v: &Value) -> &str {
    if let Some(t) = v.get("type").and_then(|t| t.as_str()) {
        if t == "user" || t == "assistant" {
            return t;
        }
    }
    v.get("role")
        .or_else(|| v.get("message").and_then(|m| m.get("role")))
        .and_then(|r| r.as_str())
        .unwrap_or("unknown")
}

fn extract_timestamp(v: &Value) -> u64 {
    // try "timestamp" field (ISO string or unix number)
    if let Some(ts) = v.get("timestamp") {
        if let Some(n) = ts.as_u64() {
            return n;
        }
        if let Some(n) = ts.as_f64() {
            return n as u64;
        }
        if let Some(s) = ts.as_str() {
            // try parsing ISO 8601
            if let Ok(dt) = chrono::DateTime::parse_from_rfc3339(s) {
                return dt.timestamp() as u64;
            }
            // try "2024-01-15T10:30:00.000Z" variant
            if let Ok(dt) = chrono::NaiveDateTime::parse_from_str(s, "%Y-%m-%dT%H:%M:%S%.fZ") {
                return dt.and_utc().timestamp() as u64;
            }
        }
    }
    0
}

fn strip_query_wrapper(s: &str) -> String {
    let s = s.trim();
    let s = s.strip_prefix("<user_query>").unwrap_or(s).trim();
    let s = s.strip_suffix("</user_query>").unwrap_or(s).trim();
    s.to_string()
}

fn load_messages(path: &Path, format: &SourceFormat) -> Vec<MessageLine> {
    match format {
        SourceFormat::Codex => load_messages_codex(path),
        SourceFormat::Cursor => load_messages_cursor(path),
        _ => load_messages_claude(path),
    }
}

/// Claude format: event-based JSONL with type, message.role, message.content
fn load_messages_claude(path: &Path) -> Vec<MessageLine> {
    let file = match fs::File::open(path) {
        Ok(f) => f,
        _ => return vec![],
    };
    let reader = BufReader::new(file);
    let mut out = Vec::new();
    for line in reader.lines() {
        let line = match line {
            Ok(x) => x,
            _ => continue,
        };
        let line = line.trim().to_string();
        if line.is_empty() {
            continue;
        }
        let v: Value = match serde_json::from_str(&line) {
            Ok(x) => x,
            _ => continue,
        };
        if v.get("type").and_then(|t| t.as_str()) == Some("queue-operation") {
            continue;
        }
        let text = extract_text(&v);
        if text.is_empty() {
            continue;
        }
        let role = get_role(&v).to_string();
        let text = strip_query_wrapper(&text);
        let timestamp = extract_timestamp(&v);
        out.push(MessageLine {
            role,
            text,
            timestamp,
        });
    }
    out
}

/// Cursor format: {role, message: {content: [{type, text}]}} per line
fn load_messages_cursor(path: &Path) -> Vec<MessageLine> {
    let file = match fs::File::open(path) {
        Ok(f) => f,
        _ => return vec![],
    };
    let reader = BufReader::new(file);
    let mut out = Vec::new();
    for line in reader.lines() {
        let line = match line {
            Ok(x) => x,
            _ => continue,
        };
        let line = line.trim().to_string();
        if line.is_empty() {
            continue;
        }
        let v: Value = match serde_json::from_str(&line) {
            Ok(x) => x,
            _ => continue,
        };
        let role = v
            .get("role")
            .and_then(|r| r.as_str())
            .unwrap_or("unknown")
            .to_string();
        // content is array of {type, text}
        let content = v.get("message").and_then(|m| m.get("content"));
        let text = match content {
            Some(c) if c.is_array() => c
                .as_array()
                .unwrap()
                .iter()
                .filter_map(|x| x.get("text").and_then(|t| t.as_str()))
                .collect::<Vec<_>>()
                .join("\n"),
            Some(c) if c.is_string() => c.as_str().unwrap_or("").to_string(),
            _ => continue,
        };
        if text.is_empty() {
            continue;
        }
        let text = strip_query_wrapper(&text);
        let timestamp = extract_timestamp(&v);
        out.push(MessageLine {
            role,
            text,
            timestamp,
        });
    }
    out
}

/// Codex format: {type: "response_item", payload: {role, content: [{type, text/input_text}]}}
fn load_messages_codex(path: &Path) -> Vec<MessageLine> {
    let file = match fs::File::open(path) {
        Ok(f) => f,
        _ => return vec![],
    };
    let reader = BufReader::new(file);
    let mut out = Vec::new();
    for line in reader.lines() {
        let line = match line {
            Ok(x) => x,
            _ => continue,
        };
        let line = line.trim().to_string();
        if line.is_empty() {
            continue;
        }
        let v: Value = match serde_json::from_str(&line) {
            Ok(x) => x,
            _ => continue,
        };
        let entry_type = v.get("type").and_then(|t| t.as_str()).unwrap_or("");
        if entry_type == "session_meta" {
            continue;
        }
        // timestamp is top-level
        let timestamp = extract_timestamp(&v);

        let payload = match v.get("payload") {
            Some(p) => p,
            None => continue,
        };
        let role = payload
            .get("role")
            .and_then(|r| r.as_str())
            .unwrap_or("unknown");
        // map codex roles: "developer" -> "user", "assistant" stays
        let role = match role {
            "developer" | "user" => "user".to_string(),
            r => r.to_string(),
        };

        let content = payload.get("content");
        let text = match content {
            Some(c) if c.is_array() => c
                .as_array()
                .unwrap()
                .iter()
                .filter_map(|x| {
                    let t = x.get("type").and_then(|t| t.as_str()).unwrap_or("");
                    match t {
                        "output_text" | "input_text" | "text" => {
                            x.get("text")
                                .or_else(|| x.get("input_text"))
                                .and_then(|t| t.as_str())
                        }
                        _ => None,
                    }
                })
                .collect::<Vec<_>>()
                .join("\n"),
            _ => continue,
        };
        if text.is_empty() {
            continue;
        }
        let text = strip_query_wrapper(&text);
        out.push(MessageLine {
            role,
            text,
            timestamp,
        });
    }
    out
}

// --- message grouping ---

#[derive(Clone)]
struct MessageGroup {
    role: String,
    messages: Vec<MessageLine>,
}

fn group_messages(msgs: &[MessageLine]) -> Vec<MessageGroup> {
    let mut groups: Vec<MessageGroup> = Vec::new();
    for m in msgs {
        if let Some(last) = groups.last_mut() {
            if last.role == m.role {
                last.messages.push(m.clone());
                continue;
            }
        }
        groups.push(MessageGroup {
            role: m.role.clone(),
            messages: vec![m.clone()],
        });
    }
    groups
}

// --- layout helpers ---

fn chars_per_line(win_w: usize, advance: i32) -> usize {
    let usable = (win_w as i32) - 2 * PAD;
    (usable / advance).max(20) as usize
}

const LIST_LINES_PER_ITEM: i32 = 4;

fn wrap_str(s: &str, width: usize) -> Vec<String> {
    let mut out = Vec::new();
    for line in s.lines() {
        if line.is_empty() {
            out.push(String::new());
            continue;
        }
        let chars: Vec<char> = line.chars().collect();
        let mut pos = 0;
        while pos < chars.len() {
            let remaining = chars.len() - pos;
            if remaining <= width {
                out.push(chars[pos..].iter().collect());
                break;
            }
            let end = pos + width;
            let mut brk = end;
            for i in (pos..end).rev() {
                if chars[i] == ' ' {
                    brk = i;
                    break;
                }
            }
            if brk == end {
                let chunk: String = chars[pos..end].iter().collect();
                out.push(chunk);
                pos = end;
            } else {
                let chunk: String = chars[pos..brk].iter().collect();
                out.push(chunk);
                pos = brk + 1;
            }
        }
    }
    if out.is_empty() && !s.is_empty() {
        out.push(s.to_string());
    }
    out
}

/// Line metadata for the flattened view
#[derive(Clone)]
enum LineMeta {
    Normal,
    Timestamp,
    Toggle(usize), // group_idx — clickable to expand/collapse
}

fn flatten_groups(
    groups: &[MessageGroup],
    expanded: &HashSet<usize>,
    wrap_width: usize,
) -> (Vec<String>, Vec<LineMeta>) {
    let label_user = "[user] ";
    let label_asst = "[asst] ";
    let prefix_len = label_user.len();
    let text_width = wrap_width.saturating_sub(prefix_len).max(10);
    let continued = "       ";

    let mut lines = Vec::new();
    let mut meta = Vec::new();

    for (gi, group) in groups.iter().enumerate() {
        // blank separator between groups
        if gi > 0 {
            lines.push(String::new());
            meta.push(LineMeta::Normal);
        }

        let label = if group.role == "user" {
            label_user
        } else {
            label_asst
        };

        let is_asst = group.role != "user";
        let collapsed = is_asst && group.messages.len() > 1 && !expanded.contains(&gi);

        if collapsed {
            // show timestamp of last message
            let last = &group.messages[group.messages.len() - 1];
            if last.timestamp > 0 {
                let ts = format_message_time(last.timestamp);
                lines.push(format!("       {}", ts));
                meta.push(LineMeta::Timestamp);
            }

            // toggle indicator
            let hidden = group.messages.len() - 1;
            let toggle = format!(
                "       .. {} more response{} [click to expand]",
                hidden,
                if hidden == 1 { "" } else { "s" }
            );
            lines.push(toggle);
            meta.push(LineMeta::Toggle(gi));

            // render only last message text
            flatten_single_msg(last, label, continued, text_width, &mut lines, &mut meta);
        } else {
            // show all messages in group
            for (mi, m) in group.messages.iter().enumerate() {
                if mi > 0 {
                    lines.push(String::new());
                    meta.push(LineMeta::Normal);
                }

                // timestamp above each message
                if m.timestamp > 0 {
                    let ts = format_message_time(m.timestamp);
                    lines.push(format!("       {}", ts));
                    meta.push(LineMeta::Timestamp);
                }

                let msg_label = if mi == 0 { label } else { label };
                flatten_single_msg(m, msg_label, continued, text_width, &mut lines, &mut meta);
            }

            // collapse indicator for expanded multi-message groups
            if is_asst && group.messages.len() > 1 {
                let toggle = "       .. [click to collapse]".to_string();
                lines.push(toggle);
                meta.push(LineMeta::Toggle(gi));
            }
        }
    }
    (lines, meta)
}

fn flatten_single_msg(
    m: &MessageLine,
    label: &str,
    continued: &str,
    text_width: usize,
    lines: &mut Vec<String>,
    meta: &mut Vec<LineMeta>,
) {
    let text_lines: Vec<&str> = m.text.lines().collect();
    for (i, line) in text_lines.iter().enumerate() {
        let blocks = wrap_str(line, text_width);
        for (j, block) in blocks.iter().enumerate() {
            let prefix = if i == 0 && j == 0 { label } else { continued };
            lines.push(format!("{}{}", prefix, block));
            meta.push(LineMeta::Normal);
        }
    }
}

// --- markdown rendering ---

struct TextRun {
    text: String,
    color: (u8, u8, u8),
    bg: Option<(u8, u8, u8)>,
}

fn parse_inline_markdown(line: &str, base_color: (u8, u8, u8)) -> Vec<TextRun> {
    let chars: Vec<char> = line.chars().collect();
    let mut runs = Vec::new();
    let mut i = 0;
    let mut current = String::new();

    while i < chars.len() {
        // **bold**
        if i + 1 < chars.len() && chars[i] == '*' && chars[i + 1] == '*' {
            let start = i + 2;
            let mut end = None;
            for j in start..chars.len().saturating_sub(1) {
                if chars[j] == '*' && chars[j + 1] == '*' {
                    end = Some(j);
                    break;
                }
            }
            if let Some(e) = end {
                if !current.is_empty() {
                    runs.push(TextRun {
                        text: std::mem::take(&mut current),
                        color: base_color,
                        bg: None,
                    });
                }
                let text: String = chars[start..e].iter().collect();
                runs.push(TextRun {
                    text,
                    color: COL_BOLD,
                    bg: None,
                });
                i = e + 2;
                continue;
            }
        }

        // `inline code`
        if chars[i] == '`' && (i + 1 >= chars.len() || chars[i + 1] != '`') {
            let start = i + 1;
            let mut end = None;
            for j in start..chars.len() {
                if chars[j] == '`' {
                    end = Some(j);
                    break;
                }
            }
            if let Some(e) = end {
                if !current.is_empty() {
                    runs.push(TextRun {
                        text: std::mem::take(&mut current),
                        color: base_color,
                        bg: None,
                    });
                }
                let text: String = chars[start..e].iter().collect();
                runs.push(TextRun {
                    text,
                    color: COL_CODE,
                    bg: Some(COL_CODE_BG),
                });
                i = e + 1;
                continue;
            }
        }

        current.push(chars[i]);
        i += 1;
    }

    if !current.is_empty() {
        runs.push(TextRun {
            text: current,
            color: base_color,
            bg: None,
        });
    }
    runs
}

fn compute_code_block_state(lines: &[String]) -> Vec<bool> {
    let mut state = Vec::with_capacity(lines.len());
    let mut inside = false;
    for line in lines {
        let trimmed = line.trim_start();
        if trimmed.starts_with("```") {
            state.push(true); // fence line gets code styling
            inside = !inside;
        } else {
            state.push(inside);
        }
    }
    state
}

// --- rendering (fontdue) ---

fn draw_text_ttf(
    buf: &mut [u32],
    x: i32,
    y: i32,
    text: &str,
    color: (u8, u8, u8),
    atlas: &mut FontAtlas,
    buf_w: usize,
    buf_h: usize,
) {
    let mut px = x;
    let baseline = y + atlas.ascent;
    for ch in text.chars() {
        let g = atlas.glyph(ch);
        let gx = px + g.x_off;
        let gy = baseline - g.height as i32 - g.y_off;
        for row in 0..g.height {
            for col in 0..g.width {
                let alpha = g.bitmap[row * g.width + col];
                if alpha == 0 {
                    continue;
                }
                let rx = gx + col as i32;
                let ry = gy + row as i32;
                if rx >= 0 && ry >= 0 {
                    let ux = rx as usize;
                    let uy = ry as usize;
                    if ux < buf_w && uy < buf_h {
                        let a = alpha as u32;
                        let bg = buf[uy * buf_w + ux];
                        let bg_r = (bg >> 16) & 0xFF;
                        let bg_g = (bg >> 8) & 0xFF;
                        let bg_b = bg & 0xFF;
                        let r = (color.0 as u32 * a + bg_r * (255 - a)) / 255;
                        let g = (color.1 as u32 * a + bg_g * (255 - a)) / 255;
                        let b = (color.2 as u32 * a + bg_b * (255 - a)) / 255;
                        buf[uy * buf_w + ux] = (r << 16) | (g << 8) | b;
                    }
                }
            }
        }
        px += atlas.advance;
    }
}

fn draw_styled_runs(
    buf: &mut [u32],
    x: i32,
    y: i32,
    runs: &[TextRun],
    atlas: &mut FontAtlas,
    buf_w: usize,
    buf_h: usize,
) {
    let mut px = x;
    let lh = atlas.line_height;
    for run in runs {
        let char_count = run.text.chars().count() as i32;
        if let Some(bg) = run.bg {
            fill_rect(buf, px, y, char_count * atlas.advance + 2, lh, bg, buf_w, buf_h);
        }
        draw_text_ttf(buf, px, y, &run.text, run.color, atlas, buf_w, buf_h);
        px += char_count * atlas.advance;
    }
}

fn fill_rect(
    buf: &mut [u32],
    x: i32,
    y: i32,
    w: i32,
    h: i32,
    color: (u8, u8, u8),
    buf_w: usize,
    buf_h: usize,
) {
    let pix = rgb(color.0, color.1, color.2);
    for dy in 0..h {
        for dx in 0..w {
            let ux = (x + dx) as usize;
            let uy = (y + dy) as usize;
            if ux < buf_w && uy < buf_h {
                buf[uy * buf_w + ux] = pix;
            }
        }
    }
}

// --- search / filter ---

fn file_contains_text(path: &Path, query_lower: &str) -> bool {
    let Ok(content) = fs::read_to_string(path) else {
        return false;
    };
    content.to_lowercase().contains(query_lower)
}

fn matches_quick(t: &TranscriptEntry, q: &str) -> bool {
    t.preview.to_lowercase().contains(q)
        || t.project.to_lowercase().contains(q)
        || t.uuid.to_lowercase().contains(q)
}

fn filter_transcripts_quick(
    all: &[TranscriptEntry],
    query: &str,
    favs_only: bool,
    chats: &HashMap<String, ChatMeta>,
) -> Vec<TranscriptEntry> {
    let q = query.to_lowercase();
    all.iter()
        .filter(|t| {
            if favs_only && !chats.get(&t.uuid).map_or(false, |m| m.starred) {
                return false;
            }
            if !q.is_empty() {
                // also search by chat name
                if let Some(meta) = chats.get(&t.uuid) {
                    if let Some(ref name) = meta.name {
                        if name.to_lowercase().contains(&q) {
                            return true;
                        }
                    }
                }
                return matches_quick(t, &q);
            }
            true
        })
        .cloned()
        .collect()
}

fn filter_transcripts_deep(
    all: &[TranscriptEntry],
    query: &str,
    favs_only: bool,
    chats: &HashMap<String, ChatMeta>,
) -> Vec<TranscriptEntry> {
    let q = query.to_lowercase();
    all.iter()
        .filter(|t| {
            if favs_only && !chats.get(&t.uuid).map_or(false, |m| m.starred) {
                return false;
            }
            if !q.is_empty() {
                if let Some(meta) = chats.get(&t.uuid) {
                    if let Some(ref name) = meta.name {
                        if name.to_lowercase().contains(&q) {
                            return true;
                        }
                    }
                }
                if matches_quick(t, &q) {
                    return true;
                }
                return file_contains_text(&t.path, &q);
            }
            true
        })
        .cloned()
        .collect()
}

fn key_to_char(key: Key, shift: bool) -> Option<char> {
    let c = match key {
        Key::A => 'a', Key::B => 'b', Key::C => 'c', Key::D => 'd',
        Key::E => 'e', Key::F => 'f', Key::G => 'g', Key::H => 'h',
        Key::I => 'i', Key::J => 'j', Key::K => 'k', Key::L => 'l',
        Key::M => 'm', Key::N => 'n', Key::O => 'o', Key::P => 'p',
        Key::Q => 'q', Key::R => 'r', Key::S => 's', Key::T => 't',
        Key::U => 'u', Key::V => 'v', Key::W => 'w', Key::X => 'x',
        Key::Y => 'y', Key::Z => 'z',
        Key::Key0 | Key::NumPad0 => '0', Key::Key1 | Key::NumPad1 => '1',
        Key::Key2 | Key::NumPad2 => '2', Key::Key3 | Key::NumPad3 => '3',
        Key::Key4 | Key::NumPad4 => '4', Key::Key5 | Key::NumPad5 => '5',
        Key::Key6 | Key::NumPad6 => '6', Key::Key7 | Key::NumPad7 => '7',
        Key::Key8 | Key::NumPad8 => '8', Key::Key9 | Key::NumPad9 => '9',
        Key::Space => ' ',
        Key::Minus => if shift { '_' } else { '-' },
        Key::Period => '.', Key::Slash => '/',
        _ => return None,
    };
    if shift && c.is_ascii_alphabetic() {
        Some(c.to_ascii_uppercase())
    } else {
        Some(c)
    }
}

// --- selection state ---

#[derive(Clone, Default)]
struct Selection {
    active: bool,
    anchor_line: usize,
    anchor_col: usize,
    cursor_line: usize,
    cursor_col: usize,
}

impl Selection {
    fn ordered(&self) -> ((usize, usize), (usize, usize)) {
        let a = (self.anchor_line, self.anchor_col);
        let b = (self.cursor_line, self.cursor_col);
        if a <= b {
            (a, b)
        } else {
            (b, a)
        }
    }

    fn extract_text(&self, lines: &[String]) -> String {
        if !self.active {
            return String::new();
        }
        let ((sl, sc), (el, ec)) = self.ordered();
        let mut out = String::new();
        for i in sl..=el {
            let Some(line) = lines.get(i) else { continue };
            let chars: Vec<char> = line.chars().collect();
            let start = if i == sl { sc.min(chars.len()) } else { 0 };
            let end = if i == el { ec.min(chars.len()) } else { chars.len() };
            if start <= end {
                out.extend(&chars[start..end]);
            }
            if i < el {
                out.push('\n');
            }
        }
        out
    }

    fn is_click(&self) -> bool {
        self.anchor_line == self.cursor_line && self.anchor_col == self.cursor_col
    }
}

// --- file mtime helper ---

fn get_file_mtime(path: &Path) -> u64 {
    fs::metadata(path)
        .ok()
        .and_then(|m| m.modified().ok())
        .and_then(|t| t.duration_since(std::time::UNIX_EPOCH).ok())
        .map(|d| d.as_secs())
        .unwrap_or(0)
}

// --- app state ---

enum AppState {
    List {
        transcripts: Vec<TranscriptEntry>,
        filtered: Vec<TranscriptEntry>,
        selected: usize,
        scroll: i32,
        searching: bool,
        search_term: String,
        favs_only: bool,
        last_scan: Instant,
        renaming: bool,
        rename_buf: String,
    },
    View {
        path: PathBuf,
        source_format: SourceFormat,
        groups: Vec<MessageGroup>,
        expanded: HashSet<usize>,
        lines: Vec<String>,
        line_meta: Vec<LineMeta>,
        in_code: Vec<bool>,
        last_wrap_w: usize,
        scroll: i32,
        sel: Selection,
        file_mtime: u64,
        last_check: Instant,
        remote: Option<RemoteOrigin>,
        chat_uuid: String,
    },
}

const HEADER_H: i32 = 28;

/// Rebuild flattened lines from groups + expansion state
fn rebuild_view(
    groups: &[MessageGroup],
    expanded: &HashSet<usize>,
    wrap_w: usize,
) -> (Vec<String>, Vec<LineMeta>, Vec<bool>) {
    let (lines, meta) = flatten_groups(groups, expanded, wrap_w);
    let in_code = compute_code_block_state(&lines);
    (lines, meta, in_code)
}

// --- LAN peer sync functions ---

fn get_hostname() -> String {
    env::var("COMPUTERNAME")
        .or_else(|_| env::var("HOSTNAME"))
        .unwrap_or_else(|_| "unknown".into())
}

fn start_tcp_server(listener: TcpListener, sources: Vec<SourceConfig>) {
    thread::spawn(move || {
        for stream in listener.incoming() {
            let Ok(stream) = stream else { continue };
            let srcs = sources.clone();
            thread::spawn(move || {
                handle_tcp_client(stream, &srcs);
            });
        }
    });
}

fn handle_tcp_client(stream: TcpStream, sources: &[SourceConfig]) {
    let _ = stream.set_read_timeout(Some(std::time::Duration::from_secs(2)));
    let Ok(write_stream) = stream.try_clone() else { return };
    let mut writer = write_stream;
    let mut reader = BufReader::new(stream);
    let mut line = String::new();
    if reader.read_line(&mut line).is_err() {
        return;
    }
    let line = line.trim().to_string();
    if line == "LIST" {
        let transcripts = get_all_transcripts(sources);
        let arr: Vec<Value> = transcripts.iter().map(|t| {
            serde_json::json!({
                "uuid": t.uuid,
                "project": t.project,
                "mtime": t.mtime_secs,
                "preview": t.preview,
                "timestamp": t.timestamp,
            })
        }).collect();
        let json = serde_json::to_string(&arr).unwrap_or_else(|_| "[]".into());
        let _ = writer.write_all(json.as_bytes());
        let _ = writer.write_all(b"\n");
    } else if let Some(uuid) = line.strip_prefix("GET ") {
        let uuid = uuid.trim();
        // find the transcript by uuid
        let transcripts = get_all_transcripts(sources);
        if let Some(entry) = transcripts.iter().find(|t| t.uuid == uuid) {
            let msgs = load_messages(&entry.path, &entry.source_format);
            let arr: Vec<Value> = msgs.iter().map(|m| {
                serde_json::json!({
                    "role": m.role,
                    "text": m.text,
                    "timestamp": m.timestamp,
                })
            }).collect();
            let json = serde_json::to_string(&arr).unwrap_or_else(|_| "[]".into());
            let _ = writer.write_all(json.as_bytes());
            let _ = writer.write_all(b"\n");
        } else {
            let _ = writer.write_all(b"[]\n");
        }
    }
}

fn start_beacon_sender(hostname: String, tcp_port: u16) {
    thread::spawn(move || {
        let Ok(sock) = UdpSocket::bind("0.0.0.0:0") else { return };
        let _ = sock.set_broadcast(true);
        let beacon = format!(
            "{{\"h\":\"{}\",\"p\":{},\"v\":\"0.1.0\"}}\n",
            hostname, tcp_port
        );
        let dest: SocketAddr = format!("255.255.255.255:{}", LAN_PORT).parse().unwrap();
        loop {
            let _ = sock.send_to(beacon.as_bytes(), dest);
            thread::sleep(std::time::Duration::from_secs(3));
        }
    });
}

fn start_beacon_listener(peer_state: Arc<Mutex<PeerState>>, my_hostname: String) {
    thread::spawn(move || {
        let Ok(sock) = UdpSocket::bind(format!("0.0.0.0:{}", LAN_PORT)) else { return };
        let _ = sock.set_read_timeout(Some(std::time::Duration::from_secs(1)));
        let mut buf = [0u8; 1024];
        loop {
            match sock.recv_from(&mut buf) {
                Ok((n, src_addr)) => {
                    let msg = String::from_utf8_lossy(&buf[..n]).to_string();
                    let msg = msg.trim();
                    let Ok(val) = serde_json::from_str::<Value>(msg) else { continue };
                    let Some(h) = val.get("h").and_then(|v| v.as_str()) else { continue };
                    let Some(p) = val.get("p").and_then(|v| v.as_u64()) else { continue };
                    if h == my_hostname {
                        continue; // skip self
                    }
                    let tcp_port = p as u16;
                    let peer_tcp_addr: SocketAddr = SocketAddr::new(src_addr.ip(), tcp_port);
                    let mut changed = false;
                    if let Ok(mut ps) = peer_state.lock() {
                        let is_new = !ps.peers.contains_key(&peer_tcp_addr);
                        ps.peers.insert(peer_tcp_addr, PeerInfo {
                            hostname: h.to_string(),
                            tcp_port,
                            last_seen: Instant::now(),
                        });
                        // evict stale peers
                        let before = ps.peers.len();
                        ps.peers.retain(|_, info| info.last_seen.elapsed().as_secs() < 9);
                        let after = ps.peers.len();
                        if is_new || before != after {
                            changed = true;
                        }
                    }
                    if changed {
                        // fetch LIST from all live peers
                        let addrs: Vec<(SocketAddr, String)> = {
                            let Ok(ps) = peer_state.lock() else { continue };
                            ps.peers.iter().map(|(addr, info)| (*addr, info.hostname.clone())).collect()
                        };
                        let mut all_remote = Vec::new();
                        for (addr, hostname) in &addrs {
                            let entries = fetch_peer_list(*addr, hostname);
                            all_remote.extend(entries);
                        }
                        if let Ok(mut ps) = peer_state.lock() {
                            ps.remote_entries = all_remote;
                            ps.dirty = true;
                        }
                    }
                }
                Err(_) => {
                    // timeout — evict stale peers
                    let mut changed = false;
                    if let Ok(mut ps) = peer_state.lock() {
                        let before = ps.peers.len();
                        ps.peers.retain(|_, info| info.last_seen.elapsed().as_secs() < 9);
                        if ps.peers.len() != before {
                            changed = true;
                        }
                    }
                    if changed {
                        let addrs: Vec<(SocketAddr, String)> = {
                            let Ok(ps) = peer_state.lock() else { continue };
                            ps.peers.iter().map(|(addr, info)| (*addr, info.hostname.clone())).collect()
                        };
                        let mut all_remote = Vec::new();
                        for (addr, hostname) in &addrs {
                            let entries = fetch_peer_list(*addr, hostname);
                            all_remote.extend(entries);
                        }
                        if let Ok(mut ps) = peer_state.lock() {
                            ps.remote_entries = all_remote;
                            ps.dirty = true;
                        }
                    }
                }
            }
        }
    });
}

fn fetch_peer_list(addr: SocketAddr, hostname: &str) -> Vec<TranscriptEntry> {
    let Ok(mut stream) = TcpStream::connect_timeout(&addr, std::time::Duration::from_secs(2)) else {
        return vec![];
    };
    let _ = stream.set_read_timeout(Some(std::time::Duration::from_secs(2)));
    if stream.write_all(b"LIST\n").is_err() {
        return vec![];
    }
    let mut response = String::new();
    let _ = stream.read_to_string(&mut response);
    let response = response.trim();
    let Ok(arr) = serde_json::from_str::<Vec<Value>>(response) else {
        return vec![];
    };
    arr.iter().filter_map(|v| {
        let uuid = v.get("uuid")?.as_str()?.to_string();
        let project = v.get("project")?.as_str()?.to_string();
        let mtime = v.get("mtime")?.as_u64().unwrap_or(0);
        let preview = v.get("preview").and_then(|p| p.as_str()).unwrap_or("").to_string();
        let timestamp = v.get("timestamp").and_then(|t| t.as_str()).unwrap_or("").to_string();
        Some(TranscriptEntry {
            path: PathBuf::new(),
            uuid,
            project,
            source_format: SourceFormat::Generic,
            mtime_secs: mtime,
            preview,
            timestamp,
            remote: Some(RemoteOrigin {
                hostname: hostname.to_string(),
                tcp_addr: addr,
            }),
        })
    }).collect()
}

fn fetch_remote_messages(addr: SocketAddr, uuid: &str) -> Vec<MessageLine> {
    let Ok(mut stream) = TcpStream::connect_timeout(&addr, std::time::Duration::from_secs(2)) else {
        return vec![];
    };
    let _ = stream.set_read_timeout(Some(std::time::Duration::from_secs(2)));
    let cmd = format!("GET {}\n", uuid);
    if stream.write_all(cmd.as_bytes()).is_err() {
        return vec![];
    }
    let mut response = String::new();
    let _ = stream.read_to_string(&mut response);
    let response = response.trim();
    let Ok(arr) = serde_json::from_str::<Vec<Value>>(response) else {
        return vec![];
    };
    arr.iter().filter_map(|v| {
        let role = v.get("role")?.as_str()?.to_string();
        let text = v.get("text")?.as_str()?.to_string();
        let timestamp = v.get("timestamp").and_then(|t| t.as_u64()).unwrap_or(0);
        Some(MessageLine { role, text, timestamp })
    }).collect()
}

fn main() {
    let cfg = load_config();
    let sources = cfg.sources;
    let theme = cfg.theme;
    let font_name = cfg.font;
    let font_weight = cfg.font_weight;
    let snapshot = env::var("CHAT_DADDY_SNAP").is_ok();

    let mut window = Window::new(
        "chat-daddy",
        WIN_W,
        WIN_H,
        WindowOptions {
            resize: true,
            ..WindowOptions::default()
        },
    )
    .expect("window");
    window.set_target_fps(60);

    let all_transcripts = get_all_transcripts(&sources);
    let mut chats = load_chats();
    let mut state = AppState::List {
        filtered: all_transcripts.clone(),
        transcripts: all_transcripts,
        selected: 0,
        scroll: 0,
        searching: false,
        search_term: String::new(),
        favs_only: false,
        last_scan: Instant::now(),
        renaming: false,
        rename_buf: String::new(),
    };

    let mut buffer = vec![rgb(theme.bg.0, theme.bg.1, theme.bg.2); WIN_W * WIN_H];
    let mut transition: Option<AppState> = None;
    let mut size_idx = DEFAULT_SIZE_IDX;
    let mut atlas = FontAtlas::with_font(FONT_SIZES[size_idx], &font_name, font_weight);
    let mut clipboard = Clipboard::new().ok();
    let mut mouse_dragging = false;
    let mut needs_rebuild = false;
    let mut show_help = false;
    let llm_endpoint = cfg.llm_endpoint;
    let mut llm_healthy = false;
    let mut llm_last_check = Instant::now() - std::time::Duration::from_secs(60); // force first check
    let mut auto_name_last = Instant::now() - std::time::Duration::from_secs(60);

    // --- LAN peer sync setup ---
    let my_hostname = get_hostname();
    let tcp_listener = TcpListener::bind("0.0.0.0:0").ok();
    let my_tcp_port = tcp_listener.as_ref().map(|l| l.local_addr().unwrap().port()).unwrap_or(0);
    let peer_state = Arc::new(Mutex::new(PeerState {
        peers: HashMap::new(),
        remote_entries: Vec::new(),
        dirty: false,
    }));

    if let Some(listener) = tcp_listener {
        start_tcp_server(listener, sources.clone());
    }
    if my_tcp_port > 0 {
        start_beacon_sender(my_hostname.clone(), my_tcp_port);
        start_beacon_listener(Arc::clone(&peer_state), my_hostname.clone());
    }

    let mut last_merged: Vec<TranscriptEntry> = Vec::new();

    // theme-derived color aliases (lowercase, used by render code)
    let c_bg = theme.bg;
    let c_dim = theme.dim;
    let c_user = theme.user;
    let c_asst = theme.asst;
    let c_text = theme.text;
    let c_sel = theme.sel;
    let c_sep = theme.sep;
    let c_header_bg = theme.header_bg;
    let c_accent = theme.accent;
    let c_search_bg = theme.search_bg;
    let c_timestamp = theme.timestamp;
    let c_select_bg = theme.select_bg;
    let c_code = theme.code;
    let c_code_bg = theme.code_bg;
    let _c_bold = theme.bold;
    let c_toggle = theme.toggle;
    let c_heading = theme.heading;
    let c_msg_time = theme.msg_time;

    while window.is_open() {
        if let Some(s) = transition.take() {
            state = s;
        }

        let (win_w, win_h) = window.get_size();
        if buffer.len() != win_w * win_h {
            buffer = vec![rgb(c_bg.0, c_bg.1, c_bg.2); win_w * win_h];
        }
        let buf_w = win_w;
        let buf_h = win_h;
        let lh = atlas.line_height;
        let advance = atlas.advance;
        let content_top = HEADER_H + PAD;

        for p in &mut buffer {
            *p = rgb(c_bg.0, c_bg.1, c_bg.2);
        }

        // --- periodic refresh ---
        match &mut state {
            AppState::List {
                transcripts,
                filtered,
                search_term,
                favs_only,
                last_scan,
                selected,
                scroll,
                ..
            } => {
                if last_scan.elapsed().as_secs() >= 2 {
                    *last_scan = Instant::now();
                    let mut fresh = get_all_transcripts(&sources);

                    // merge remote peers
                    if let Ok(mut ps) = peer_state.try_lock() {
                        ps.dirty = false;
                        let remote = ps.remote_entries.clone();
                        fresh.extend(remote);
                        fresh.sort_by(|a, b| b.mtime_secs.cmp(&a.mtime_secs));
                    }

                    last_merged = fresh.clone();

                    if fresh.len() != transcripts.len()
                        || fresh
                            .first()
                            .map(|t| t.mtime_secs)
                            != transcripts.first().map(|t| t.mtime_secs)
                        || fresh.iter().any(|t| t.remote.is_some()) != transcripts.iter().any(|t| t.remote.is_some())
                    {
                        *transcripts = fresh.clone();
                        *filtered = filter_transcripts_quick(
                            &fresh,
                            search_term,
                            *favs_only,
                            &chats,
                        );
                        *selected = (*selected).min(filtered.len().saturating_sub(1));
                        let visible_lines = ((win_h as i32) - content_top - PAD) / lh;
                        let total_lines = (filtered.len() as i32) * LIST_LINES_PER_ITEM;
                        *scroll = ((*selected as i32) * LIST_LINES_PER_ITEM - visible_lines / 2)
                            .max(0)
                            .min((total_lines - visible_lines).max(0));
                    }
                }
            }
            AppState::View {
                path,
                source_format,
                groups,
                expanded,
                lines,
                line_meta,
                in_code,
                last_wrap_w,
                scroll,
                file_mtime,
                last_check,
                remote,
                chat_uuid,
                sel: _,
            } => {
                if last_check.elapsed().as_secs() >= 1 {
                    *last_check = Instant::now();
                    match remote {
                        Some(origin) => {
                            // re-fetch from peer
                            let msgs = fetch_remote_messages(origin.tcp_addr, chat_uuid);
                            if !msgs.is_empty() {
                                *groups = group_messages(&msgs);
                                let wrap_w = chars_per_line(win_w, advance);
                                let (nl, nm, nc) = rebuild_view(groups, expanded, wrap_w);
                                *lines = nl;
                                *line_meta = nm;
                                *in_code = nc;
                                *last_wrap_w = wrap_w;
                                let visible = ((win_h as i32) - content_top - PAD) / lh;
                                *scroll = (lines.len() as i32 - visible).max(0);
                            }
                        }
                        None => {
                            let new_mtime = get_file_mtime(path);
                            if new_mtime != *file_mtime {
                                *file_mtime = new_mtime;
                                let msgs = load_messages(path, source_format);
                                *groups = group_messages(&msgs);
                                let wrap_w = chars_per_line(win_w, advance);
                                let (nl, nm, nc) = rebuild_view(groups, expanded, wrap_w);
                                *lines = nl;
                                *line_meta = nm;
                                *in_code = nc;
                                *last_wrap_w = wrap_w;
                                // scroll to bottom
                                let visible = ((win_h as i32) - content_top - PAD) / lh;
                                *scroll = (lines.len() as i32 - visible).max(0);
                            }
                        }
                    }
                }
            }
        }

        // --- LLM auto-naming scan (list mode, every 10s, one chat per tick) ---
        if let AppState::List { transcripts, .. } = &state {
            // check LLM health every 30s
            if llm_last_check.elapsed().as_secs() >= 30 {
                llm_last_check = Instant::now();
                llm_healthy = llm_is_healthy(&llm_endpoint);
            }
            if llm_healthy && auto_name_last.elapsed().as_secs() >= 10 {
                auto_name_last = Instant::now();
                // find first unnamed chat
                let unnamed: Option<TranscriptEntry> = transcripts.iter().find(|t| {
                    !chats.get(&t.uuid).and_then(|m| m.name.as_ref()).is_some()
                }).cloned();
                if let Some(entry) = unnamed {
                    let msgs = load_messages(&entry.path, &entry.source_format);
                    if !msgs.is_empty() {
                        if let Some(name) = llm_auto_name(&llm_endpoint, &msgs) {
                            let meta = chats.entry(entry.uuid).or_insert_with(ChatMeta::default);
                            meta.name = Some(name);
                            meta.auto_named = true;
                            save_chats(&chats);
                        }
                    }
                }
            }
        }

        // --- mouse: selection + click-to-expand in view mode ---
        if let AppState::View {
            lines,
            line_meta,
            scroll,
            sel,
            groups: _,
            expanded,
            in_code: _,
            last_wrap_w: _,
            ..
        } = &mut state
        {
            if let Some((mx, my)) = window.get_mouse_pos(MouseMode::Clamp) {
                let line_idx = ((my as i32 - content_top + *scroll * lh) / lh).max(0) as usize;
                let col_idx = ((mx as i32 - PAD) / advance).max(0) as usize;

                if window.get_mouse_down(MouseButton::Left) {
                    if !mouse_dragging {
                        mouse_dragging = true;
                        sel.active = true;
                        sel.anchor_line = line_idx;
                        sel.anchor_col = col_idx;
                        sel.cursor_line = line_idx;
                        sel.cursor_col = col_idx;
                    } else {
                        sel.cursor_line = line_idx.min(lines.len().saturating_sub(1));
                        sel.cursor_col = col_idx;
                    }
                } else if mouse_dragging {
                    mouse_dragging = false;

                    if sel.is_click() {
                        // check for toggle click
                        let click_line = sel.anchor_line;
                        if let Some(LineMeta::Toggle(gi)) = line_meta.get(click_line) {
                            let gi = *gi;
                            if expanded.contains(&gi) {
                                expanded.remove(&gi);
                            } else {
                                expanded.insert(gi);
                            }
                            needs_rebuild = true;
                        }
                        sel.active = false;
                    } else {
                        // drag completed — copy selection
                        let text = sel.extract_text(lines);
                        if !text.is_empty() {
                            if let Some(ref mut cb) = clipboard {
                                let _ = cb.set_text(&text);
                            }
                        }
                    }
                }
            }
        } else {
            mouse_dragging = false;
        }

        // rebuild lines if toggle was clicked
        if needs_rebuild {
            needs_rebuild = false;
            if let AppState::View {
                groups,
                expanded,
                lines,
                line_meta,
                in_code,
                last_wrap_w,
                ..
            } = &mut state
            {
                let wrap_w = chars_per_line(win_w, advance);
                let (nl, nm, nc) = rebuild_view(groups, expanded, wrap_w);
                *lines = nl;
                *line_meta = nm;
                *in_code = nc;
                *last_wrap_w = wrap_w;
            }
        }

        // --- mouse scroll ---
        if let Some((_sx, sy)) = window.get_scroll_wheel() {
            let scroll_lines = (sy / 3.0) as i32;
            match &mut state {
                AppState::List {
                    filtered,
                    selected,
                    scroll,
                    ..
                } => {
                    if scroll_lines > 0 {
                        *selected = (*selected as i32 - scroll_lines).max(0) as usize;
                    } else {
                        *selected = (*selected as i32 - scroll_lines)
                            .min(filtered.len() as i32 - 1)
                            .max(0) as usize;
                    }
                    let visible_lines = ((win_h as i32) - content_top - PAD) / lh;
                    let total_lines = (filtered.len() as i32) * LIST_LINES_PER_ITEM;
                    *scroll = ((*selected as i32) * LIST_LINES_PER_ITEM - visible_lines / 2)
                        .max(0)
                        .min((total_lines - visible_lines).max(0));
                }
                AppState::View { lines, scroll, .. } => {
                    let delta = -scroll_lines * 3;
                    let visible = ((win_h as i32) - content_top - PAD) / lh;
                    let max_scroll = (lines.len() as i32 - visible).max(0);
                    *scroll = (*scroll + delta).max(0).min(max_scroll);
                }
            }
        }

        // --- keyboard ---
        let ctrl = window.is_key_down(Key::LeftCtrl) || window.is_key_down(Key::RightCtrl);
        let shift = window.is_key_down(Key::LeftShift) || window.is_key_down(Key::RightShift);

        for key in window.get_keys_pressed(KeyRepeat::Yes) {
            if ctrl {
                match key {
                    Key::Equal | Key::NumPadPlus => {
                        size_idx = (size_idx + 1).min(FONT_SIZES.len() - 1);
                        atlas = FontAtlas::with_font(FONT_SIZES[size_idx], &font_name, font_weight);
                        if let AppState::View { last_wrap_w, .. } = &mut state {
                            *last_wrap_w = 0; // force re-wrap
                        }
                        continue;
                    }
                    Key::Minus | Key::NumPadMinus => {
                        size_idx = size_idx.saturating_sub(1);
                        atlas = FontAtlas::with_font(FONT_SIZES[size_idx], &font_name, font_weight);
                        if let AppState::View { last_wrap_w, .. } = &mut state {
                            *last_wrap_w = 0;
                        }
                        continue;
                    }
                    Key::C => {
                        if let AppState::View { lines, sel, .. } = &state {
                            let text = if sel.active {
                                sel.extract_text(lines)
                            } else {
                                lines.join("\n")
                            };
                            if let Some(ref mut cb) = clipboard {
                                let _ = cb.set_text(&text);
                            }
                        }
                        continue;
                    }
                    Key::A => {
                        if let AppState::View { lines, sel, .. } = &mut state {
                            sel.active = true;
                            sel.anchor_line = 0;
                            sel.anchor_col = 0;
                            sel.cursor_line = lines.len().saturating_sub(1);
                            sel.cursor_col = lines.last().map_or(0, |l| l.chars().count());
                        }
                        continue;
                    }
                    _ => {}
                }
            }

            match &mut state {
                AppState::List {
                    transcripts,
                    filtered,
                    selected,
                    scroll,
                    searching,
                    search_term,
                    favs_only,
                    renaming,
                    rename_buf,
                    ..
                } => {
                    if *renaming {
                        match key {
                            Key::Escape => {
                                *renaming = false;
                                rename_buf.clear();
                            }
                            Key::Enter => {
                                if let Some(entry) = filtered.get(*selected) {
                                    let uuid = entry.uuid.clone();
                                    let meta = chats.entry(uuid).or_insert_with(ChatMeta::default);
                                    if rename_buf.trim().is_empty() {
                                        meta.name = None;
                                        meta.auto_named = false;
                                    } else {
                                        meta.name = Some(rename_buf.trim().to_string());
                                        meta.auto_named = false;
                                    }
                                    save_chats(&chats);
                                }
                                *renaming = false;
                                rename_buf.clear();
                            }
                            Key::Backspace => {
                                rename_buf.pop();
                            }
                            other => {
                                if let Some(ch) = key_to_char(other, shift) {
                                    rename_buf.push(ch);
                                }
                            }
                        }
                    } else if *searching {
                        match key {
                            Key::Escape => {
                                *searching = false;
                                search_term.clear();
                                *filtered = filter_transcripts_quick(
                                    transcripts,
                                    "",
                                    *favs_only,
                                    &chats,
                                );
                                *selected = 0;
                                *scroll = 0;
                            }
                            Key::Enter => {
                                *searching = false;
                                *filtered = filter_transcripts_deep(
                                    transcripts,
                                    search_term,
                                    *favs_only,
                                    &chats,
                                );
                                *selected = 0;
                                *scroll = 0;
                            }
                            Key::Backspace => {
                                search_term.pop();
                                *filtered = filter_transcripts_quick(
                                    transcripts,
                                    search_term,
                                    *favs_only,
                                    &chats,
                                );
                                *selected = 0;
                                *scroll = 0;
                            }
                            other => {
                                if let Some(ch) = key_to_char(other, shift) {
                                    search_term.push(ch);
                                    *filtered = filter_transcripts_quick(
                                        transcripts,
                                        search_term,
                                        *favs_only,
                                        &chats,
                                    );
                                    *selected = 0;
                                    *scroll = 0;
                                }
                            }
                        }
                    } else {
                        match key {
                            Key::Escape => {
                                if show_help {
                                    show_help = false;
                                } else {
                                    return;
                                }
                            }
                            Key::Slash if shift => {
                                // ? key
                                show_help = !show_help;
                            }
                            Key::N => {
                                *renaming = true;
                                rename_buf.clear();
                                // pre-fill with existing name if any
                                if let Some(entry) = filtered.get(*selected) {
                                    if let Some(meta) = chats.get(&entry.uuid) {
                                        if let Some(ref name) = meta.name {
                                            *rename_buf = name.clone();
                                        }
                                    }
                                }
                            }
                            Key::S => {
                                *searching = true;
                                search_term.clear();
                            }
                            Key::F if shift => {
                                *favs_only = !*favs_only;
                                *filtered = filter_transcripts_quick(
                                    transcripts,
                                    search_term,
                                    *favs_only,
                                    &chats,
                                );
                                *selected = 0;
                                *scroll = 0;
                            }
                            Key::F => {
                                if let Some(entry) = filtered.get(*selected) {
                                    let uuid = entry.uuid.clone();
                                    let meta = chats.entry(uuid).or_insert_with(ChatMeta::default);
                                    meta.starred = !meta.starred;
                                    save_chats(&chats);
                                    if *favs_only {
                                        *filtered = filter_transcripts_quick(
                                            transcripts,
                                            search_term,
                                            true,
                                            &chats,
                                        );
                                        *selected =
                                            (*selected).min(filtered.len().saturating_sub(1));
                                    }
                                }
                            }
                            Key::Up => {
                                *selected = (*selected as i32 - 1).max(0) as usize;
                            }
                            Key::Down => {
                                *selected =
                                    (*selected + 1).min(filtered.len().saturating_sub(1));
                            }
                            Key::Enter => {
                                if let Some(entry) = filtered.get(*selected).cloned() {
                                    let fmt = entry.source_format.clone();
                                    let msgs = match &entry.remote {
                                        Some(origin) => fetch_remote_messages(origin.tcp_addr, &entry.uuid),
                                        None => load_messages(&entry.path, &fmt),
                                    };
                                    let groups = group_messages(&msgs);
                                    let expanded = HashSet::new();
                                    let wrap_w = chars_per_line(win_w, advance);
                                    let (lines, line_meta, in_code) =
                                        rebuild_view(&groups, &expanded, wrap_w);
                                    // scroll to bottom
                                    let visible = ((win_h as i32) - content_top - PAD) / lh;
                                    let scroll = (lines.len() as i32 - visible).max(0);
                                    let mtime = get_file_mtime(&entry.path);
                                    let chat_uuid = entry.uuid.clone();
                                    transition = Some(AppState::View {
                                        path: entry.path,
                                        source_format: fmt,
                                        groups,
                                        expanded,
                                        lines,
                                        line_meta,
                                        in_code,
                                        last_wrap_w: wrap_w,
                                        scroll,
                                        sel: Selection::default(),
                                        file_mtime: mtime,
                                        last_check: Instant::now(),
                                        remote: entry.remote.clone(),
                                        chat_uuid,
                                    });
                                }
                            }
                            _ => {}
                        }
                    }
                    let visible_lines = ((win_h as i32) - content_top - PAD) / lh;
                    let total_lines = (filtered.len() as i32) * LIST_LINES_PER_ITEM;
                    *scroll = ((*selected as i32) * LIST_LINES_PER_ITEM - visible_lines / 2)
                        .max(0)
                        .min((total_lines - visible_lines).max(0));
                }
                AppState::View {
                    path: _, source_format: _, lines, scroll, sel, chat_uuid, ..
                } => match key {
                    Key::Escape => {
                        if show_help {
                            show_help = false;
                        } else if sel.active {
                            sel.active = false;
                        } else {
                            let all = get_all_transcripts(&sources);
                            transition = Some(AppState::List {
                                filtered: all.clone(),
                                transcripts: all,
                                selected: 0,
                                scroll: 0,
                                searching: false,
                                search_term: String::new(),
                                favs_only: false,
                                last_scan: Instant::now(),
                                renaming: false,
                                rename_buf: String::new(),
                            });
                        }
                    }
                    Key::Left | Key::Right => {
                        // navigate to prev/next chat using last_merged list
                        let all = last_merged.clone();
                        if let Some(idx) = all.iter().position(|t| t.uuid == *chat_uuid) {
                            let next_idx = if key == Key::Left {
                                if idx == 0 { all.len() - 1 } else { idx - 1 }
                            } else {
                                if idx + 1 >= all.len() { 0 } else { idx + 1 }
                            };
                            if let Some(entry) = all.get(next_idx).cloned() {
                                let fmt = entry.source_format.clone();
                                let msgs = match &entry.remote {
                                    Some(origin) => fetch_remote_messages(origin.tcp_addr, &entry.uuid),
                                    None => load_messages(&entry.path, &fmt),
                                };
                                let groups = group_messages(&msgs);
                                let expanded = HashSet::new();
                                let wrap_w = chars_per_line(win_w, advance);
                                let (lines, line_meta, in_code) =
                                    rebuild_view(&groups, &expanded, wrap_w);
                                let visible = ((win_h as i32) - content_top - PAD) / lh;
                                let scroll = (lines.len() as i32 - visible).max(0);
                                let mtime = get_file_mtime(&entry.path);
                                let nav_uuid = entry.uuid.clone();
                                transition = Some(AppState::View {
                                    path: entry.path,
                                    source_format: fmt,
                                    groups,
                                    expanded,
                                    lines,
                                    line_meta,
                                    in_code,
                                    last_wrap_w: wrap_w,
                                    scroll,
                                    sel: Selection::default(),
                                    file_mtime: mtime,
                                    last_check: Instant::now(),
                                    remote: entry.remote.clone(),
                                    chat_uuid: nav_uuid,
                                });
                            }
                        }
                    }
                    Key::Slash if shift => {
                        // ? key
                        show_help = !show_help;
                    }
                    Key::Up => *scroll = (*scroll - 1).max(0),
                    Key::Down => {
                        let visible = ((win_h as i32) - content_top - PAD) / lh;
                        let max_scroll = (lines.len() as i32 - visible).max(0);
                        *scroll = (*scroll + 1).min(max_scroll);
                    }
                    Key::PageUp => *scroll = (*scroll - 15).max(0),
                    Key::PageDown => {
                        let visible = ((win_h as i32) - content_top - PAD) / lh;
                        let max_scroll = (lines.len() as i32 - visible).max(0);
                        *scroll = (*scroll + 15).min(max_scroll);
                    }
                    Key::Home => *scroll = 0,
                    Key::End => {
                        let visible = ((win_h as i32) - content_top - PAD) / lh;
                        *scroll = (lines.len() as i32 - visible).max(0);
                    }
                    _ => {}
                },
            }
        }

        // re-wrap if needed
        if let AppState::View {
            groups,
            expanded,
            lines,
            line_meta,
            in_code,
            last_wrap_w,
            ..
        } = &mut state
        {
            let wrap_w = chars_per_line(win_w, advance);
            if wrap_w != *last_wrap_w {
                let (nl, nm, nc) = rebuild_view(groups, expanded, wrap_w);
                *lines = nl;
                *line_meta = nm;
                *in_code = nc;
                *last_wrap_w = wrap_w;
            }
        }

        // ========== RENDER ==========

        // content
        match &state {
            AppState::List {
                filtered: list,
                selected,
                scroll,
                ..
            } => {
                let total_lines = (list.len() as i32) * LIST_LINES_PER_ITEM;
                let mut y = content_top - *scroll * lh;
                for line_idx in 0..total_lines {
                    if y + lh < content_top || y > win_h as i32 {
                        y += lh;
                        continue;
                    }
                    let i = (line_idx / LIST_LINES_PER_ITEM) as usize;
                    let sub = line_idx % LIST_LINES_PER_ITEM;
                    let t = match list.get(i) {
                        Some(x) => x,
                        None => continue,
                    };
                    let is_selected = i == *selected;
                    if is_selected && sub == 0 {
                        fill_rect(
                            &mut buffer,
                            0,
                            y,
                            win_w as i32,
                            lh * 3,
                            c_sel,
                            buf_w,
                            buf_h,
                        );
                    }
                    match sub {
                        0 => {
                            // Left side: star + display name (or preview fallback)
                            let star = if chats.get(&t.uuid).map_or(false, |m| m.starred) {
                                "* "
                            } else {
                                "  "
                            };
                            let display_name = chats.get(&t.uuid).and_then(|m| m.name.as_ref());
                            let left_text = match display_name {
                                Some(name) => format!("{}{}", star, name),
                                None => {
                                    // truncate preview for left side
                                    let max_left = chars_per_line(win_w, advance) / 2;
                                    let prev: String = t.preview.chars().take(max_left).collect();
                                    format!("{}{}", star, prev)
                                }
                            };
                            draw_text_ttf(
                                &mut buffer,
                                PAD,
                                y,
                                &left_text,
                                if is_selected { c_text } else { c_dim },
                                &mut atlas,
                                buf_w,
                                buf_h,
                            );

                            // Right side: [hostname · platform]  [hash]  [timestamp]
                            let project_display = match &t.remote {
                                Some(origin) => format!("{} · {}", origin.hostname, t.project),
                                None => t.project.clone(),
                            };
                            let uuid_short = if t.uuid.len() >= 8 {
                                &t.uuid[..8]
                            } else {
                                &t.uuid
                            };
                            let right = format!(
                                "{}  {}  {}",
                                project_display, uuid_short, t.timestamp
                            );
                            let right_w = right.len() as i32 * advance;
                            draw_text_ttf(
                                &mut buffer,
                                win_w as i32 - PAD - right_w,
                                y,
                                &right,
                                c_timestamp,
                                &mut atlas,
                                buf_w,
                                buf_h,
                            );
                        }
                        1 => {
                            // If entry has a name, show preview underneath; otherwise skip (preview is already on line 0)
                            if chats.get(&t.uuid).and_then(|m| m.name.as_ref()).is_some() {
                                let max_prev = chars_per_line(win_w, advance).saturating_sub(5);
                                let prev = if t.preview.chars().count() > max_prev {
                                    format!(
                                        "     {}..",
                                        t.preview.chars().take(max_prev).collect::<String>()
                                    )
                                } else {
                                    format!("     {}", t.preview)
                                };
                                draw_text_ttf(
                                    &mut buffer,
                                    PAD,
                                    y,
                                    &prev,
                                    if is_selected {
                                        c_dim
                                    } else {
                                        c_msg_time
                                    },
                                    &mut atlas,
                                    buf_w,
                                    buf_h,
                                );
                            }
                        }
                        2 => {} // spacer
                        _ => {
                            let sep_y = y + lh / 2;
                            fill_rect(
                                &mut buffer,
                                PAD,
                                sep_y,
                                (win_w as i32) - 2 * PAD,
                                1,
                                c_sep,
                                buf_w,
                                buf_h,
                            );
                        }
                    }
                    y += lh;
                }
            }
            AppState::View {
                lines,
                line_meta,
                in_code,
                scroll,
                sel,
                ..
            } => {
                let mut y = content_top - *scroll * lh;
                let ((sel_sl, sel_sc), (sel_el, sel_ec)) = sel.ordered();
                for (li, line) in lines.iter().enumerate() {
                    if y + lh < content_top || y > win_h as i32 {
                        y += lh;
                        continue;
                    }

                    // selection highlight
                    if sel.active && li >= sel_sl && li <= sel_el {
                        let chars_count = line.chars().count();
                        let start_col = if li == sel_sl {
                            sel_sc.min(chars_count)
                        } else {
                            0
                        };
                        let end_col = if li == sel_el {
                            sel_ec.min(chars_count)
                        } else {
                            chars_count
                        };
                        if end_col > start_col {
                            let hx = PAD + start_col as i32 * advance;
                            let hw = (end_col - start_col) as i32 * advance;
                            fill_rect(
                                &mut buffer, hx, y, hw, lh, c_select_bg, buf_w, buf_h,
                            );
                        }
                    }

                    let meta = line_meta.get(li);

                    // timestamp lines
                    if matches!(meta, Some(LineMeta::Timestamp)) {
                        draw_text_ttf(
                            &mut buffer, PAD, y, line, c_msg_time, &mut atlas, buf_w, buf_h,
                        );
                        y += lh;
                        continue;
                    }

                    // toggle lines (clickable)
                    if matches!(meta, Some(LineMeta::Toggle(_))) {
                        draw_text_ttf(
                            &mut buffer, PAD, y, line, c_toggle, &mut atlas, buf_w, buf_h,
                        );
                        y += lh;
                        continue;
                    }

                    // code block lines
                    if *in_code.get(li).unwrap_or(&false) {
                        let usable_w = (win_w as i32) - 2 * PAD;
                        fill_rect(
                            &mut buffer,
                            PAD - 2,
                            y,
                            usable_w + 4,
                            lh,
                            c_code_bg,
                            buf_w,
                            buf_h,
                        );
                        draw_text_ttf(
                            &mut buffer, PAD, y, line, c_code, &mut atlas, buf_w, buf_h,
                        );
                        y += lh;
                        continue;
                    }

                    // determine base color from role prefix
                    let base_color = if line.starts_with("[user] ") {
                        c_user
                    } else if line.starts_with("[asst] ") {
                        c_asst
                    } else {
                        c_text
                    };

                    // check for headers (# or ##)
                    let trimmed = line.trim_start();
                    if trimmed.starts_with("## ") || trimmed.starts_with("# ") {
                        draw_text_ttf(
                            &mut buffer, PAD, y, line, c_heading, &mut atlas, buf_w, buf_h,
                        );
                        y += lh;
                        continue;
                    }

                    // inline markdown parsing
                    let runs = parse_inline_markdown(line, base_color);
                    draw_styled_runs(&mut buffer, PAD, y, &runs, &mut atlas, buf_w, buf_h);
                    y += lh;
                }
            }
        }

        // ===== HEADER (drawn OVER content) =====
        fill_rect(
            &mut buffer, 0, 0, win_w as i32, HEADER_H, c_header_bg, buf_w, buf_h,
        );
        fill_rect(
            &mut buffer, 0, HEADER_H, win_w as i32, 1, c_sep, buf_w, buf_h,
        );
        {
            let header_y = (HEADER_H - lh) / 2;
            match &state {
                AppState::List {
                    filtered,
                    searching,
                    search_term,
                    favs_only,
                    renaming,
                    rename_buf,
                    ..
                } => {
                    let title = if *favs_only {
                        "chat-daddy *favs*"
                    } else {
                        "chat-daddy"
                    };
                    draw_text_ttf(
                        &mut buffer, PAD, header_y, title, c_accent, &mut atlas, buf_w, buf_h,
                    );
                    if *renaming {
                        let prompt = format!("name: {}_", rename_buf);
                        let prompt_w = prompt.len() as i32 * advance;
                        let sx = win_w as i32 - PAD - prompt_w - 8;
                        fill_rect(
                            &mut buffer, sx - 4, 2, prompt_w + 12, HEADER_H - 4, c_search_bg,
                            buf_w, buf_h,
                        );
                        draw_text_ttf(
                            &mut buffer, sx, header_y, &prompt, c_text, &mut atlas, buf_w, buf_h,
                        );
                    } else if *searching {
                        let prompt = format!("/{}_  [Enter]deep", search_term);
                        let prompt_w = prompt.len() as i32 * advance;
                        let sx = win_w as i32 - PAD - prompt_w - 8;
                        fill_rect(
                            &mut buffer, sx - 4, 2, prompt_w + 12, HEADER_H - 4, c_search_bg,
                            buf_w, buf_h,
                        );
                        draw_text_ttf(
                            &mut buffer, sx, header_y, &prompt, c_text, &mut atlas, buf_w, buf_h,
                        );
                    } else {
                        let count_str = format!("{} chats  ?", filtered.len());
                        let hint_w = count_str.len() as i32 * advance;
                        draw_text_ttf(
                            &mut buffer,
                            win_w as i32 - PAD - hint_w,
                            header_y,
                            &count_str,
                            c_dim,
                            &mut atlas,
                            buf_w,
                            buf_h,
                        );
                    }
                }
                AppState::View { path, chat_uuid, remote, .. } => {
                    let title = path
                        .file_stem()
                        .map(|s| s.to_string_lossy())
                        .unwrap_or_default();
                    // show chat name if available, else uuid
                    let uuid_str = if chat_uuid.is_empty() { title.to_string() } else { chat_uuid.clone() };
                    let _remote_label = remote.as_ref().map(|o| o.hostname.clone());
                    let display = chats
                        .get(&uuid_str)
                        .and_then(|m| m.name.as_deref())
                        .unwrap_or(&uuid_str);
                    draw_text_ttf(
                        &mut buffer,
                        PAD,
                        header_y,
                        &format!("chat-daddy > {}", display),
                        c_accent,
                        &mut atlas,
                        buf_w,
                        buf_h,
                    );
                    let hint = "[</>] ?";
                    let hint_w = hint.len() as i32 * advance;
                    draw_text_ttf(
                        &mut buffer,
                        win_w as i32 - PAD - hint_w,
                        header_y,
                        hint,
                        c_dim,
                        &mut atlas,
                        buf_w,
                        buf_h,
                    );
                }
            }
        }

        // ===== HELP OVERLAY =====
        if show_help {
            let help_lines: &[&str] = match &state {
                AppState::List { .. } => &[
                    "HOTKEYS — LIST",
                    "",
                    "  Up/Down      navigate",
                    "  Enter        open chat",
                    "  Left/Right   (in view) prev/next chat",
                    "  S            search",
                    "  N            rename chat",
                    "  F            star/unstar",
                    "  Shift+F      toggle favs filter",
                    "  Ctrl+/−      font size",
                    "  Esc          quit",
                    "  ?            toggle this help",
                ],
                AppState::View { .. } => &[
                    "HOTKEYS — VIEW",
                    "",
                    "  Up/Down      scroll",
                    "  PgUp/PgDn    fast scroll",
                    "  Home/End     jump top/bottom",
                    "  Left/Right   prev/next chat",
                    "  Ctrl+C       copy selection",
                    "  Ctrl+A       select all",
                    "  Ctrl+/−      font size",
                    "  Esc          back to list",
                    "  ?            toggle this help",
                ],
            };
            let panel_w = 40 * advance;
            let panel_h = (help_lines.len() as i32 + 2) * lh;
            let px = (win_w as i32 - panel_w) / 2;
            let py = (win_h as i32 - panel_h) / 2;
            // dark background
            fill_rect(&mut buffer, px - PAD, py - PAD, panel_w + 2 * PAD, panel_h + 2 * PAD, c_header_bg, buf_w, buf_h);
            // border
            fill_rect(&mut buffer, px - PAD, py - PAD, panel_w + 2 * PAD, 1, c_sep, buf_w, buf_h);
            fill_rect(&mut buffer, px - PAD, py + panel_h + PAD, panel_w + 2 * PAD, 1, c_sep, buf_w, buf_h);
            fill_rect(&mut buffer, px - PAD, py - PAD, 1, panel_h + 2 * PAD, c_sep, buf_w, buf_h);
            fill_rect(&mut buffer, px + panel_w + PAD, py - PAD, 1, panel_h + 2 * PAD, c_sep, buf_w, buf_h);
            for (i, line) in help_lines.iter().enumerate() {
                let color = if i == 0 { c_accent } else { c_text };
                draw_text_ttf(
                    &mut buffer, px, py + (i as i32) * lh, line, color, &mut atlas, buf_w, buf_h,
                );
            }
        }

        if snapshot {
            let _ = dump_png("debug-frame.png", &buffer, buf_w as u32, buf_h as u32);
            break;
        }

        window.update_with_buffer(&buffer, win_w, win_h).unwrap();
    }
}

fn dump_png(path: &str, buf: &[u32], w: u32, h: u32) -> Result<(), image::ImageError> {
    let mut out = Vec::with_capacity((w * h * 3) as usize);
    for &pix in buf {
        let r = ((pix >> 16) & 0xFF) as u8;
        let g = ((pix >> 8) & 0xFF) as u8;
        let b = (pix & 0xFF) as u8;
        out.push(r);
        out.push(g);
        out.push(b);
    }
    save_buffer(path, &out, w, h, ColorType::Rgb8)
}
