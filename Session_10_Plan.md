# Session 10: Settings Panel - Server Section

## Overview

**Goal:** Create a Settings modal dialog with a Server section that displays OpenCode server status and provides connection controls.

**Estimated Token Budget:** ~150K tokens (well within 200K limit)

---

## Current State Analysis

### What Exists

**C# Frontend:**
- `IpcClient.cs` - WebSocket client with session operations (537 lines)
- `IIpcClient.cs` - Interface with `ListSessions`, `CreateSession`, `DeleteSession`, `HealthCheckAsync`
- `Home.razor` - Session list page using Radzen components (good pattern reference)
- `MainLayout.razor` - Simple layout with sidebar and content area
- `Program.cs` - DI setup with singleton IpcClient

**Proto Messages Available (ipc.proto):**
- `IpcDiscoverServerRequest/Response` - Discover running servers
- `IpcSpawnServerRequest/Response` - Start new server
- `IpcCheckHealthRequest/Response` - Health check
- `IpcStopServerRequest/Response` - Stop server (if owned)
- `IpcServerInfo` - Server info (pid, port, base_url, owned)
- `IpcGetConfigRequest/Response` - Get config (added in Session 9)

### What's Missing for Session 10

1. **IpcClient methods** for server management operations
2. **Settings modal** UI component
3. **Server status state** management
4. **Navigation** to settings (button in layout)

---

## Architecture Decisions

### Decision 1: Modal vs. Page

**Options:**
- A) Dedicated `/settings` page route
- B) Modal overlay on top of any page

**Chosen: B (Modal)** - Matches egui behavior, doesn't disrupt navigation state, can be opened from anywhere.

### Decision 2: State Management

**Options:**
- A) Fetch server status on every modal open
- B) Maintain global server status service with periodic refresh
- C) Fetch once on modal open, refresh via button

**Chosen: C (Fetch on open + manual refresh)** - Simple, predictable, avoids unnecessary polling.

### Decision 3: Component Structure

```
MainLayout.razor          ← Add settings button + modal host
  └── SettingsModal.razor ← Modal shell with tabs/sections
       └── ServerSection.razor ← Server status + controls
```

---

## Implementation Steps

### Step 1: Add Server Management Methods to IIpcClient

**File:** `frontend/desktop/opencode/Services/IIpcClient.cs`

**Add methods:**
```csharp
// Server management operations

/// <summary>
/// Discovers running OpenCode servers.
/// </summary>
/// <returns>Server info if found, null if no server running.</returns>
Task<IpcServerInfo?> DiscoverServerAsync(CancellationToken cancellationToken = default);

/// <summary>
/// Spawns a new OpenCode server.
/// </summary>
/// <param name="port">Preferred port (optional).</param>
/// <returns>Spawned server info.</returns>
Task<IpcServerInfo> SpawnServerAsync(uint? port = null, CancellationToken cancellationToken = default);

/// <summary>
/// Stops the OpenCode server (only works if we spawned it).
/// </summary>
/// <returns>True if stopped successfully.</returns>
Task<bool> StopServerAsync(CancellationToken cancellationToken = default);
```

**Note:** `HealthCheckAsync` already exists but checks IPC connection, not OpenCode server. Clarify naming:
- `HealthCheckAsync()` → Checks IPC WebSocket health
- `CheckServerHealthAsync()` → Checks OpenCode server health via IPC

---

### Step 2: Implement Server Methods in IpcClient

**File:** `frontend/desktop/opencode/Services/IpcClient.cs`

**Add after existing `DeleteSessionAsync` method:**

```csharp
public async Task<IpcServerInfo?> DiscoverServerAsync(CancellationToken cancellationToken = default)
{
    var request = new IpcClientMessage
    {
        DiscoverServer = new IpcDiscoverServerRequest()
    };

    var response = await SendRequestAsync(request, cancellationToken: cancellationToken);
    return response.DiscoverServerResponse.Server;  // null if not found
}

public async Task<IpcServerInfo> SpawnServerAsync(uint? port = null, CancellationToken cancellationToken = default)
{
    var request = new IpcClientMessage
    {
        SpawnServer = new IpcSpawnServerRequest { Port = port ?? 0 }
    };

    var response = await SendRequestAsync(request, cancellationToken: cancellationToken);
    return response.SpawnServerResponse.Server;
}

public async Task<bool> StopServerAsync(CancellationToken cancellationToken = default)
{
    var request = new IpcClientMessage
    {
        StopServer = new IpcStopServerRequest()
    };

    var response = await SendRequestAsync(request, cancellationToken: cancellationToken);
    return response.StopServerResponse.Success;
}

public async Task<bool> CheckServerHealthAsync(CancellationToken cancellationToken = default)
{
    var request = new IpcClientMessage
    {
        CheckHealth = new IpcCheckHealthRequest()
    };

    var response = await SendRequestAsync(request, cancellationToken: cancellationToken);
    return response.CheckHealthResponse.Healthy;
}
```

