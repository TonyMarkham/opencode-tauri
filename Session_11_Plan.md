# Session 11: Settings Panel - Models Section

## Overview

**Goal:** Add a Models section to the Settings modal that displays curated models, allows default model selection, and provides a footer model selector for per-chat selection.

**Estimated Token Budget:** ~180K tokens (within 200K limit)

---

## Current State Analysis

### What Exists

**Backend (Rust):**
- `ModelsConfig` struct with:
  - `providers: Vec<ProviderConfig>` - provider API configs (4 providers)
  - `models: ModelsSection` with `default_model` and `curated: Vec<CuratedModel>`
- `CuratedModel` struct: `name`, `provider`, `model_id`
- `ConfigState` actor with `get_models_config()` async method
- IPC handlers: `get_config` returns JSON-serialized `AppConfig` + `ModelsConfig`
- `models.toml` bundled with 9 curated models across 4 providers

**Proto Messages (ipc.proto):**
```protobuf
// Already exists:
message IpcGetConfigRequest {}
message IpcGetConfigResponse {
  string app_config_json = 1;     // JSON-serialized AppConfig
  string models_config_json = 2;  // JSON-serialized ModelsConfig
}
```

**C# Frontend:**
- `IpcClient.cs` with server management methods (Session 10)
- `ServerSection.razor` - pattern for async operations, error handling
- Settings modal already exists with Server section
- **NO** config methods in IIpcClient yet

### What's Missing for Session 11

1. **IIpcClient methods** for config operations (`GetConfigAsync`)
2. **C# DTOs** for AppConfig and ModelsConfig (deserialize JSON from proto)
3. **ModelsSection.razor** component in Settings modal
4. **ModelSelector.razor** component for footer
5. **Footer area** in MainLayout

---

## Architecture Decisions

### Decision 1: Config DTO Strategy

**Options:**
- A) Parse JSON manually with `JsonDocument`
- B) Create C# classes matching Rust structs
- C) Use `System.Text.Json` source generators

**Chosen: B (C# classes)** - Clear types, IntelliSense support, easy to maintain.

### Decision 2: Default Model Persistence

**Options:**
- A) Store in C# (Blazor preference storage)
- B) Store in backend AppConfig
- C) Store in ModelsConfig

**Chosen: B (AppConfig in backend)** - Already have update_config handler, consistent with egui.

**Note:** This requires adding `default_model` field to `AppConfig.server` section (currently only in ModelsConfig). For Session 11, we'll READ from ModelsConfig but acknowledge this as a limitation.

### Decision 3: Model Selector Location

**Options:**
- A) Footer bar (global, always visible)
- B) Chat header (per-chat, visible when chatting)
- C) Both

**Chosen: A (Footer bar)** - Matches egui, always accessible. Per-chat selection is future work (Session 26).

---

## Implementation Steps

### Step 1: Create Config DTOs

**File to create:** `frontend/desktop/opencode/Services/ConfigModels.cs`

These classes mirror the Rust structs and are used to deserialize the JSON from `IpcGetConfigResponse`.

