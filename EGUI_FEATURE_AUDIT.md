# EGUI Client - Feature Audit for Blazor Implementation

**Date:** 2026-01-05 (Updated)  
**Purpose:** Comprehensive feature audit of the egui reference client to guide Blazor/Tauri implementation  
**Source:** `submodules/opencode-egui/`

---

## Executive Summary

The egui client is a **production-grade desktop chat client** with sophisticated features including:

### Core Architecture
- Multi-tab session management with server-backed persistence
- Real-time SSE event streaming for incremental message updates
- HTTP/gRPC-ready API client with OAuth token injection

### Chat Features
- Advanced tool execution visualization with collapsible blocks
- Permission system (approve/reject/always allow) with inline dialogs
- Markdown rendering with code syntax highlighting
- Reasoning display (collapsible extended thinking sections)
- Message cancellation with smart event filtering

### Agent & Model Management
- **Left sidebar Agents Pane** with visual indicators (color dots, badges)
- Agent filtering (show/hide subagents)
- Per-tab agent selection with smart fallback
- **Dynamic model discovery** from provider APIs (OpenAI, Anthropic, Google, OpenRouter)
- Provider configuration via TOML with flexible auth (bearer, header, query param)
- Curated models list with search/filter UI

### Authentication & Settings
- API key sync to server (.env → server)
- OAuth mode toggle (Anthropic) with expiry countdown
- Comprehensive settings panel (Server, UI, Models)
- Config persistence (platform-specific app data + models.toml)

### Additional Features (Session 8-9)
- Tab rename (context menu)
- Clipboard image paste
- OAuth countdown timer
- Push-to-talk audio transcription (Whisper)

**Complexity Level:** High - This is not a basic chat UI. It's a full-featured IDE-like client with dynamic configuration.

**Goal:** Full feature parity with egui reference implementation.

---

## Part 1: Core Architecture

### 1.1 Server Communication

**HTTP Client** (`src/client/api.rs`):
- ✅ `OpencodeClient` - reqwest-based HTTP client
- ✅ Base URL management (discovery or manual override)
- ✅ Directory header support (`x-opencode-directory`)
- ✅ OAuth token injection (`Authorization: Bearer {token}`)
- ✅ 30-second request timeout

**API Endpoints Used**:
```rust
GET  /doc                           // Server documentation
GET  /session                       // List all sessions
POST /session                       // Create new session
DELETE /session/{id}                // Delete session
POST /session/{id}/message          // Send message with parts
POST /session/{id}/abort            // Cancel active response
POST /session/{id}/permissions/{id} // Respond to permission request
GET  /agent                         // List available agents
GET  /provider                      // Get provider connection status
PUT  /auth/{provider}               // Sync API keys or OAuth tokens
POST /instance/dispose              // Reload server state (after auth change)
```

**SSE Event Streaming** (`src/client/events.rs`):
- ✅ `subscribe_global(base_url)` - SSE subscription to `/global/event`
- ✅ Tokio channel-based event delivery to UI thread
- ✅ Graceful reconnection handling
- ✅ Event types:
  - `message.updated` - New message started/finished
  - `message.part.updated` - Text/reasoning/tool part updates
  - `permission.updated` - New permission request
  - `permission.replied` - Permission was answered

**Discovery & Spawning** (`src/discovery/`):
- ✅ `discover()` - Check for running server on known ports (4008-4018)
- ✅ `check_health(base_url)` - HTTP GET to `/doc` to verify server
- ✅ `spawn_and_wait()` - Launch `opencode server` and wait for ready
- ✅ `stop_pid(pid)` - Kill owned server process on exit
- ✅ Port override support for testing

### 1.2 State Management

**App State** (`OpenCodeApp` struct):
```rust
// Multi-tab sessions
tabs: Vec<Tab>              // Each tab = independent session
active: usize               // Currently visible tab index

// Server connection
server: Option<ServerInfo>  // Connected server (url, pid, owned)
client: Option<OpencodeClient>
oauth_token: Option<String>

// Async runtime
runtime: Option<Arc<Runtime>>  // Tokio runtime for background tasks
ui_rx/ui_tx: mpsc channels     // Message passing from async → UI thread

// Auth state
auth_sync_state: AuthSyncState
anthropic_subscription_mode: bool
anthropic_oauth_expires: Option<u64>

// Agents
agents: Vec<AgentInfo>
default_agent: String
show_subagents: bool

// Permissions
pending_permissions: Vec<PermissionInfo>

// Audio (STT)
audio_tx: Option<mpsc::Sender<AudioCmd>>
recording_state: RecordingState

// Config
config: AppConfig
models_config: ModelsConfig

// UI state
show_settings: bool
show_model_discovery: bool
renaming_tab: Option<usize>
```

