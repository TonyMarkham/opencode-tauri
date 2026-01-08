# Session 9: Config Loading

> **Read [CRITICAL_OPERATING_CONSTRAINTS.md](CRITICAL_OPERATING_CONSTRAINTS.md) before starting this session.**

**Status:** Ready to Start  
**Prerequisite:** Session 8 complete (Phase 1 Communication Foundation)  
**Estimated Tokens:** ~80K

---

## Goal

App loads configuration from disk on startup, persists changes, and exposes config to Blazor UI.

---

## Architecture Decision: Where Does Config Live?

Per ADR-0002 (Thin Tauri Layer Principle):

| Question | Answer | Layer |
|----------|--------|-------|
| Does config need OS-level access? | Yes (file paths) | Tauri provides paths |
| Can config logic be tested without Tauri? | Yes | client-core |
| Would a CLI tool want this? | Yes | client-core |

**Decision:** Config loading/saving logic lives in **client-core**. Tauri provides:
1. Platform-specific config directory path (via `app.path().app_config_dir()`)
2. Thin command to expose config to Blazor

---

## Reference: egui Implementation

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

**Location:** `{exe_dir}/config/models.toml` (bundled with app)

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

## Key Design Decisions

### 1. Config vs Models - Different Purposes

| File | Purpose | Location | Mutability |
|------|---------|----------|------------|
| `config.json` | User preferences | App data dir | Frequently changed |
| `models.toml` | Provider/model definitions | App bundle | Rarely changed (curated list) |

### 2. Startup Flow

```
Tauri main.rs
    ├── Get config_dir from app.path().app_config_dir()
    ├── Get resource_dir from app.path().resource_dir() (for models.toml)
    ├── Pass paths to client-core config loader
    └── Store loaded config in Tauri state

client-core config module
    ├── Load config.json (or create default if missing)
    ├── Load models.toml (bundled, read-only)
    └── Return AppConfig + ModelsConfig
```

### 3. Config Access Pattern

**Option A: Via IPC (recommended)**
- Add `IpcGetConfig` / `IpcSetConfig` messages
- Config changes flow through WebSocket
- Consistent with Phase 1 architecture

**Option B: Via Tauri Command**
- One-time config fetch at startup
- Config changes still via IPC
- Simpler for initial load

**Recommendation:** Option B for initial load, Option A for changes. This keeps the pattern from Session 8 (IPC config via Tauri command at startup).

### 4. Config State Management

```
Blazor UI ─────────────────────────────────────────────────
    │                                                      
    │ (1) OnInit: Get config via Tauri command            
    │ (4) Update UI                                        
    │                                                      
    ▼                                                      
Tauri ─────────────────────────────────────────────────────
    │ get_app_config command                               
    │                                                      
    ▼                                                      
client-core ───────────────────────────────────────────────
    │ ConfigManager                                        
    │   - load() from disk                                 
    │   - save() to disk                                   
    │   - get_app_config()                                 
    │   - get_models_config()                              
    │   - update_server_url(url)                           
    │   - update_ui_preferences(prefs)                     
```

---

## Deliverables

| # | Deliverable | Success Criteria |
|---|-------------|------------------|
| 1 | `client-core/src/config/mod.rs` | AppConfig struct with load/save |
| 2 | `client-core/src/config/models.rs` | ModelsConfig struct with load |
| 3 | Config loading in Tauri startup | Paths passed from Tauri to client-core |
| 4 | Tauri command `get_app_config` | Blazor can fetch config at startup |
| 5 | IPC messages for config updates | `IpcGetConfig`, `IpcUpdateConfig` |
| 6 | Config persistence | Changes saved to disk immediately |
| 7 | Tests | Unit tests for load/save/defaults |

---

## Implementation Steps

### Step 1: Create config module in client-core

**Files to create:**
- `backend/client-core/src/config/mod.rs`
- `backend/client-core/src/config/models.rs`
- `backend/client-core/src/config/error.rs`

**AppConfig structure:**
```rust
pub struct AppConfig {
    pub server: ServerConfig,
    pub ui: UiPreferences,
    pub audio: AudioConfig,
}

pub struct ServerConfig {
    pub last_opencode_url: Option<String>,  // Note: NOT "base_url" per naming convention
    pub auto_start: bool,
    pub directory_override: Option<String>,
}

pub struct UiPreferences {
    pub font_size: FontSizePreset,  // Small, Standard, Large
    pub base_font_points: f32,
    pub chat_density: ChatDensity,  // Compact, Normal, Comfortable
}

pub struct AudioConfig {
    pub push_to_talk_key: String,
    pub whisper_model_path: Option<String>,
}
```

**Key methods:**
```rust
impl AppConfig {
    pub fn load(config_dir: &Path) -> Result<Self, ConfigError>;
    pub fn save(&self, config_dir: &Path) -> Result<(), ConfigError>;
    pub fn default() -> Self;
}
```