```csharp
namespace OpenCode.Services;

using System.Text.Json.Serialization;

// ============================================
// APP CONFIG (mirrors backend/client-core/src/config/mod.rs)
// ============================================

public record AppConfig
{
    [JsonPropertyName("version")]
    public int Version { get; init; } = 1;

    [JsonPropertyName("server")]
    public ServerConfig Server { get; init; } = new();

    [JsonPropertyName("ui")]
    public UiPreferences Ui { get; init; } = new();

    [JsonPropertyName("audio")]
    public AudioConfig Audio { get; init; } = new();
}

public record ServerConfig
{
    [JsonPropertyName("last_opencode_url")]
    public string? LastOpencodeUrl { get; init; }

    [JsonPropertyName("auto_start")]
    public bool AutoStart { get; init; } = true;

    [JsonPropertyName("directory_override")]
    public string? DirectoryOverride { get; init; }
}

public record UiPreferences
{
    [JsonPropertyName("font_size")]
    public string FontSize { get; init; } = "Standard";

    [JsonPropertyName("base_font_points")]
    public float BaseFontPoints { get; init; } = 14.0f;

    [JsonPropertyName("chat_density")]
    public string ChatDensity { get; init; } = "Normal";
}

public record AudioConfig
{
    [JsonPropertyName("push_to_talk_key")]
    public string PushToTalkKey { get; init; } = "AltRight";

    [JsonPropertyName("whisper_model_path")]
    public string? WhisperModelPath { get; init; }
}

// ============================================
// MODELS CONFIG (mirrors backend/client-core/src/config/models.rs)
// ============================================

public record ModelsConfig
{
    [JsonPropertyName("providers")]
    public List<ProviderConfig> Providers { get; init; } = new();

    [JsonPropertyName("models")]
    public ModelsSection Models { get; init; } = new();
}

public record ModelsSection
{
    [JsonPropertyName("default_model")]
    public string DefaultModel { get; init; } = "openai/gpt-4";

    [JsonPropertyName("curated")]
    public List<CuratedModel> Curated { get; init; } = new();
}

public record CuratedModel
{
    [JsonPropertyName("name")]
    public string Name { get; init; } = "";

    [JsonPropertyName("provider")]
    public string Provider { get; init; } = "";

    [JsonPropertyName("model_id")]
    public string ModelId { get; init; } = "";

    /// <summary>
    /// Formatted ID for display and API calls (provider/model_id).
    /// </summary>
    public string FullId => $"{Provider}/{ModelId}";
}

public record ProviderConfig
{
    [JsonPropertyName("name")]
    public string Name { get; init; } = "";

    [JsonPropertyName("display_name")]
    public string DisplayName { get; init; } = "";

    [JsonPropertyName("api_key_env")]
    public string ApiKeyEnv { get; init; } = "";

    // Other fields omitted - not needed for Session 11
}
```

---

### Step 2: Add Config Methods to IIpcClient Interface

**File:** `frontend/desktop/opencode/Services/IIpcClient.cs`

**Add after server management methods:**

```csharp
// Config operations

/// <summary>
/// Gets the current application and models configuration.
/// </summary>
/// <param name="cancellationToken">Cancellation token.</param>
/// <returns>Tuple of (AppConfig, ModelsConfig).</returns>
/// <exception cref="Exceptions.IpcConnectionException">Not connected to IPC.</exception>
/// <exception cref="Exceptions.IpcTimeoutException">Request timed out.</exception>
Task<(AppConfig App, ModelsConfig Models)> GetConfigAsync(CancellationToken cancellationToken = default);

/// <summary>
/// Updates the application configuration.
/// </summary>
/// <param name="config">Updated app config.</param>
/// <param name="cancellationToken">Cancellation token.</param>
/// <returns>True if updated successfully.</returns>
/// <exception cref="Exceptions.IpcConnectionException">Not connected to IPC.</exception>
/// <exception cref="Exceptions.IpcTimeoutException">Request timed out.</exception>
/// <exception cref="Exceptions.ConfigUpdateException">Config validation failed.</exception>
Task<bool> UpdateConfigAsync(AppConfig config, CancellationToken cancellationToken = default);
```

---

### Step 3: Add Config Exception Type

**File to create:** `frontend/desktop/opencode/Services/Exceptions/ConfigUpdateException.cs`

```csharp
namespace OpenCode.Services.Exceptions;

/// <summary>
/// Thrown when config update fails.
/// </summary>
public class ConfigUpdateException : IpcException
{
    public ConfigUpdateException(string message) : base(message) { }
    public ConfigUpdateException(string message, Exception inner) : base(message, inner) { }
}
```

---

### Step 4: Implement Config Methods in IpcClient

**File:** `frontend/desktop/opencode/Services/IpcClient.cs`

**Add after server management methods:**