**Tab State** (`Tab` struct):
```rust
title: String                      // Tab display name (editable)
session_id: Option<String>         // Server-backed session ID
directory: Option<String>          // Working directory for this session
messages: Vec<DisplayMessage>      // Chat history
active_assistant: Option<String>   // ID of streaming message
input: String                      // User input buffer
selected_model: Option<(String, String)> // (provider, model_id)
selected_agent: Option<String>     // Selected agent name
cancelled_messages: Vec<String>    // IDs of cancelled messages
cancelled_calls: Vec<String>       // IDs of cancelled tool calls
pending_attachments: Vec<PendingAttachment> // Images to send
suppress_incoming: bool            // Block events after cancel
```

**Message State** (`DisplayMessage`):
```rust
message_id: String
role: String                       // "user" | "assistant" | "system"
text_parts: Vec<String>            // Accumulated text (SSE updates)
reasoning_parts: Vec<String>       // Extended thinking text
tokens_input: Option<u64>
tokens_output: Option<u64>
tokens_reasoning: Option<u64>
tool_calls: Vec<ToolCall>          // Tool executions in this message
```

**Tool Call State** (`ToolCall`):
```rust
id: String                         // Tool part ID
name: String                       // Tool name (e.g., "bash", "read")
status: String                     // "pending" | "running" | "success" | "error" | "cancelled"
call_id: Option<String>            // Execution call ID (for permissions)
input: serde_json::Value           // Tool parameters
output: Option<String>             // Tool result
error: Option<String>              // Error message if failed
logs: Vec<String>                  // Streaming tool logs
metadata: serde_json::Map          // Extra tool metadata
started_at: Option<i64>
finished_at: Option<i64>
```

---

## Part 2: Feature Breakdown by Category

### 2.1 Session Management ⭐⭐⭐ (MVP)

**Multi-Tab Support**:
- ✅ Create new tab (`+` button)
- ✅ Switch between tabs (tab bar)
- ✅ Close tab (`X` button)
- ✅ Rename tab (right-click context menu)
- ✅ Each tab = independent session on server
- ✅ Auto-create first tab on startup

**Session Lifecycle**:
- ✅ Create session on tab creation (`POST /session`)
- ✅ Delete session on tab close (`DELETE /session/{id}`)
- ✅ Delete all other sessions (cleanup button in settings)
- ✅ Session versioning (displayed in footer)

**Implementation Notes**:
- Session creation is async - tab shows "(creating…)" until ready
- Session ID is required before user can send messages
- Each session has its own message history (not persisted locally)

### 2.2 Messaging ⭐⭐⭐ (MVP)

**Send Message**:
- ✅ Text input (multiline, resizable)
- ✅ Send button (enabled when input not empty)
- ✅ Keyboard shortcut (Cmd+Enter on macOS)
- ✅ Attachment support (see below)
- ✅ Request body:
  ```json
  {
    "parts": [
      { "type": "text", "text": "..." },
      { "type": "file", "mime": "image/png", "url": "data:image/png;base64,..." }
    ],
    "model": { "providerID": "...", "modelID": "..." },
    "agent": "build"
  }
  ```

**Receive Messages (SSE Stream)**:
- ✅ `message.updated` event → Create new `DisplayMessage`
- ✅ `message.part.updated` (type=text) → Update `text_parts`
- ✅ `message.part.updated` (type=reasoning) → Update `reasoning_parts`
- ✅ `message.part.updated` (type=tool) → Update/create `ToolCall`
- ✅ Token counts displayed when message finishes
- ✅ Streaming indicator (spinner while waiting for text)

**Cancel/Abort**:
- ✅ "Stop" button appears during streaming
- ✅ Cancels active assistant message
- ✅ Cancels all in-flight tool calls
- ✅ Marks cancelled tools as "cancelled" status
- ✅ Sends `POST /session/{id}/abort` to server (twice, 200ms apart)

**Event Filtering** (Complex!):
- ✅ Ignore events for cancelled messages (`cancelled_messages` list)
- ✅ Ignore events for cancelled tool calls (`cancelled_calls` list)
- ✅ Ignore events older than `cancelled_after` timestamp
- ✅ Ignore events older than `last_send_at` (prevents stale events)
- ✅ `suppress_incoming` flag blocks all events except assistant text

### 2.3 Tool Call Visualization ⭐⭐⭐ (MVP)

**Tool Display** (Collapsible Block):
- ✅ Header shows: `[status_icon] (tool_name) - [command_summary] [duration]`
- ✅ Status icons: ✅ success, ❌ error, 🚫 cancelled, ⏳ running
- ✅ Command summary extracts: `command`, `filePath`, `url`, `prompt` from input
- ✅ Duration displayed when `started_at` and `finished_at` present
- ✅ Horizontal scrolling for long paths

**Tool Body** (Expanded):
- ✅ **COMMAND** section (if tool has `command` input)
- ✅ **INPUT** section (JSON-formatted remaining parameters)
- ✅ **OUTPUT** section (scrollable, max 300px height, monospace)
- ✅ **ERROR** section (red text)
- ✅ **LOGS** section (scrollable, max 150px height, monospace)

**Auto-Expand Behavior**:
- ✅ Auto-expand when: `is_running || has_permission || has_error`
- ✅ Otherwise collapsed by default

### 2.4 Permission System ⭐⭐ (Core)

