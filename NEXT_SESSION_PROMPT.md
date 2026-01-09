# Next Session: Session 11 - Settings Panel (Models Section)

## Quick Context

**What We Completed (Sessions 5-10):**

- ✅ IPC WebSocket server with binary protobuf protocol
- ✅ Session handlers (list/create/delete) via OpenCode HTTP API
- ✅ C# IPC client with production-grade WebSocket management
- ✅ Home.razor displaying sessions using Radzen components
- ✅ Config management system with IPC handlers (get_config/update_config)
- ✅ Settings modal with Server section (discover/spawn/stop server controls)

**Current State:**

- App launches, loads config, connects to IPC server via WebSocket
- Blazor authenticates and can list/create/delete sessions
- Settings modal shows server status with Start/Stop/Refresh controls
- Config includes models.toml with 9 curated models across 4 providers
- **BUT:** No UI for viewing or selecting models - users can't see available models or set a default

---

## Your Mission: Session 11

Create a Models section in the Settings modal and a footer model selector:
- Display curated models from config in a DataGrid
- Default model dropdown in Settings
- Compact model selector in footer for quick access

**Full implementation plan:** See `Session_11_Plan.md`

---

## Key Architecture Points

### Config Structure (already exists in backend)

**ModelsConfig** (from `models.toml`):
```rust
pub struct ModelsConfig {
    pub providers: Vec<ProviderConfig>,  // 4 providers
    pub models: ModelsSection,
}

pub struct ModelsSection {
    pub default_model: String,           // e.g., "openai/gpt-5.1-2025-11-13"
    pub curated: Vec<CuratedModel>,      // 9 models
}

pub struct CuratedModel {
    pub name: String,      // "gpt-5.1-2025-11-13"
    pub provider: String,  // "openai"
    pub model_id: String,  // "gpt-5.1-2025-11-13"
}
```

### IPC Protocol (already exists)

```protobuf
// Request
message IpcGetConfigRequest {}

// Response
message IpcGetConfigResponse {
  string app_config_json = 1;     // JSON-serialized AppConfig
  string models_config_json = 2;  // JSON-serialized ModelsConfig
}
```

### Known Limitation

**Default model persistence not available in Session 11:**
- `default_model` is in `ModelsConfig`, not `AppConfig`
- Backend only has `UpdateAppConfig` handler, not `UpdateModelsConfig`
- Implement display-only for now, add persistence in future session

---

## Implementation Steps (Summary)

1. **Create ConfigModels.cs** - C# DTOs matching Rust structs
2. **Add to IIpcClient.cs** - `GetConfigAsync()` method signature
3. **Add ConfigUpdateException.cs** - New exception type
4. **Implement in IpcClient.cs** - `GetConfigAsync()` with JSON deserialization
5. **Create ModelsSection.razor** - Settings section with DataGrid and dropdown
6. **Update SettingsModal.razor** - Add `<ModelsSection />`
7. **Create ModelSelector.razor** - Compact footer dropdown
8. **Update MainLayout.razor** - Add footer with ModelSelector
9. **Add CSS** - Footer styling in MainLayout.razor.css

---

## Success Criteria

- [ ] `dotnet build` succeeds
- [ ] Settings modal shows Models section (below Server section)
- [ ] Models section has:
  - [ ] Default model dropdown
  - [ ] DataGrid with name, provider, model_id columns
  - [ ] Provider badges (color-coded by provider)
  - [ ] Refresh button
  - [ ] Loading/error states
- [ ] Footer appears at bottom with ModelSelector
- [ ] ModelSelector shows all curated models
- [ ] Selection works (local state only - no persistence)

---

## Key Files to Reference

**Existing patterns:**
- `frontend/desktop/opencode/Components/ServerSection.razor` - Component pattern
- `frontend/desktop/opencode/Services/IpcClient.cs` - Request/response pattern
- `backend/client-core/src/config/models.rs` - Rust struct definitions
- `apps/desktop/opencode/config/models.toml` - Actual model data

**Full plan:**
- `Session_11_Plan.md` - Complete implementation details with code snippets

---

## Important Reminders

1. **Follow ServerSection patterns** - async, disposal, error handling
2. **Use `PropertyNameCaseInsensitive = true`** when deserializing JSON
3. **Production quality** - null checks, cancellation tokens, structured logging
4. **Test incrementally** - build after each step
5. **Note the persistence limitation** - add TODO comment for future work

---

**Start with:** Step 1 - Create `ConfigModels.cs` with DTO classes