```csharp
// ========== Config Operations ==========

public async Task<(AppConfig App, ModelsConfig Models)> GetConfigAsync(CancellationToken cancellationToken = default)
{
    ThrowIfDisposed();

    _logger.LogDebug("Getting config...");

    try
    {
        var request = new IpcClientMessage
        {
            GetConfig = new IpcGetConfigRequest()
        };

        var response = await SendRequestAsync(request, cancellationToken: cancellationToken);

        // Validate response
        if (response.GetConfigResponse == null)
        {
            _logger.LogError("GetConfigResponse is null in response payload");
            throw new IpcException("Invalid response from server: GetConfigResponse is null");
        }

        // Deserialize JSON strings to typed objects
        var appConfig = JsonSerializer.Deserialize<AppConfig>(
            response.GetConfigResponse.AppConfigJson,
            new JsonSerializerOptions { PropertyNameCaseInsensitive = true })
            ?? throw new IpcException("Failed to deserialize AppConfig");

        var modelsConfig = JsonSerializer.Deserialize<ModelsConfig>(
            response.GetConfigResponse.ModelsConfigJson,
            new JsonSerializerOptions { PropertyNameCaseInsensitive = true })
            ?? throw new IpcException("Failed to deserialize ModelsConfig");

        _logger.LogInformation(
            "Config loaded: {ModelCount} curated models, default={DefaultModel}",
            modelsConfig.Models.Curated.Count,
            modelsConfig.Models.DefaultModel);

        return (appConfig, modelsConfig);
    }
    catch (IpcException)
    {
        throw;
    }
    catch (Exception ex)
    {
        _logger.LogError(ex, "Failed to get config");
        throw new IpcException("Config retrieval failed unexpectedly", ex);
    }
}

public async Task<bool> UpdateConfigAsync(AppConfig config, CancellationToken cancellationToken = default)
{
    ThrowIfDisposed();

    _logger.LogDebug("Updating config...");

    try
    {
        var configJson = JsonSerializer.Serialize(config);

        var request = new IpcClientMessage
        {
            UpdateConfig = new IpcUpdateConfigRequest { ConfigJson = configJson }
        };

        var response = await SendRequestAsync(request, cancellationToken: cancellationToken);

        // Validate response
        if (response.UpdateConfigResponse == null)
        {
            _logger.LogError("UpdateConfigResponse is null in response payload");
            throw new IpcException("Invalid response from server: UpdateConfigResponse is null");
        }

        if (!response.UpdateConfigResponse.Success)
        {
            var errorMsg = response.UpdateConfigResponse.Error ?? "Unknown error";
            _logger.LogError("Config update failed: {Error}", errorMsg);
            throw new ConfigUpdateException($"Config update failed: {errorMsg}");
        }

        _logger.LogInformation("Config updated successfully");
        return true;
    }
    catch (IpcException)
    {
        throw;
    }
    catch (Exception ex)
    {
        _logger.LogError(ex, "Failed to update config");
        throw new IpcException("Config update failed unexpectedly", ex);
    }
}
```

**Add using statement at top:**
```csharp
using System.Text.Json;
```

---

### Step 5: Create ModelsSection Component

**File to create:** `frontend/desktop/opencode/Components/ModelsSection.razor`

This displays the curated models list and default model dropdown.

**Key elements:**
- Default model dropdown at top
- DataGrid/list showing curated models (name, provider, model_id)
- Refresh button
- Loading and error states

**Pattern:** Follow ServerSection.razor for async operations, disposal, error handling.