### Step 2: Create ModelsConfig in client-core

**ModelsConfig structure:**
```rust
pub struct ModelsConfig {
    pub providers: Vec<ProviderConfig>,
    pub models: ModelsSection,
}

pub struct ProviderConfig {
    pub name: String,
    pub display_name: String,
    pub api_key_env: String,
    pub models_url: String,
    pub auth_type: AuthType,  // Bearer, Header, QueryParam
    pub auth_header: Option<String>,
    pub auth_param: Option<String>,
    pub extra_headers: HashMap<String, String>,
    pub response_format: ResponseFormat,
}

pub struct ModelsSection {
    pub default_model: String,  // "provider/model_id" format
    pub curated: Vec<CuratedModel>,
}

pub struct CuratedModel {
    pub name: String,
    pub provider: String,
    pub model_id: String,
}
```

**Key methods:**
```rust
impl ModelsConfig {
    pub fn load(resource_dir: &Path) -> Result<Self, ConfigError>;
    pub fn get_provider(&self, name: &str) -> Option<&ProviderConfig>;
    pub fn get_curated_models(&self) -> &[CuratedModel];
    pub fn add_curated_model(&mut self, model: CuratedModel);
    pub fn remove_curated_model(&mut self, provider: &str, model_id: &str);
    pub fn save(&self, resource_dir: &Path) -> Result<(), ConfigError>;
}
```

### Step 3: Add config error types

**File:** `backend/client-core/src/config/error.rs`

```rust
#[derive(Debug, Error)]
pub enum ConfigError {
    #[error("Failed to read config file: {path}")]
    ReadError { path: PathBuf, source: std::io::Error },
    
    #[error("Failed to parse config: {reason}")]
    ParseError { reason: String },
    
    #[error("Failed to write config file: {path}")]
    WriteError { path: PathBuf, source: std::io::Error },
    
    #[error("Config directory not found")]
    DirectoryNotFound,
}
```

### Step 4: Integrate with Tauri startup

**Modify:** `apps/desktop/opencode/src/main.rs`

```rust
.setup(|app| {
    // ... existing IPC server setup ...
    
    // Get platform-specific paths
    let config_dir = app.path().app_config_dir()
        .map_err(|e| format!("Failed to get config dir: {e}"))?;
    let resource_dir = app.path().resource_dir()
        .map_err(|e| format!("Failed to get resource dir: {e}"))?;
    
    // Load configs via client-core
    let app_config = client_core::config::AppConfig::load(&config_dir)
        .unwrap_or_else(|e| {
            warn!("Failed to load config, using defaults: {e}");
            client_core::config::AppConfig::default()
        });
    
    let models_config = client_core::config::ModelsConfig::load(&resource_dir)
        .unwrap_or_else(|e| {
            warn!("Failed to load models.toml, using defaults: {e}");
            client_core::config::ModelsConfig::default()
        });
    
    // Store in Tauri state
    app.manage(ConfigState::new(config_dir, app_config, models_config));
    
    Ok(())
})
```

### Step 5: Create Tauri command for config

**File:** `apps/desktop/opencode/src/tauri_commands/config.rs`

```rust
#[tauri::command]
pub fn get_app_config(state: State<'_, ConfigState>) -> AppConfigResponse {
    // Return serializable config
}

#[tauri::command]
pub async fn update_app_config(
    state: State<'_, ConfigState>,
    updates: AppConfigUpdate,
) -> Result<(), String> {
    // Update config and save to disk
}
```

### Step 6: Add IPC messages for config

**File:** `proto/ipc.proto`

Add messages:
```protobuf
message IpcGetConfigRequest {}

message IpcGetConfigResponse {
    AppConfig app_config = 1;
    ModelsConfig models_config = 2;
}

message IpcUpdateConfigRequest {
    oneof update {
        ServerConfigUpdate server = 1;
        UiPreferencesUpdate ui = 2;
        AudioConfigUpdate audio = 3;
    }
}

message IpcUpdateConfigResponse {
    bool success = 1;
    string error = 2;
}
```

### Step 7: Create C# types and service

**File:** `frontend/desktop/opencode/Services/ConfigService.cs`

```csharp
public interface IConfigService
{
    Task<AppConfig> GetAppConfigAsync();
    Task<ModelsConfig> GetModelsConfigAsync();
    Task UpdateServerConfigAsync(ServerConfigUpdate update);
    Task UpdateUiPreferencesAsync(UiPreferencesUpdate update);
}
```

### Step 8: Bundle models.toml with app

**File:** `apps/desktop/opencode/tauri.conf.json`

Add to bundle resources:
```json
{
  "bundle": {
    "resources": [
      "config/models.toml"
    ]
  }
}
```

**Create:** `apps/desktop/opencode/config/models.toml` (copy from egui reference)

---

## Testing Strategy

### Unit Tests (client-core)