**Permission Requests**:
- ✅ Received via `permission.updated` SSE event
- ✅ Stored in `pending_permissions: Vec<PermissionInfo>`
- ✅ Displayed inline in tool call header (red warning box)
- ✅ Auto-rejected if message/call already cancelled

**Permission UI**:
- ✅ "❌ Reject" button
- ✅ "✅ Allow Once" button  
- ✅ "✅ Always Allow" button
- ✅ Sends `POST /session/{id}/permissions/{perm_id}` with response: "reject" | "once" | "always"
- ✅ Removed from `pending_permissions` after response

**Input Blocking**:
- ✅ Send button disabled when pending permission exists for active tab's session
- ✅ Input area disabled

### 2.5 Agent Selection ⭐⭐⭐ (MVP - Critical UI Component)

**Visual Layout:**
```
┌─────────────────────────┬─────────────────────────────────┐
│ Agents         [▾]      │ Message Area                    │
├─────────────────────────┤                                 │
│ ☑ build         ⬤      │ (chat messages)                 │
│   built-in             │                                 │
│                        │                                 │
│ ☐ expert-developer ⬤   │                                 │
│   built-in             │                                 │
│                        │                                 │
│ ☐ qa-engineer     ⬤    │                                 │
│   built-in             │                                 │
└─────────────────────────┴─────────────────────────────────┘
```

**Agent Fetching:**
- ✅ Fetch on server connect: `GET /agent`
- ✅ Response structure:
  ```json
  [
    {
      "name": "build",
      "description": "General-purpose build agent",
      "mode": null,
      "builtIn": true,
      "color": "#3b82f6"
    },
    {
      "name": "task",
      "description": "Specialized task agent",
      "mode": "subagent",
      "builtIn": true,
      "color": "#10b981"
    }
  ]
  ```

**Agent Filtering:**
- ✅ Default: Hide subagents (show only `mode != "subagent"`)
- ✅ Toggle in Settings → UI Preferences → "Show subagents in agent list"
- ✅ When toggled:
  - Re-filter agent list
  - Update default agent to first primary agent
  - Reset tabs with hidden agents to default

**Left Sidebar Pane:**
- ✅ Collapsible (▸/▾ toggle button)
- ✅ Agent list with visual indicators:
  - Checkbox (☑ selected / ☐ unselected)
  - Agent name (grayed if subagent)
  - Color dot (⬤) from hex color `#rrggbb`
  - "built-in" badge
  - "subagent" badge
- ✅ Click agent to select for active tab
- ✅ Per-tab selection (each tab remembers agent)
- ✅ Default agent fallback (first primary agent or "build")

**Footer Display:**
- ✅ Shows current agent: `agent: expert-developer`
- ✅ Visible even when pane collapsed

**Sent with Message:**
```json
POST /session/{id}/message
{
  "parts": [...],
  "model": {...},
  "agent": "expert-developer"
}
```

**Smart Agent Management:**
```rust
// Ensure tab has valid agent (runs on filter toggle, agent load, tab creation)
fn ensure_tab_agent(default: &str, tab: &mut Tab, filtered: &[AgentInfo]) {
    if let Some(name) = tab.selected_agent.clone() {
        if filtered.iter().any(|a| a.name == name) {
            return;  // Keep if still valid
        }
    }
    tab.selected_agent = Some(default.to_string());  // Use default
}
```

### 2.6 Model Selection & Discovery ⭐⭐⭐ (Core - Not Advanced!)

**Model Management:**
- ✅ Curated models list (persisted in `config/models.toml`)
- ✅ Default model for new tabs (configurable)
- ✅ Per-tab model override
- ✅ **Model discovery UI** (provider → search → add to curated list)

**Model Selector** (Footer ComboBox):
- ✅ Show current model name (e.g., "Claude 3.5 Sonnet")
- ✅ Show "(use default)" option
- ✅ Show all curated models
- ✅ Show "🟢 (Subscription)" indicator for OAuth models
- ✅ "⚙ Manage Models" button → opens settings

**Provider Configuration** (`config/models.toml`):
```toml
[[providers]]
name = "openai"
display_name = "OpenAI"
api_key_env = "OPENAI_API_KEY"
models_url = "https://api.openai.com/v1/models"
auth_type = "bearer"  # or "header" or "query_param"

[providers.response_format]
models_path = "data"
model_id_field = "id"
model_name_field = "id"

[models]
default_model = "openai/gpt-5.1"

[[models.curated]]
name = "GPT-5.1"
provider = "openai"
model_id = "gpt-5.1-2025-11-13"
```

**Built-in Providers:**
1. ✅ OpenAI - Bearer auth
2. ✅ Anthropic - Header auth (`x-api-key`)
3. ✅ Google Gemini - Query param auth
4. ✅ OpenRouter - Bearer auth