```razor
@namespace OpenCode.Components
@inject IIpcClient IpcClient
@inject ILogger<ModelsSection> Logger
@using OpenCode.Services
@using OpenCode.Services.Exceptions
@implements IDisposable

<RadzenFieldset Text="Models" Style="margin-bottom: 1rem;" aria-label="Model Configuration">
    <RadzenStack Gap="1rem">
        
        @* Default Model Dropdown *@
        <RadzenRow AlignItems="AlignItems.Center">
            <RadzenColumn Size="3">
                <RadzenText TextStyle="TextStyle.Body2" Style="color: var(--rz-text-secondary-color);">
                    Default Model
                </RadzenText>
            </RadzenColumn>
            <RadzenColumn Size="9">
                <RadzenDropDown 
                    @bind-Value="_selectedDefaultModel"
                    Data="_curatedModels"
                    TextProperty="Name"
                    ValueProperty="FullId"
                    Placeholder="Select default model..."
                    Style="width: 100%;"
                    Disabled="_loading"
                    Change="@OnDefaultModelChanged"
                    aria-label="Select default model" />
            </RadzenColumn>
        </RadzenRow>
        
        @* Curated Models List *@
        <RadzenText TextStyle="TextStyle.Body2" Style="color: var(--rz-text-secondary-color); margin-top: 0.5rem;">
            Available Models (@(_curatedModels?.Count ?? 0))
        </RadzenText>
        
        <RadzenDataGrid 
            Data="_curatedModels" 
            TItem="CuratedModel"
            AllowSorting="true"
            Style="height: 200px;"
            Density="Density.Compact">
            <Columns>
                <RadzenDataGridColumn TItem="CuratedModel" Property="Name" Title="Name" Width="200px" />
                <RadzenDataGridColumn TItem="CuratedModel" Property="Provider" Title="Provider" Width="100px">
                    <Template Context="model">
                        <RadzenBadge BadgeStyle="@GetProviderBadgeStyle(model.Provider)" Text="@model.Provider" />
                    </Template>
                </RadzenDataGridColumn>
                <RadzenDataGridColumn TItem="CuratedModel" Property="ModelId" Title="Model ID" />
            </Columns>
        </RadzenDataGrid>
        
        @* Error display *@
        @if (_error != null)
        {
            <RadzenAlert 
                AlertStyle="AlertStyle.Danger" 
                Variant="Variant.Flat" 
                Shade="Shade.Lighter" 
                AllowClose="true" 
                Close="@DismissError"
                role="alert">
                @_error
            </RadzenAlert>
        }
        
        @* Action Buttons *@
        <RadzenStack Orientation="Orientation.Horizontal" Gap="0.5rem" Style="margin-top: 0.5rem;">
            <RadzenButton 
                Text="Refresh" 
                Icon="refresh" 
                ButtonStyle="ButtonStyle.Light"
                Click="RefreshAsync"
                Disabled="_loading"
                aria-label="Refresh models" />
        </RadzenStack>
        
        @if (_loading)
        {
            <RadzenProgressBar Mode="ProgressBarMode.Indeterminate" Style="height: 4px;" />
        }
        
    </RadzenStack>
</RadzenFieldset>

@code {
    private List<CuratedModel>? _curatedModels;
    private string? _selectedDefaultModel;
    private bool _loading;
    private string? _error;
    private CancellationTokenSource? _cts;
    private AppConfig? _appConfig;  // Store for updates

    protected override async Task OnInitializedAsync()
    {
        _cts = new CancellationTokenSource();
        await RefreshAsync();
    }

    private async Task RefreshAsync()
    {
        await CancelCurrentOperationAsync();

        _cts = new CancellationTokenSource();
        _loading = true;
        _error = null;

        try
        {
            if (!IpcClient.IsConnected)
            {
                await IpcClient.ConnectAsync();
            }

            var (appConfig, modelsConfig) = await IpcClient.GetConfigAsync(_cts.Token);
            
            _appConfig = appConfig;
            _curatedModels = modelsConfig.Models.Curated;
            _selectedDefaultModel = modelsConfig.Models.DefaultModel;

            Logger.LogDebug("Models loaded: {Count} curated, default={Default}", 
                _curatedModels.Count, _selectedDefaultModel);
        }
        catch (OperationCanceledException)
        {
            Logger.LogDebug("Models refresh cancelled");
        }
        catch (IpcConnectionException ex)
        {
            _error = "Not connected to IPC server.";
            Logger.LogError(ex, "IPC connection error loading models");
        }
        catch (IpcTimeoutException ex)
        {
            _error = "Request timed out.";
            Logger.LogError(ex, "Timeout loading models");
        }
        catch (Exception ex)
        {
            _error = "Failed to load models.";
            Logger.LogError(ex, "Error loading models");
        }
        finally
        {
            _loading = false;
        }
    }

    private async Task OnDefaultModelChanged()
    {
        if (_appConfig == null || string.IsNullOrEmpty(_selectedDefaultModel))
            return;

        // NOTE: Currently default_model is in ModelsConfig, not AppConfig.
        // For Session 11, we just display - no persistence.
        // Full persistence requires backend changes (add to AppConfig).
        
        Logger.LogInformation("Default model selected: {Model}", _selectedDefaultModel);
        
        // TODO Session 12+: Persist to backend
        // await IpcClient.UpdateConfigAsync(updatedConfig, _cts.Token);
    }

    private async Task CancelCurrentOperationAsync()
    {
        if (_cts != null && !_cts.IsCancellationRequested)
        {
            _cts.Cancel();
            _cts.Dispose();
            _cts = null;
            await Task.Delay(50);
        }
    }

    private void DismissError() => _error = null;

    private BadgeStyle GetProviderBadgeStyle(string provider) => provider.ToLower() switch
    {
        "openai" => BadgeStyle.Success,
        "anthropic" => BadgeStyle.Warning,
        "google" => BadgeStyle.Info,
        "openrouter" => BadgeStyle.Secondary,
        _ => BadgeStyle.Light
    };

    public void Dispose()
    {
        _cts?.Cancel();
        _cts?.Dispose();
    }
}
```