```rust
#[test]
fn load_default_config_when_file_missing() {
    let temp_dir = tempdir().unwrap();
    let config = AppConfig::load(temp_dir.path()).unwrap();
    assert_eq!(config.ui.font_size, FontSizePreset::Standard);
}

#[test]
fn save_and_reload_config() {
    let temp_dir = tempdir().unwrap();
    let mut config = AppConfig::default();
    config.server.auto_start = false;
    config.save(temp_dir.path()).unwrap();
    
    let reloaded = AppConfig::load(temp_dir.path()).unwrap();
    assert_eq!(reloaded.server.auto_start, false);
}

#[test]
fn parse_models_toml() {
    let toml = r#"
        [[providers]]
        name = "openai"
        display_name = "OpenAI"
        ...
    "#;
    let config: ModelsConfig = toml::from_str(toml).unwrap();
    assert_eq!(config.providers[0].name, "openai");
}
```

### Integration Tests

```rust
#[tokio::test]
async fn config_via_ipc() {
    // Start IPC server
    // Connect client
    // Send IpcGetConfigRequest
    // Verify response contains config
}
```

---

## Files Summary

### client-core (New)

| File | Purpose |
|------|---------|
| `src/config/mod.rs` | AppConfig, ConfigState, load/save |
| `src/config/models.rs` | ModelsConfig, ProviderConfig, CuratedModel |
| `src/config/error.rs` | ConfigError enum |
| `src/tests/config.rs` | Unit tests |

### Tauri App (Modify)

| File | Change |
|------|--------|
| `src/main.rs` | Load config in setup, manage ConfigState |
| `src/lib.rs` | Add `pub mod config_state;` |
| `src/config_state.rs` | ConfigState wrapper for Tauri |
| `src/tauri_commands/mod.rs` | Add config module |
| `src/tauri_commands/config.rs` | get_app_config, update_app_config commands |
| `tauri.conf.json` | Bundle models.toml |
| `config/models.toml` | Default models configuration |

### Proto (Modify)

| File | Change |
|------|--------|
| `proto/ipc.proto` | Add config messages |
| `proto/oc_config.proto` | NEW: Config-related types |

### C# Frontend (Modify)

| File | Change |
|------|--------|
| `Services/ConfigService.cs` | NEW: IConfigService interface + impl |
| `Services/IIpcClient.cs` | Add GetConfigAsync method |
| `Services/IpcClient.cs` | Implement GetConfigAsync |
| `Program.cs` | Register IConfigService |

---

## Dependencies

### Rust Crates (client-core)

```toml
[dependencies]
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"
toml = "0.8"
directories = "5.0"  # For fallback paths if needed
```

### Proto Dependencies

Config types may need to be shared:
- `oc_config.proto` - AppConfig, ModelsConfig as proto messages
- Or use JSON serialization for simplicity (proto for IPC envelope, JSON for config payload)

**Recommendation:** Use JSON for config payload inside IPC messages. Config structure is complex with optional fields and maps - JSON handles this better than proto.

---

## Edge Cases to Handle

1. **Config file doesn't exist** → Create with defaults
2. **Config file corrupted/invalid JSON** → Log warning, use defaults
3. **Config directory doesn't exist** → Create it
4. **models.toml not bundled** → Log error, use empty providers list
5. **Partial config (missing fields)** → Serde defaults fill in missing fields
6. **Concurrent config updates** → Use RwLock in ConfigState
7. **Save fails (disk full, permissions)** → Return error, don't crash

---

## Verification

```bash
# Build client-core
cargo build -p client-core

# Run client-core tests
cargo test -p client-core config

# Build Tauri app
cargo build -p opencode

# Run Tauri app - verify config loaded in logs
cargo run -p opencode

# Check config file created
ls -la ~/Library/Application\ Support/com.opencode.blazor/
# Should see config.json

# C# build
dotnet build frontend/desktop/opencode/Opencode.csproj
```

---

## Success Criteria

- [ ] App starts with config loaded (visible in logs)
- [ ] Config file created if missing
- [ ] Config persists across restarts
- [ ] Models.toml loads from bundle
- [ ] Blazor can fetch config via IPC
- [ ] Config changes save immediately
- [ ] Unit tests pass
- [ ] No unwraps or panics in config code

---

## Out of Scope (Future Sessions)

- Settings UI panel (Session 10)
- Model selector dropdown (Session 11)
- API key sync to OpenCode server (Session 12)
- Model discovery from provider APIs (Session 31-33)

---

## Notes for Developer

1. **Follow egui patterns** - The config structures mirror `submodules/opencode-egui/src/config/`
2. **Use naming conventions** - `opencode_url` not `base_url`, per SESSION_PLAN naming rules
3. **Serde defaults** - Use `#[serde(default)]` liberally for forward compatibility
4. **Error handling** - Never crash on config errors, fall back to defaults with warning
5. **Logging** - Log config load/save at INFO level, errors at WARN/ERROR