**Model Discovery Flow:**
1. Settings → Models → "+ Add Model"
2. Modal shows provider list (OpenAI, Anthropic, Google, OpenRouter)
3. Click provider → Reads `{PROVIDER}_API_KEY` from env
4. Calls provider API: `GET {models_url}` with configured auth
5. Parses response using `response_format` config:
   ```rust
   let models = json.get(models_path).as_array();
   for model in models {
       let id = model.get(model_id_field).strip_prefix(strip_prefix);
       let name = model.get(model_name_field);
   }
   ```
6. Shows list with search/filter (50+ models for OpenAI)
7. Click "+" to add to curated list
8. Saves to `config/models.toml`

**Curated Models Management:**
- ✅ List with remove buttons: `GPT-4o  (openai/gpt-4o-2024-08-06)` [✖]
- ✅ Add/remove persisted immediately
- ✅ Prevents duplicates (by provider + model_id)

**models.dev Integration:**
- ✅ Fetch model metadata from models.dev on startup
- ✅ Find latest Haiku for OAuth default model
- ✅ Used to display model names in OAuth mode

### 2.7 Authentication ⭐⭐ (Core)

**API Key Sync** (`startup/auth.rs`):
- ✅ Load `.env` file from executable directory
- ✅ Extract all `{PROVIDER}_API_KEY` variables
- ✅ Send to server: `PUT /auth/{provider}` with `{ "type": "api", "key": "..." }`
- ✅ Display sync status in settings (✅ success, ❌ failure per provider)
- ✅ Skip Anthropic if OAuth tokens detected

**OAuth Mode Toggle** (Anthropic):
- ✅ Checkbox in footer: "☐" (API Key) / "☑" (Subscription)
- ✅ **Switch to Subscription**:
  1. Read OAuth tokens from `.env` cache
  2. Send `PUT /auth/anthropic` with `{ "type": "oauth", "access": "...", "refresh": "...", "expires": ... }`
  3. Send `POST /instance/dispose` to reload server
- ✅ **Switch to API Key**:
  1. Read `ANTHROPIC_API_KEY` from `.env`
  2. Send `PUT /auth/anthropic` with `{ "type": "api", "key": "..." }`
  3. Send `POST /instance/dispose`
- ✅ OAuth expiry countdown timer (⏱ 23h 59m remaining)
- ✅ Color-coded: 🟢 green (>5m), 🟡 yellow (0-5m), 🔴 red (expired)
- ✅ "🔄 Refresh" button to reload tokens from server

**Provider Status**:
- ✅ Fetch `GET /provider` → `{ "connected": ["anthropic", "openai"] }`
- ✅ Show 🟢 indicator for OAuth-connected providers

### 2.8 Message Rendering ⭐⭐⭐ (MVP)

**Message Bubbles**:
- ✅ **User messages**: Right-aligned, blue background, 75% max width
- ✅ **Assistant messages**: Left-aligned, gray background, 75% max width
- ✅ **System messages**: Left-aligned, purple background (audio status, errors)