---

### Step 6: Add ModelsSection to SettingsModal

**File:** `frontend/desktop/opencode/Components/SettingsModal.razor`

**Modify to add ModelsSection after ServerSection:**

```razor
<ServerSection />
<ModelsSection />

@* Future sections: Audio, UI Preferences *@
```

---

### Step 7: Create ModelSelector Component (Footer)

**File to create:** `frontend/desktop/opencode/Components/ModelSelector.razor`

A compact dropdown for the footer.

```razor
@namespace OpenCode.Components
@inject IIpcClient IpcClient
@inject ILogger<ModelSelector> Logger
@using OpenCode.Services
@implements IDisposable

<div class="model-selector" aria-label="Model selection">
    <RadzenDropDown 
        @bind-Value="_selectedModel"
        Data="_models"
        TextProperty="Name"
        ValueProperty="FullId"
        Placeholder="Select model..."
        Style="width: 200px; height: 32px;"
        Disabled="_loading"
        Density="Density.Compact"
        aria-label="Select model for chat" />
    
    @if (_loading)
    {
        <RadzenIcon Icon="hourglass_empty" Style="margin-left: 4px; color: var(--rz-text-secondary-color);" />
    }
</div>

@code {
    private List<CuratedModel>? _models;
    private string? _selectedModel;
    private bool _loading = true;
    private CancellationTokenSource? _cts;

    /// <summary>
    /// Event raised when model selection changes.
    /// </summary>
    [Parameter]
    public EventCallback<string> OnModelSelected { get; set; }

    /// <summary>
    /// Gets the currently selected model ID.
    /// </summary>
    public string? SelectedModel => _selectedModel;

    protected override async Task OnInitializedAsync()
    {
        _cts = new CancellationTokenSource();
        await LoadModelsAsync();
    }

    private async Task LoadModelsAsync()
    {
        _loading = true;

        try
        {
            if (!IpcClient.IsConnected)
            {
                await IpcClient.ConnectAsync();
            }

            var (_, modelsConfig) = await IpcClient.GetConfigAsync(_cts?.Token ?? default);
            
            _models = modelsConfig.Models.Curated;
            _selectedModel = modelsConfig.Models.DefaultModel;

            Logger.LogDebug("ModelSelector loaded {Count} models", _models.Count);
        }
        catch (Exception ex)
        {
            Logger.LogError(ex, "Failed to load models for selector");
            _models = new List<CuratedModel>();
        }
        finally
        {
            _loading = false;
        }
    }

    /// <summary>
    /// Refresh models from backend.
    /// </summary>
    public async Task RefreshAsync()
    {
        await LoadModelsAsync();
        StateHasChanged();
    }

    public void Dispose()
    {
        _cts?.Cancel();
        _cts?.Dispose();
    }
}
```

---

### Step 8: Add Footer to MainLayout

**File:** `frontend/desktop/opencode/Layout/MainLayout.razor`

**Modify to add footer area with ModelSelector:**

```razor
@inherits LayoutComponentBase

<div class="page">
    <div class="sidebar">
        <NavMenu />
    </div>

    <main>
        <div class="top-row px-4">
            <RadzenButton 
                Icon="settings" 
                ButtonStyle="ButtonStyle.Base"
                Variant="Variant.Text"
                Size="ButtonSize.Medium"
                Click="@ShowSettings"
                Style="color: #666;"
                aria-label="Open settings"
                title="Settings" />
        </div>

        <article class="content px-4">
            @Body
        </article>
        
        <footer class="footer px-4">
            <ModelSelector @ref="_modelSelector" />
        </footer>
    </main>
</div>

<SettingsModal @ref="_settingsModal" />

@code {
    private SettingsModal? _settingsModal;
    private ModelSelector? _modelSelector;
    
    private void ShowSettings()
    {
        _settingsModal?.Show();
    }
}
```

---

### Step 9: Add Footer CSS

**File:** `frontend/desktop/opencode/Layout/MainLayout.razor.css`

**Add footer styles:**

```css
.footer {
    position: fixed;
    bottom: 0;
    left: var(--sidebar-width, 250px);
    right: 0;
    height: 48px;
    background: var(--rz-base-background-color, #f8f9fa);
    border-top: 1px solid var(--rz-border-color, #dee2e6);
    display: flex;
    align-items: center;
    z-index: 100;
}

.model-selector {
    display: flex;
    align-items: center;
}
```

