# Next Session: Session 9 - Config Loading

## Quick Context

**What We Completed (Phase 1 - Sessions 5-8B):**

- ✅ IPC WebSocket server with binary protobuf protocol
- ✅ Auth handshake and token validation
- ✅ Session handlers (list/create/delete)
- ✅ C# IPC client with production-grade WebSocket management
- ✅ Home.razor displaying sessions via IPC
- ✅ Full pipeline: Blazor → WebSocket → client-core → OpenCode HTTP API

**Current State:**

- App launches and connects to IPC server
- Blazor authenticates and displays sessions
- **BUT:** No configuration loading yet (hardcoded values, no persistence)

---

## Your Mission: Session 9

Implement configuration loading so the app:
1. Loads user preferences from `config.json` on startup
2. Loads model/provider definitions from bundled `models.toml`
3. Exposes config to Blazor via IPC
4. Persists config changes to disk

### Architecture (Per ADR-0002)

```
Tauri main.rs
    ├── Get config_dir from app.path().app_config_dir()
    ├── Get resource_dir from app.path().resource_dir()
    └── Pass paths to client-core

client-core config module
    ├── AppConfig::load(config_dir) → config.json
    ├── ModelsConfig::load(resource_dir) → models.toml
    └── ConfigManager for state management

IPC Server
    └── Handle IpcGetConfig, IpcUpdateConfig messages

Blazor
    └── Fetch config at startup, update UI
```

---

## Step 1: Create config module in client-core

**Goal:** Define config structs with load/save functionality

**Files to create:**
- `backend/client-core/src/config/mod.rs`
- `backend/client-core/src/config/models.rs`
- `backend/client-core/src/config/error.rs`

**AppConfig structure:**
```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppConfig {
    #[serde(default)]
    pub server: ServerConfig,
    #[serde(default)]
    pub ui: UiPreferences,
    #[serde(default)]
    pub audio: AudioConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServerConfig {
    pub last_opencode_url: Option<String>,
    #[serde(default = "default_auto_start")]
    pub auto_start: bool,
    pub directory_override: Option<String>,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum FontSizePreset {
    Small,
    Standard,
    Large,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum ChatDensity {
    Compact,
    Normal,
    Comfortable,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UiPreferences {
    #[serde(default)]
    pub font_size: FontSizePreset,
    #[serde(default = "default_base_font_points")]
    pub base_font_points: f32,
    #[serde(default)]
    pub chat_density: ChatDensity,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AudioConfig {
    #[serde(default = "default_push_to_talk_key")]
    pub push_to_talk_key: String,
    pub whisper_model_path: Option<String>,
}
```

**Key methods:**
```rust
impl AppConfig {
    pub fn load(config_dir: &Path) -> Result<Self, ConfigError>;
    pub fn save(&self, config_dir: &Path) -> Result<(), ConfigError>;
}
```

**Reference:** See `submodules/opencode-egui/src/config/mod.rs` for patterns

---

## Step 2: Create ModelsConfig

**Goal:** Parse models.toml for provider and model definitions

**ModelsConfig structure:**
```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelsConfig {
    #[serde(default)]
    pub providers: Vec<ProviderConfig>,
    #[serde(default)]
    pub models: ModelsSection,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProviderConfig {
    pub name: String,
    pub display_name: String,
    pub api_key_env: String,
    pub models_url: String,
    pub auth_type: String,
    #[serde(default)]
    pub auth_header: Option<String>,
    #[serde(default)]
    pub auth_param: Option<String>,
    #[serde(default)]
    pub extra_headers: HashMap<String, String>,
    pub response_format: ResponseFormat,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CuratedModel {
    pub name: String,
    pub provider: String,
    pub model_id: String,
}
```

**Reference:** See `submodules/opencode-egui/src/config/models.rs`

---

## Step 3: Integrate with Tauri startup

**Goal:** Load config during app initialization

**Modify:** `apps/desktop/opencode/src/main.rs`

Add after IPC server startup:
```rust
// Get platform-specific paths
let config_dir = app.path().app_config_dir()?;
let resource_dir = app.path().resource_dir()?;

// Load configs
let app_config = client_core::config::AppConfig::load(&config_dir)
    .unwrap_or_else(|e| {
        warn!("Config load failed, using defaults: {e}");
        client_core::config::AppConfig::default()
    });

info!("Config loaded: auto_start={}, font_size={:?}", 
    app_config.server.auto_start, app_config.ui.font_size);

// Store in Tauri state
app.manage(ConfigState::new(config_dir, app_config));
```