---

### Step 3: Create SettingsModal Component

**File to create:** `frontend/desktop/opencode/Components/SettingsModal.razor`

**Create directory first:** `frontend/desktop/opencode/Components/`

```razor
@using OpenCode.Services

<RadzenDialog 
    @bind-Visible="_visible"
    Style="width: 600px; max-width: 90vw;"
    ShowClose="true"
    CloseOnEsc="true"
    CloseOnOverlayClick="false">
    
    <ChildContent>
        <RadzenStack Gap="1.5rem" Style="padding: 1rem;">
            <RadzenText TextStyle="TextStyle.H5" Style="margin: 0;">Settings</RadzenText>
            
            <ServerSection />
            
            @* Future: ModelsSection, AudioSection, UiSection *@
        </RadzenStack>
    </ChildContent>
</RadzenDialog>

@code {
    private bool _visible;
    
    public void Show() 
    {
        _visible = true;
        StateHasChanged();
    }
    
    public void Hide()
    {
        _visible = false;
        StateHasChanged();
    }
    
    public bool IsVisible => _visible;
}
```

---

### Step 4: Create ServerSection Component

**File to create:** `frontend/desktop/opencode/Components/ServerSection.razor`

```razor
@inject IIpcClient IpcClient
@inject ILogger<ServerSection> Logger
@using OpenCode.Services
@using OpenCode.Services.Exceptions
@using Opencode

<RadzenFieldset Text="Server" Style="margin-bottom: 1rem;">
    <RadzenStack Gap="1rem">
        
        @* Status Row *@
        <RadzenRow AlignItems="AlignItems.Center">
            <RadzenColumn Size="3">
                <RadzenText TextStyle="TextStyle.Body2" Style="color: var(--rz-text-secondary-color);">
                    Status
                </RadzenText>
            </RadzenColumn>
            <RadzenColumn Size="9">
                <RadzenStack Orientation="Orientation.Horizontal" AlignItems="AlignItems.Center" Gap="0.5rem">
                    <RadzenIcon Icon="@GetStatusIcon()" Style="@GetStatusColor()" />
                    <RadzenText TextStyle="TextStyle.Body1">@GetStatusText()</RadzenText>
                </RadzenStack>
            </RadzenColumn>
        </RadzenRow>
        
        @if (_serverInfo != null)
        {
            @* URL Row *@
            <RadzenRow AlignItems="AlignItems.Center">
                <RadzenColumn Size="3">
                    <RadzenText TextStyle="TextStyle.Body2" Style="color: var(--rz-text-secondary-color);">
                        URL
                    </RadzenText>
                </RadzenColumn>
                <RadzenColumn Size="9">
                    <RadzenText TextStyle="TextStyle.Body1" Style="font-family: monospace;">
                        @_serverInfo.BaseUrl
                    </RadzenText>
                </RadzenColumn>
            </RadzenRow>
            
            @* PID Row *@
            <RadzenRow AlignItems="AlignItems.Center">
                <RadzenColumn Size="3">
                    <RadzenText TextStyle="TextStyle.Body2" Style="color: var(--rz-text-secondary-color);">
                        PID
                    </RadzenText>
                </RadzenColumn>
                <RadzenColumn Size="9">
                    <RadzenText TextStyle="TextStyle.Body1" Style="font-family: monospace;">
                        @_serverInfo.Pid
                    </RadzenText>
                </RadzenColumn>
            </RadzenRow>
            
            @* Owned Row *@
            <RadzenRow AlignItems="AlignItems.Center">
                <RadzenColumn Size="3">
                    <RadzenText TextStyle="TextStyle.Body2" Style="color: var(--rz-text-secondary-color);">
                        Owned
                    </RadzenText>
                </RadzenColumn>
                <RadzenColumn Size="9">
                    <RadzenText TextStyle="TextStyle.Body1">
                        @(_serverInfo.Owned ? "Yes (spawned by this app)" : "No (external)")
                    </RadzenText>
                </RadzenColumn>
            </RadzenRow>
        }
        
        @* Error display *@
        @if (_error != null)
        {
            <RadzenAlert AlertStyle="AlertStyle.Danger" Variant="Variant.Flat" Shade="Shade.Lighter" AllowClose="true" Close="@(() => _error = null)">
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
                Disabled="_loading" />
            
            <RadzenButton 
                Text="Start Server" 
                Icon="play_arrow" 
                ButtonStyle="ButtonStyle.Success"
                Click="StartServerAsync"
                Disabled="_loading || _serverInfo != null" />
            
            <RadzenButton 
                Text="Stop Server" 
                Icon="stop" 
                ButtonStyle="ButtonStyle.Danger"
                Click="StopServerAsync"
                Disabled="_loading || _serverInfo == null || !_serverInfo.Owned"
                title="@(_serverInfo?.Owned == false ? "Cannot stop external server" : "")" />
        </RadzenStack>
        
        @if (_loading)
        {
            <RadzenProgressBar Mode="ProgressBarMode.Indeterminate" Style="height: 4px;" />
        }
        
    </RadzenStack>
</RadzenFieldset>

@code {
    private IpcServerInfo? _serverInfo;
    private bool _loading;
    private string? _error;
    private bool _healthy;

    protected override async Task OnInitializedAsync()
    {
        await RefreshAsync();
    }
    
    private async Task RefreshAsync()
    {
        _loading = true;
        _error = null;
        StateHasChanged();
        
        try
        {
            // Ensure connected
            if (!IpcClient.IsConnected)
            {
                await IpcClient.ConnectAsync();
            }
            
            // Discover server
            _serverInfo = await IpcClient.DiscoverServerAsync();
            
            // Check health if server found
            if (_serverInfo != null)
            {
                _healthy = await IpcClient.CheckServerHealthAsync();
            }
            else
            {
                _healthy = false;
            }
            
            Logger.LogInformation("Server status refreshed: {Status}", 
                _serverInfo != null ? $"Connected to {_serverInfo.BaseUrl}" : "Not connected");
        }
        catch (IpcException ex)
        {
            _error = ex.Message;
            Logger.LogError(ex, "Failed to refresh server status");
        }
        catch (Exception ex)
        {
            _error = "An unexpected error occurred";
            Logger.LogError(ex, "Unexpected error refreshing server status");
        }
        finally
        {
            _loading = false;
            StateHasChanged();
        }
    }
    
    private async Task StartServerAsync()
    {
        _loading = true;
        _error = null;
        StateHasChanged();
        
        try
        {
            if (!IpcClient.IsConnected)
            {
                await IpcClient.ConnectAsync();
            }
            
            _serverInfo = await IpcClient.SpawnServerAsync();
            _healthy = true;
            
            Logger.LogInformation("Server started: PID={Pid}, Port={Port}", 
                _serverInfo.Pid, _serverInfo.Port);
        }
        catch (IpcException ex)
        {
            _error = $"Failed to start server: {ex.Message}";
            Logger.LogError(ex, "Failed to start server");
        }
        catch (Exception ex)
        {
            _error = "An unexpected error occurred while starting server";
            Logger.LogError(ex, "Unexpected error starting server");
        }
        finally
        {
            _loading = false;
            StateHasChanged();
        }
    }
    
    private async Task StopServerAsync()
    {
        _loading = true;
        _error = null;
        StateHasChanged();
        
        try
        {
            if (!IpcClient.IsConnected)
            {
                await IpcClient.ConnectAsync();
            }
            
            var success = await IpcClient.StopServerAsync();
            
            if (success)
            {
                _serverInfo = null;
                _healthy = false;
                Logger.LogInformation("Server stopped successfully");
            }
            else
            {
                _error = "Failed to stop server (may not be owned by this app)";
            }
        }
        catch (IpcException ex)
        {
            _error = $"Failed to stop server: {ex.Message}";
            Logger.LogError(ex, "Failed to stop server");
        }
        catch (Exception ex)
        {
            _error = "An unexpected error occurred while stopping server";
            Logger.LogError(ex, "Unexpected error stopping server");
        }
        finally
        {
            _loading = false;
            StateHasChanged();
        }
    }
    
    private string GetStatusIcon() => _serverInfo != null 
        ? (_healthy ? "check_circle" : "warning") 
        : "cancel";
    
    private string GetStatusColor() => _serverInfo != null 
        ? (_healthy ? "color: var(--rz-success);" : "color: var(--rz-warning);") 
        : "color: var(--rz-text-disabled-color);";
    
    private string GetStatusText() => _serverInfo != null 
        ? (_healthy ? "Connected" : "Unhealthy") 
        : "Not Connected";
}
```