**Markdown Rendering** (`egui_commonmark`):
- ✅ CommonMark-spec compliant
- ✅ Code blocks with syntax highlighting
- ✅ Lists, headers, emphasis, links
- ✅ Code fence normalization (ensure `\n` before ` ``` `)
- ✅ Cached rendering (`CommonMarkCache`)

**Reasoning Display**:
- ✅ Collapsible section (default open if no text yet)
- ✅ Dark gray background with rounded corners
- ✅ Header: "Reasoning"
- ✅ Auto-collapse when message finishes

**Token Counts**:
- ✅ Displayed below message text
- ✅ Format: `tokens: in 1234, out 567, reason 89`
- ✅ Small, weak (gray) text

**Copy Button**:
- ✅ Copy full message text to clipboard

**Emoji Support**:
- ✅ System messages use `egui_twemoji::EmojiLabel` for colored emoji
- ✅ Assistant messages use markdown renderer

### 2.9 Audio (STT) ⭐ (Advanced)

**Push-to-Talk Recording**:
- ✅ Configurable key (default: `AltRight`)
- ✅ State machine: Idle → Recording → Processing → Transcribed
- ✅ Audio capture via `cpal` (cross-platform audio)
- ✅ Whisper model loading (`ggml-base.en.bin`)
- ✅ Resampling to 16kHz mono (Whisper requirement)
- ✅ Transcription via `whisper-rs` (local inference)

**Audio Task Lifecycle**:
1. App startup → Load Whisper model (if configured)
2. User presses `AltRight` → Send `AudioCmd::StartRecording`
3. Audio task starts capture → Send `UiMsg::RecordingStarted`
4. UI shows "🎙 Recording..." system message
5. User releases `AltRight` → Send `AudioCmd::StopRecording`
6. Audio task stops capture, resamples, transcribes (blocking)
7. Send `UiMsg::Transcription(text)`
8. UI appends text to input box

**Configuration**:
- ✅ `whisper_model_path` in `config.json`
- ✅ Auto-detect model in `models/ggml-base.en.bin` relative to executable
- ✅ Push-to-talk key customizable

**Error Handling**:
- ✅ Model not found → Audio disabled silently
- ✅ Capture fails → Show error system message

### 2.10 Attachments ⭐ (Enhanced)

**Image Paste from Clipboard**:
- ✅ "📋 Paste Image" button
- ✅ Reads image from clipboard via `arboard` crate
- ✅ Encodes to PNG via `image` crate
- ✅ Adds to `pending_attachments` list
- ✅ Displays "📎 Image" preview with `✖` remove button

**Sending Attachments**:
- ✅ Convert to base64 data URI: `data:image/png;base64,{base64}`
- ✅ Send as message part:
  ```json
  {
    "type": "file",
    "mime": "image/png",
    "filename": null,
    "url": "data:image/png;base64,..."
  }
  ```

**Limitations**:
- ⚠️ No file picker (only clipboard paste)
- ⚠️ No preview rendering (just shows "📎 Image" text)

### 2.11 Settings & Configuration ⭐⭐ (Enhanced - Comprehensive)

**Settings Window** (⚙ button in footer):
- ✅ Modal dialog, 600px wide, scrollable
- ✅ Three collapsible sections: Server, UI, Models

**Server Preferences** (Collapsible):
1. ✅ **Base URL override** - Text input (empty = auto-discovery)
2. ✅ **Directory override** - Text input (`x-opencode-directory` header)
3. ✅ **Auto-start server** - Checkbox (default: true)
4. ✅ **Server status display**:
   - Connected: `http://localhost:4008` (PID 12345)
   - Owned: `true` (client spawned this server)
   - Directory header: `/path/to/project`
5. ✅ **Server actions**:
   - "Reconnect" - Retry discovery/spawn
   - "Start Server" - Force spawn new server
   - "Stop Server" - Kill owned server (disabled if not owned)
   - "Delete all other sessions" - Cleanup (keeps only current tab)
6. ✅ **Save Server Settings** - Persist to config.json

**UI Preferences** (Collapsible):
1. ✅ **Font size preset** - Radio buttons:
   - Small (-2pt)
   - Standard (base)
   - Large (+2pt)
2. ✅ **Base font points** - Slider: 10pt to 24pt
3. ✅ **Chat density** - Radio buttons:
   - Compact (4px spacing)
   - Normal (8px spacing)
   - Comfortable (12px spacing)
4. ✅ **Show subagents** - Checkbox (show/hide subagents in agent list)
5. ✅ **Live preview** - All changes apply immediately to UI

**Models Preferences** (Collapsible):
1. ✅ **Curated models list**:
   - Format: `GPT-4o  (openai/gpt-4o-2024-08-06)` [✖]
   - Click [✖] to remove from list
2. ✅ **"+ Add Model" button** - Opens model discovery dialog
3. ✅ **Default model selector** - Dropdown (used for new tabs)
4. ✅ **Auth sync status**:
   - ⏸ Not started
   - ⏳ Syncing keys to server...
   - ✅ Complete (Synced: openai, anthropic)
   - ❌ Failed: `provider: error message`

**Config Persistence:**

`config.json` (Application Preferences):
- **Path:** Platform-specific app data directory
  - Linux: `~/.config/opencode-egui/config.json`
  - macOS: `~/Library/Application Support/opencode-egui/config.json`
  - Windows: `%APPDATA%\opencode-egui\config.json`
- **Structure:**
  ```json
  {
    "server": {
      "last_base_url": "http://localhost:4008",
      "auto_start": true,
      "directory_override": null
    },
    "ui": {
      "font_size": "Standard",
      "base_font_points": 14.0,
      "chat_density": "Normal"
    },
    "audio": {
      "push_to_talk_key": "AltRight",
      "whisper_model_path": null
    }
  }
  ```

`models.toml` (Model Configuration):
- **Path:** `{executable_dir}/config/models.toml`
- **Why TOML?** Human-editable, supports arrays/nested structures
- **Structure:** See section 2.6 (Provider Configuration)

**Additional Footer Features:**
- ✅ **Working directory display**: `CWD | /Users/tony/projects/my-app`
  - Shows `directory_override` (global) or `tab.directory` (per-session)
- ✅ **Session version display**: `v1.2.3` (from server)
- ✅ **Server ownership indicator**: Gray out "Stop Server" if not owned

---

## Part 3: UI Layout & Navigation

### Layout Structure

```
┌──────────────────────────────────────────────────────────────┐
│ Top Bar: [Tab 1] [Tab 2] [+]                                 │
├────────┬─────────────────────────────────────────────────────┤
│ Agents │ Message Area                                        │
│ Pane   │ ┌─────────────────────────────────────────────┐     │
│ (Left) │ │ User: Hello                                │     │
│        │ │ Assistant: [tool block] [text]             │     │
│        │ │   ├─ ⏳ (bash) - ls -la [0.5s]           │     │
│        │ │   └─ [message text with markdown]          │     │
│        │ │      tokens: in 123, out 456               │     │
│        │ └─────────────────────────────────────────────┘     │
├────────┴─────────────────────────────────────────────────────┤
│ Input Area (Resizable)                                       │
│ ┌──────┬────────────────────────────────────────┬──────────┐ │
│ │Attach│ [User input text area...]             │  Send    │ │
│ │      │                                        │  ⌘+Enter │ │
│ └──────┴────────────────────────────────────────┴──────────┘ │
├──────────────────────────────────────────────────────────────┤
│ Footer: ☐ [Model ▾] agent: build | CWD | Server | ⚙ Settings│
└──────────────────────────────────────────────────────────────┘
```

### Keyboard Shortcuts

- ✅ **Cmd+Enter** (macOS) / **Ctrl+Enter** (Windows/Linux): Send message
- ✅ **AltRight** (configurable): Push-to-talk (hold to record, release to transcribe)
- ✅ **Enter** (while renaming tab): Confirm rename
- ✅ **Escape** (while renaming tab): Cancel rename
- ✅ **Tab** (while renaming tab): Confirm rename

### Context Menus

- ✅ **Tab right-click**: "Rename" option

---

## Part 4: Implementation Priorities for Blazor

### MVP Features (Must Have)
**Session 4.5: Server Discovery + Basic Chat**
1. ✅ Server discovery/spawn
2. ✅ Single tab session
3. ✅ Send text message (`POST /session/{id}/message`)
4. ✅ Receive message events (SSE `/global/event`)
5. ✅ Display messages (text only, no markdown)
6. ✅ Basic message bubbles (user/assistant roles)

**Estimated Complexity**: ~120K tokens (gRPC setup, event handling, state management)

---

### Core Features (Should Have)
**Session 5: Multi-Tab + Agent Selection**
1. ✅ Tab bar (create/close/switch)
2. ✅ Agents pane with list
3. ✅ Agent selection per tab
4. ✅ Agent sent with message

**Session 6: Model Selection + Auth**
1. ✅ Model selector (dropdown)
2. ✅ API key sync to server
3. ✅ Provider status display
4. ✅ OAuth toggle (Anthropic)

**Session 7: Tool Calls + Permissions**
1. ✅ Tool call rendering (collapsible blocks)
2. ✅ Tool status icons
3. ✅ Permission request dialogs
4. ✅ Permission approval/reject

**Estimated Complexity**: ~300K tokens total

---

### Enhanced Features (Nice to Have)
**Session 8: Markdown + Polish**
1. ✅ Markdown rendering (Markdig in Blazor)
2. ✅ Reasoning sections (collapsible)
3. ✅ Token counts
4. ✅ Copy message button
5. ✅ Settings panel
6. ✅ Config persistence

**Estimated Complexity**: ~80K tokens

---

### Session 8: Polish & UX Features
**Include for Feature Parity**:
1. ✅ Tab rename (right-click context menu, inline edit)
2. ✅ Clipboard image paste (📋 button, Tauri clipboard API)
3. ✅ OAuth countdown timer (⏱ in footer with color coding)
4. ✅ Working directory display in footer
5. ✅ Session version display

### Session 9+: Audio Features (Optional)
**Complex but achievable**:
1. ⏳ Push-to-talk audio transcription (Whisper integration)
   - Platform-specific audio capture via Tauri plugin
   - Whisper model integration (ggml-base.en.bin)
   - Configurable hotkey

**Goal:** Full feature parity with egui reference implementation.

---

## Part 5: Technical Considerations for Blazor

### 5.1 State Management Challenges

**egui Approach** (Immediate Mode):
- State lives in `OpenCodeApp` struct
- UI renders from state every frame
- No separate view models

**Blazor Approach** (Retained Mode):
- Need reactive state management
- Consider: **Fluxor** (Redux-like) or **MobX.Blazor**
- Tab state should be observable
- Message updates should trigger re-render

**Recommendation**:
```csharp
public class AppState
{
    public List<TabState> Tabs { get; set; }
    public int ActiveTabIndex { get; set; }
    public ServerInfo? Server { get; set; }
    public List<AgentInfo> Agents { get; set; }
    public List<PermissionInfo> PendingPermissions { get; set; }
}

// Use Fluxor for state updates
public record AddMessageAction(string TabId, DisplayMessage Message);
public record UpdateToolCallAction(string TabId, string MessageId, string ToolId, ToolCall Update);
```

### 5.2 gRPC vs HTTP/SSE

**egui uses HTTP + SSE**:
- Simple reqwest HTTP client
- SSE for real-time events
- Works over standard HTTP/1.1

**Blazor/Tauri can use gRPC**:
- ✅ Better performance (binary protocol)
- ✅ Streaming built-in (server-side streaming for events)
- ✅ Type-safe with protobuf
- ⚠️ More complex setup

**Recommendation**: Use gRPC as planned in SESSION_PLAN.md

### 5.3 Event Handling Pattern

**egui Pattern**:
```rust
// Background task sends events to UI thread
let (tx, rx) = mpsc::channel();
tokio::spawn(async move {
    while let Some(event) = sse.recv().await {
        tx.send(UiMsg::GlobalEvent(event));
    }
});

// UI thread drains channel every frame
fn update(&mut self) {
    while let Ok(msg) = self.ui_rx.try_recv() {
        match msg {
            UiMsg::GlobalEvent(payload) => handle_event(payload),
            // ...
        }
    }
}
```

**Blazor Pattern**:
```csharp
// Background service listens to gRPC stream
public class EventStreamService : BackgroundService
{
    protected override async Task ExecuteAsync(CancellationToken stoppingToken)
    {
        await foreach (var evt in grpcClient.SubscribeEvents(stoppingToken))
        {
            // Dispatch to Fluxor store
            _dispatcher.Dispatch(new EventReceivedAction(evt));
        }
    }
}
```

### 5.4 Tool Call Updates (Complex!)

**Challenge**: Tool calls are updated incrementally via SSE events.

**egui approach**:
```rust
// Find or create tool call by ID or call_id
let tool = msg.tool_calls.iter_mut().find(|t| 
    t.id == tool_id || t.call_id.as_deref() == Some(call_id)
);

// Update fields incrementally
tool.status = new_status;
tool.logs.push(new_log);
if let Some(output) = new_output {
    tool.output = Some(output);
}
```

**Blazor challenge**:
- Need efficient lookup by `id` or `call_id`
- Consider using `Dictionary<string, ToolCall>` for fast lookup
- Or maintain dual index: `byId` and `byCallId`

### 5.5 Markdown Rendering

**egui uses**: `egui_commonmark` (custom egui-native renderer)

**Blazor options**:
1. **Markdig** (C# markdown parser) + custom Blazor components
2. **BlazorMarkdown** (wrapper around Markdig)
3. **Markdown.Blazor** (another wrapper)

**Recommendation**: Use **Markdig** with custom rendering to Blazor components for syntax highlighting support.

### 5.6 Permission System Edge Cases

**Complex filtering logic** (egui approach):
```rust
// Auto-reject permission if:
// 1. Message ID is in cancelled_messages
// 2. Call ID is in cancelled_calls
// 3. Permission created before cancelled_after timestamp
// 4. Permission created before last_send_at timestamp
// 5. suppress_incoming flag is true

let is_cancelled = 
    tab.cancelled_messages.contains(&perm.message_id) ||
    tab.cancelled_calls.contains(&perm.call_id) ||
    perm.time.created <= tab.cancelled_after ||
    perm.time.created <= tab.last_send_at ||
    tab.suppress_incoming;

if is_cancelled {
    auto_reject(perm);
} else {
    show_permission_dialog(perm);
}
```

**Blazor should replicate this exact logic** to avoid permission dialog spam after user cancels.

---

## Part 6: Full Feature Parity Roadmap

### Session 8: Polish & UX Enhancements

**Tab Rename** ⭐
- **Complexity**: Low
- **Implementation**:
  - Right-click context menu on tab
  - Inline text edit with focus + select all
  - Enter/Tab to confirm, Escape to cancel
  - Keyboard shortcuts
- **Value**: High - users want to organize their sessions
- **Recommendation**: **Include in Session 8**

**Clipboard Image Paste** ⭐⭐
- **Complexity**: Medium
- **Implementation**:
  - "📋 Paste Image" button (already in egui)
  - Tauri clipboard plugin for cross-platform access
  - PNG encoding via Blazor libraries
  - Base64 data URI construction
  - Preview list with remove buttons
- **Value**: High - multimodal interactions are core to modern LLMs
- **Recommendation**: **Include in Session 8**

**OAuth Countdown Timer** ⭐
- **Complexity**: Low
- **Implementation**:
  - Footer display: `⏱ 23h 59m remaining`
  - Color coding: 🟢 green (>5m), 🟡 yellow (0-5m), 🔴 red (expired)
  - Update every second when OAuth mode enabled
  - "🔄 Refresh" button
- **Value**: Medium - helpful for OAuth users
- **Recommendation**: **Include in Session 8**

**Session 8 Token Estimate**: 80K → **100K** (+20K for full UX parity)

---

### Session 9+: Audio/STT (Advanced)

**Push-to-Talk Audio Transcription** ⭐⭐⭐
- **Complexity**: High (but achievable with Tauri)
- **Implementation**:
  1. Tauri audio plugin for cross-platform capture
  2. Whisper model integration:
     - Download `ggml-base.en.bin` (74MB) on first use
     - Load model in background thread
     - Inference via whisper-rs or whisper.cpp
  3. Configurable hotkey (default: AltRight)
  4. State machine: Idle → Recording → Processing → Transcribed
  5. Resampling to 16kHz mono (Whisper requirement)
- **Value**: Very High - hands-free input, accessibility
- **Recommendation**: **Include in Session 9** if time permits

**Why include audio:**
- Egui has it - we should have parity
- Accessibility feature (users with mobility issues)
- Productivity boost (faster than typing)
- Tauri makes it achievable (not as hard as I initially thought)

**Session 9 Token Estimate**: ~80K (audio integration)

---

## Revised Total Token Budget

| Session | Feature | Tokens |
|---------|---------|--------|
| 4.5 | Server + Basic Chat | 120K |
| 5 | Multi-Tab + Agents | 100K |
| 6 | Tool Calls + Permissions | 110K |
| 7 | Model Selection + Discovery + Auth | 120K |
| 8 | Markdown + **Full UX Parity** (rename, paste, timer) | **100K** |
| 9 | Audio/STT (optional) | 80K |
| **Total (MVP + Parity)** | | **550K** |
| **Total (Full Parity)** | | **630K** |

**Goal: Full feature parity with egui reference implementation.**

---

## Part 7: Key Takeaways

### What Makes This Client Complex?

1. **Real-time event streaming** with incremental updates
   - Text arrives character-by-character
   - Tool calls update status/logs/output incrementally
   - Reasoning sections grow over time

2. **Stateful cancellation logic**
   - Multiple cancel points (message, tool call, timestamp)
   - Auto-reject permissions for cancelled work
   - Suppress incoming events after cancel

3. **Multi-tab session isolation**
   - Each tab = separate server session
   - Events routed by session ID
   - Per-tab model/agent selection

4. **Rich tool visualization**
   - 11+ tool types supported
   - Collapsible/expandable UI
   - Permission dialogs inline in tool block
   - Smart command summaries

5. **Auth mode switching**
   - API key vs OAuth
   - Dynamic provider status
   - Token expiry countdown

### What to Prioritize for Blazor MVP?

**Focus on these workflows first**:
1. ✅ Launch app → Auto-discover/spawn server
2. ✅ Create tab → Create session → Send message → See response
3. ✅ See tool execution → Approve permission → See result
4. ✅ Switch agent → Send message → Verify agent behavior
5. ✅ Switch model → Send message → Verify model used

**Defer these until polish phase**:
- ❌ Audio/STT (too complex)
- ❌ Model discovery (can use hardcoded list)
- ❌ Clipboard paste (minor feature)
- ❌ OAuth countdown timer (visual polish)
- ❌ Tab rename (UX polish)

### Estimated Development Effort

| Phase | Features | Tokens | Effort |
|-------|----------|--------|--------|
| MVP (4.5) | Server + Basic Chat | 120K | 2-3 days |
| Core (5-7) | Tabs + Agents + Tools + Perms | 300K | 5-7 days |
| Polish (8) | Markdown + Settings + Config | 80K | 2-3 days |
| **Total** | **Production-Ready Client** | **500K** | **~10 days** |

---

## Appendix: Event Flow Examples

### Example 1: Send Message Flow

```
User: Types "Hello" and clicks Send
  ↓
Blazor: POST /session/{id}/message { parts: [{ type: "text", text: "Hello" }], agent: "build" }
  ↓
Server: Accepts message, starts processing
  ↓
SSE Event: { type: "message.updated", properties: { info: { id: "msg_123", role: "user" } } }
  ↓
Blazor: Add new DisplayMessage to tab.messages (role="user", text="Hello")
  ↓
SSE Event: { type: "message.updated", properties: { info: { id: "msg_456", role: "assistant" } } }
  ↓
Blazor: Add new DisplayMessage to tab.messages (role="assistant", text="", tool_calls=[])
  ↓
SSE Event: { type: "message.part.updated", properties: { part: { type: "text", text: "Hi!" } } }
  ↓
Blazor: Update msg_456.text_parts = ["Hi!"]
  ↓
SSE Event: { type: "message.updated", properties: { info: { finish: "stop", tokens: { input: 10, output: 2 } } } }
  ↓
Blazor: Update msg_456.tokens_input = 10, tokens_output = 2, active_assistant = None
```

---

### Example 2: Tool Call with Permission

```
User: "Read the file test.txt"
  ↓
Blazor: POST /session/{id}/message with text
  ↓
SSE: message.updated (user)
SSE: message.updated (assistant, msg_789)
SSE: message.part.updated (type=tool, id=tool_1, name=read, status=pending)
  ↓
Blazor: Add ToolCall { id: "tool_1", name: "read", status: "pending", input: { filePath: "test.txt" } }
  ↓
SSE: permission.updated { id: "perm_1", sessionID: "...", callID: "call_1", type: "read", pattern: ["test.txt"] }
  ↓
Blazor: Add to pending_permissions, show dialog in tool block
  ↓
User: Clicks "✅ Allow Once"
  ↓
Blazor: POST /session/{id}/permissions/perm_1 { response: "once" }
  ↓
SSE: permission.replied { permissionID: "perm_1", response: "once" }
  ↓
Blazor: Remove perm_1 from pending_permissions
  ↓
SSE: message.part.updated (type=tool, id=tool_1, status=running, started_at=...)
  ↓
Blazor: Update tool_1.status = "running", tool_1.started_at = ...
  ↓
SSE: message.part.updated (type=tool, id=tool_1, status=success, output="File contents...", finished_at=...)
  ↓
Blazor: Update tool_1.status = "success", tool_1.output = "...", tool_1.finished_at = ...
  ↓
SSE: message.part.updated (type=text, text="I read the file. Here's what it says...")
  ↓
Blazor: Update msg_789.text_parts = ["I read the file..."]
```

---

**End of Audit**