---

## Step 4: Add IPC handlers for config

**Goal:** Allow Blazor to fetch and update config via WebSocket

**Add to `proto/ipc.proto`:**
```protobuf
// In IpcClientMessage oneof:
IpcGetConfigRequest get_config = 20;
IpcUpdateConfigRequest update_config = 21;

// In IpcServerMessage oneof:
IpcGetConfigResponse config = 20;
IpcUpdateConfigResponse update_config_response = 21;

message IpcGetConfigRequest {}

message IpcGetConfigResponse {
    string app_config_json = 1;  // JSON serialized AppConfig
    string models_config_json = 2;  // JSON serialized ModelsConfig
}

message IpcUpdateConfigRequest {
    string config_json = 1;  // Partial config update as JSON
}

message IpcUpdateConfigResponse {
    bool success = 1;
    string error = 2;
}
```

**Note:** Using JSON inside proto messages for flexibility with complex nested config.

---

## Step 5: Create C# config types

**Goal:** Define C# classes matching Rust config

**File:** `frontend/desktop/opencode/Services/Config/AppConfig.cs`

```csharp
public class AppConfig
{
    public ServerConfig Server { get; set; } = new();
    public UiPreferences Ui { get; set; } = new();
    public AudioConfig Audio { get; set; } = new();
}

public class ServerConfig
{
    public string? LastOpencodeUrl { get; set; }
    public bool AutoStart { get; set; } = true;
    public string? DirectoryOverride { get; set; }
}

public enum FontSizePreset { Small, Standard, Large }
public enum ChatDensity { Compact, Normal, Comfortable }

public class UiPreferences
{
    public FontSizePreset FontSize { get; set; } = FontSizePreset.Standard;
    public float BaseFontPoints { get; set; } = 14.0f;
    public ChatDensity ChatDensity { get; set; } = ChatDensity.Normal;
}
```

---

## Step 6: Add config to IIpcClient

**Goal:** Extend IPC client with config operations

**Add to `IIpcClient.cs`:**
```csharp
Task<(AppConfig App, ModelsConfig Models)> GetConfigAsync(CancellationToken ct = default);
Task UpdateConfigAsync(AppConfig config, CancellationToken ct = default);
```

---

## Step 7: Bundle models.toml

**Goal:** Include default models configuration with app

**Create:** `apps/desktop/opencode/config/models.toml`

Copy content from `submodules/opencode-egui/config/models.toml`

**Update:** `apps/desktop/opencode/tauri.conf.json`
```json
{
  "bundle": {
    "resources": ["config/models.toml"]
  }
}
```

---

## Success Criteria for Session 9

- [ ] `cargo build -p client-core` succeeds with config module
- [ ] `cargo test -p client-core` passes config unit tests
- [ ] App startup logs show config loaded
- [ ] `config.json` created in app data directory if missing
- [ ] Config persists across app restarts
- [ ] `models.toml` loads from bundle
- [ ] Blazor can fetch config via IPC (add test button if needed)

---

## Key Files to Reference

**Existing (Read these first):**
- `submodules/opencode-egui/src/config/mod.rs` - AppConfig patterns
- `submodules/opencode-egui/src/config/models.rs` - ModelsConfig patterns
- `submodules/opencode-egui/config/models.toml` - Default models
- `docs/EGUI_ARCHITECTURE.md` - Config file locations and structure
- `Session_9_Plan.md` - Full session plan with edge cases

**To Create/Modify:**
- `backend/client-core/src/config/mod.rs` - New
- `backend/client-core/src/config/models.rs` - New
- `backend/client-core/src/config/error.rs` - New
- `apps/desktop/opencode/src/main.rs` - Add config loading
- `apps/desktop/opencode/config/models.toml` - New (copy from egui)
- `proto/ipc.proto` - Add config messages
- `frontend/desktop/opencode/Services/` - Add config service

---

## Important Reminders

1. **Per ADR-0002**: Config logic lives in client-core, not Tauri
2. **Use naming conventions**: `opencode_url` not `base_url`
3. **Never crash on config errors**: Fall back to defaults with warning
4. **Use `#[serde(default)]`**: For forward compatibility
5. **Production-grade**: No unwraps, comprehensive error handling

---

## Dependencies to Add

**client-core Cargo.toml:**
```toml
toml = "0.8"
```

(serde and serde_json already present)

---

**Start with:** "Read Session_9_Plan.md and the egui config files, then create the config module structure in client-core"