---

### Step 5: Add Settings Button to MainLayout

**File:** `frontend/desktop/opencode/Layout/MainLayout.razor`

**Replace entire file:**

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
                ButtonStyle="ButtonStyle.Light" 
                Variant="Variant.Text"
                Click="@(() => _settingsModal?.Show())"
                title="Settings" />
            <a href="https://learn.microsoft.com/aspnet/core/" target="_blank">About</a>
        </div>

        <article class="content px-4">
            @Body
        </article>
    </main>
</div>

<SettingsModal @ref="_settingsModal" />

@code {
    private SettingsModal? _settingsModal;
}
```

---

### Step 6: Register Components in _Imports.razor

**File:** `frontend/desktop/opencode/_Imports.razor`

**Add:**
```razor
@using OpenCode.Components
```

**Note:** This assumes we create the Components folder and files.

---

### Step 7: Create Components Directory Structure

**Required directory structure:**
```
frontend/desktop/opencode/
  Components/
    SettingsModal.razor
    ServerSection.razor
```

---

## Verification Steps

After implementation:

1. **Build verification:**
   ```bash
   cd frontend/desktop/opencode && dotnet build
   ```

2. **Manual testing checklist:**
   - [ ] Settings button appears in top-right
   - [ ] Clicking settings opens modal
   - [ ] Server status shows "Not Connected" initially
   - [ ] "Start Server" button spawns server
   - [ ] After spawn: shows URL, PID, Owned=Yes
   - [ ] "Stop Server" button stops server
   - [ ] "Refresh" button updates status
   - [ ] Stop button disabled for external servers
   - [ ] Error messages display on failure
   - [ ] ESC key closes modal
   - [ ] X button closes modal

3. **Integration testing:**
   - Start app with no OpenCode server running
   - Click settings → Should show "Not Connected"
   - Click "Start Server" → Should show connected status
   - Click "Stop Server" → Should return to "Not Connected"

---

## Risk Assessment

### Low Risk
- Adding methods to IpcClient (straightforward pattern)
- Creating Radzen modal (well-documented component library)

### Medium Risk  
- Modal state management (need proper show/hide handling)
- Proto message mapping (need to verify field names match)

### Mitigations
- Follow existing patterns from Home.razor
- Test each step incrementally
- Use Radzen's built-in dialog management

---

## Dependencies

**NuGet packages (already installed):**
- Radzen.Blazor (provides RadzenDialog, RadzenButton, etc.)
- Google.Protobuf (provides IpcServerInfo)

**No new dependencies required.**

---

## Files Summary

### Files to Create (3)
1. `frontend/desktop/opencode/Components/SettingsModal.razor`
2. `frontend/desktop/opencode/Components/ServerSection.razor`
3. (directory) `frontend/desktop/opencode/Components/`

### Files to Modify (3)
1. `frontend/desktop/opencode/Services/IIpcClient.cs` - Add server methods
2. `frontend/desktop/opencode/Services/IpcClient.cs` - Implement server methods
3. `frontend/desktop/opencode/Layout/MainLayout.razor` - Add settings button + modal
4. `frontend/desktop/opencode/_Imports.razor` - Add Components namespace

---

## Success Criteria

- [ ] `dotnet build` succeeds
- [ ] Settings button visible in app header
- [ ] Settings modal opens and closes properly
- [ ] Server section displays:
  - [ ] Connection status with appropriate icon
  - [ ] Server URL (when connected)
  - [ ] Server PID (when connected)
  - [ ] Owned status (when connected)
- [ ] "Refresh" button discovers server
- [ ] "Start Server" button spawns new instance
- [ ] "Stop Server" button works (disabled if not owned)
- [ ] Loading states shown during async operations
- [ ] Error messages displayed on failure

---

## Out of Scope (Future Sessions)

- Models section (Session 11)
- Audio settings section (later)
- UI preferences section (later)
- Config persistence from Blazor side (uses IPC update_config)
- Auto-refresh polling

---

**Start with:** Step 1 - Add the interface methods to IIpcClient.cs, then implement in IpcClient.cs. Build and verify compilation before creating UI components.