**Adjust content padding to account for footer:**

```css
.content {
    padding-bottom: 60px; /* Space for fixed footer */
}
```

---

## Verification Steps

After implementation:

1. **Build verification:**
   ```bash
   cd frontend/desktop/opencode && dotnet build
   ```

2. **Manual testing checklist:**
   - [ ] Settings button opens modal
   - [ ] Models section visible below Server section
   - [ ] Models section shows curated models in grid
   - [ ] Default model dropdown shows current selection
   - [ ] Provider badges have correct colors
   - [ ] Refresh button reloads models
   - [ ] Loading indicator shows during fetch
   - [ ] Error messages display on failure
   - [ ] Footer appears at bottom of page
   - [ ] Footer model selector shows models
   - [ ] Can select different model in footer dropdown

---

## Risk Assessment

### Low Risk
- Creating DTOs (straightforward mapping)
- Adding IpcClient methods (follows existing patterns)
- Adding ModelsSection (follows ServerSection pattern)

### Medium Risk
- JSON deserialization (field name casing, null handling)
- Footer CSS layout (may need adjustments)
- Default model persistence (requires backend change - deferred)

### Mitigations
- Use `PropertyNameCaseInsensitive = true` for JSON parsing
- Test with actual models.toml data
- Defer persistence to future session, just display for now

---

## Dependencies

**NuGet packages (already installed):**
- Radzen.Blazor (provides RadzenDropDown, RadzenDataGrid, RadzenBadge)
- System.Text.Json (for JSON deserialization)
- Google.Protobuf (provides IpcGetConfigRequest/Response)

**No new dependencies required.**

---

## Files Summary

### Files to Create (4)
1. `frontend/desktop/opencode/Services/ConfigModels.cs` - DTO classes
2. `frontend/desktop/opencode/Services/Exceptions/ConfigUpdateException.cs` - Exception
3. `frontend/desktop/opencode/Components/ModelsSection.razor` - Settings section
4. `frontend/desktop/opencode/Components/ModelSelector.razor` - Footer selector

### Files to Modify (4)
1. `frontend/desktop/opencode/Services/IIpcClient.cs` - Add config methods
2. `frontend/desktop/opencode/Services/IpcClient.cs` - Implement config methods
3. `frontend/desktop/opencode/Components/SettingsModal.razor` - Add ModelsSection
4. `frontend/desktop/opencode/Layout/MainLayout.razor` - Add footer with ModelSelector

### Files to Create/Modify (CSS)
5. `frontend/desktop/opencode/Layout/MainLayout.razor.css` - Footer styles

---

## Success Criteria

- [ ] `dotnet build` succeeds
- [ ] Settings modal shows Models section below Server section
- [ ] Models section displays:
  - [ ] Default model dropdown with current selection
  - [ ] DataGrid of curated models (name, provider, model_id)
  - [ ] Provider badges with color coding
  - [ ] Refresh button
  - [ ] Loading/error states
- [ ] Footer visible at bottom of page
- [ ] Footer ModelSelector shows:
  - [ ] Dropdown with all curated models
  - [ ] Current default model pre-selected
  - [ ] Loading indicator during fetch
- [ ] Model selection works (local state, no persistence yet)

---

## Out of Scope (Future Sessions)

- **Default model persistence** - Requires adding `default_model` to AppConfig (Session 12)
- **Per-chat model selection** - Requires tab state isolation (Session 26)
- **Model discovery** - Requires provider API calls (Sessions 31-33)
- **Add/remove curated models** - Requires ModelsConfig update handler (Session 33)
- **Provider status indicators** - Requires auth sync (Session 28)

---

## Known Limitations

1. **Default model display-only:** The dropdown shows the selection but doesn't persist changes because `default_model` is currently in `ModelsConfig`, not `AppConfig`. The backend only has `UpdateAppConfig` handler, not `UpdateModelsConfig`.

2. **No model validation:** We don't verify if the selected model is actually available from the provider.

3. **Static curated list:** Can't add/remove models yet - requires backend changes.

**Recommendation for developer:** Implement display functionality first. Note the persistence limitation with a TODO comment. This maintains forward progress while documenting the gap.

---

**Start with:** Step 1 - Create ConfigModels.cs with the DTO classes, then Step 2 - Add interface methods. Build and verify after each step.
