# EGUI Client Architecture Reference

**Source:** `submodules/opencode-egui/`  
**Purpose:** Document the egui client's architecture so future sessions don't have to rediscover it.

---

## Startup Flow

When the egui app launches, here's what happens **in order**:

### 1. Load Config (Synchronous, Immediate)
```
OpenCodeApp::new()
├── AppConfig::load()           → ~/.config/opencode-egui/config.json
├── ModelsConfig::load()        → {exe_dir}/config/models.toml
└── Apply UI preferences to egui context
```

**Files:**
- `src/config/mod.rs` - `AppConfig` (server, ui, audio settings)
- `src/config/models.rs` - `ModelsConfig` (providers, curated models, default model)

### 2. Check OAuth State (Synchronous, Immediate)
```
OpenCodeApp::new()
└── AnthropicAuth::read_from_server()  → reads server's auth.json
    ├── If OAuth found → cache to .env, set oauth_token, anthropic_subscription_mode = true
    └── If API key found → anthropic_subscription_mode = false
```

**File:** `src/auth/mod.rs`

### 3. Server Discovery (Async, First Frame)
```
start_server_discovery()
├── Create tokio runtime
├── Create UI message channel (tx/rx)
├── Spawn audio task (if Whisper model found)
├── Spawn models.dev fetch (for OAuth default model)
└── Spawn discover_or_spawn task:
    ├── discover() → check ports 4008-4018
    └── spawn_and_wait() → if not found
```

**Files:**
- `src/discovery/process.rs` - `discover()`, `check_health()`
- `src/discovery/spawn.rs` - `spawn_and_wait()`

### 4. On Server Connected (Async Callback)
```
UiMsg::ServerConnected(info)
├── Create OpencodeClient with base_url
├── Set directory header (from config or cwd)
├── Set OAuth token (if present)
├── Spawn: Fetch provider status (GET /provider)
├── Spawn: Subscribe to SSE (GET /global/event)
├── Spawn: Fetch agents (GET /agent)
├── Spawn: Sync API keys to server (PUT /auth/{provider})
└── Save base_url to config
```

**Key insight:** Auth sync happens AFTER server connects, not before.

### 5. Create First Tab (User Action or Auto)
```
User clicks "+" or app auto-creates tab
├── Create empty Tab struct
├── Tab has NO session_id yet
├── Spawn: POST /session to create server session
└── UiMsg::SessionCreated updates tab with session_id
```

**Key insight:** Tab exists before session. User can't send messages until session_id is set.

---

## State Structure

### AppState (Global)
```rust
struct OpenCodeApp {
    // Tabs
    tabs: Vec<Tab>,
    active: usize,
    
    // Server
    server: Option<ServerInfo>,      // Connected server info
    client: Option<OpencodeClient>,  // HTTP client
    
    // Auth
    oauth_token: Option<String>,
    auth_sync_state: AuthSyncState,
    connected_providers: Vec<String>,
    anthropic_subscription_mode: bool,
    anthropic_oauth_expires: Option<u64>,
    
    // Agents
    agents: Vec<AgentInfo>,
    default_agent: String,
    show_subagents: bool,
    
    // Models
    models_config: ModelsConfig,     // From models.toml
    
    // Config
    config: AppConfig,               // From config.json
    
    // Permissions
    pending_permissions: Vec<PermissionInfo>,
}
```

### TabState (Per-Tab)
```rust
struct Tab {
    title: String,
    session_id: Option<String>,      // None until server creates session
    directory: Option<String>,
    messages: Vec<DisplayMessage>,
    active_assistant: Option<String>, // ID of streaming message
    input: String,
    selected_model: Option<(String, String)>,  // (provider, model_id)
    selected_agent: Option<String>,
    
    // Cancellation state
    cancelled_messages: Vec<String>,
    cancelled_calls: Vec<String>,
    cancelled_after: Option<i64>,
    suppress_incoming: bool,
    last_send_at: i64,
}
```

---

## Config Files

### config.json (User Preferences)
**Location:** Platform-specific app data directory
- Linux: `~/.config/opencode-egui/config.json`
- macOS: `~/Library/Application Support/opencode-egui/config.json`
- Windows: `%APPDATA%\opencode-egui\config.json`

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

### models.toml (Model Configuration)
**Location:** `{exe_dir}/config/models.toml`

```toml
[[providers]]
name = "openai"
display_name = "OpenAI"
api_key_env = "OPENAI_API_KEY"
models_url = "https://api.openai.com/v1/models"
auth_type = "bearer"

[providers.response_format]
models_path = "data"
model_id_field = "id"
model_name_field = "id"

[models]
default_model = "openai/gpt-4"

[[models.curated]]
name = "GPT-4"
provider = "openai"
model_id = "gpt-4"
```

---

## API Endpoints Used

### Session Management
- `GET /session` - List all sessions
- `POST /session` - Create new session
- `DELETE /session/{id}` - Delete session

### Messaging
- `POST /session/{id}/message` - Send message (with parts, model, agent)
- `POST /session/{id}/abort` - Cancel active response
- `POST /session/{id}/permissions/{id}` - Respond to permission

### Server Info
- `GET /doc` - Health check
- `GET /agent` - List agents
- `GET /provider` - Provider connection status

### Auth
- `PUT /auth/{provider}` - Sync API key or OAuth tokens
- `POST /instance/dispose` - Reload server (after auth change)

### Events
- `GET /global/event` - SSE stream for real-time updates

---

## Message Flow

### Send Message
```
User types message, clicks Send
├── Record last_send_at = now()
├── Clear suppress_incoming
├── POST /session/{id}/message with:
│   ├── parts: [{ type: "text", text: "..." }]
│   ├── model: { providerID: "...", modelID: "..." }  (optional)
│   └── agent: "build"  (optional)
└── Wait for SSE events
```

### Receive Events
```
SSE: message.updated (role=user)
└── Create DisplayMessage, add to tab.messages

SSE: message.updated (role=assistant)
└── Create DisplayMessage, set tab.active_assistant = id

SSE: message.part.updated (type=text)
└── Update message.text_parts

SSE: message.part.updated (type=reasoning)
└── Update message.reasoning_parts

SSE: message.part.updated (type=tool)
└── Create/update ToolCall in message.tool_calls

SSE: message.updated (finish != null)
└── Set tab.active_assistant = None, update token counts

SSE: permission.updated
└── Add to pending_permissions (or auto-reject if cancelled)

SSE: permission.replied
└── Remove from pending_permissions
```

---

## Cancellation Logic

When user clicks "Stop":
```rust
fn cancel_active_response(tab: &mut Tab) {
    let now = timestamp_ms();
    
    // Mark message as cancelled
    tab.cancelled_messages.push(active_id);
    
    // Mark all tool calls as cancelled
    for tool in &mut message.tool_calls {
        if tool.status not in ["success", "error", "completed", "cancelled"] {
            tool.status = "cancelled";
            tab.cancelled_calls.push(tool.call_id);
        }
    }
    
    // Set cutoff timestamp
    tab.cancelled_after = Some(now);
    tab.suppress_incoming = true;
    tab.active_assistant = None;
}
```

Event filtering (5 conditions):
```rust
let is_cancelled = 
    tab.cancelled_messages.contains(&event.message_id) ||
    tab.cancelled_calls.contains(&event.call_id) ||
    event.created <= tab.cancelled_after ||
    event.created <= tab.last_send_at ||
    tab.suppress_incoming;
```

---

## Key Dependencies for Chat

To send a message, you need:
1. **Server connected** - `server: Some(ServerInfo)`
2. **Session created** - `tab.session_id: Some(String)`
3. **Model selected** - `tab.selected_model` OR `models_config.default_model`
4. **Auth configured** - API keys synced OR OAuth token set

**Order matters:**
1. Config loads (has default model)
2. Server connects
3. Auth syncs (API keys to server)
4. Session creates
5. NOW you can send messages

---

## File Reference

| File | Purpose |
|------|---------|
| `src/main.rs` | Entry point, creates `OpenCodeApp` |
| `src/app.rs` | Main app struct, UI rendering, event handling |
| `src/config/mod.rs` | `AppConfig` - user preferences |
| `src/config/models.rs` | `ModelsConfig` - provider and model config |
| `src/auth/mod.rs` | OAuth token handling |
| `src/startup/auth.rs` | API key sync to server |
| `src/client/api.rs` | `OpencodeClient` - HTTP API calls |
| `src/client/events.rs` | SSE subscription |
| `src/client/providers.rs` | Model discovery from provider APIs |
| `src/discovery/process.rs` | Server discovery |
| `src/discovery/spawn.rs` | Server spawning |
| `src/types/agent.rs` | `AgentInfo` struct |
| `src/types/models.rs` | `MessagePart`, `ModelIdentifier` |
| `src/audio/` | Push-to-talk, Whisper integration |

---

## Implications for Blazor Implementation

1. **Config must load first** - Before anything else, load `config.json` and `models.toml`
2. **Model selection is required** - Can't send messages without a model
3. **Auth sync happens on server connect** - Not on app startup
4. **Session creation is async** - Tab exists before session_id is set
5. **Event filtering is complex** - Must replicate 5-condition cancellation logic
6. **OAuth state persists** - Read from server's auth.json, cache to .env
