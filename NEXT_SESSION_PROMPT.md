# Next Session: Session 10 - Settings Panel (Server Section)

## Quick Context

**What We Completed (Sessions 5-9):**

- ✅ IPC WebSocket server with binary protobuf protocol
- ✅ Session handlers (list/create/delete) via OpenCode HTTP API
- ✅ C# IPC client with production-grade WebSocket management
- ✅ Home.razor displaying sessions using Radzen components
- ✅ Config management system with IPC handlers

**Current State:**

- App launches, loads config, connects to IPC server via WebSocket
- Blazor authenticates and can list/create/delete sessions
- Config is available via IPC (get_config/update_config)
- **BUT:** No UI for settings - users can't see server status or start/stop servers

---

## Your Mission: Session 10

Create a Settings modal dialog with a Server section that displays:
- Connection status (icon + text)
- Server URL, PID, Owned status
- Buttons: Refresh, Start Server, Stop Server

---

## Implementation Plan

### Step 1: Add Server Methods to IIpcClient Interface

**File:** `frontend/desktop/opencode/Services/IIpcClient.cs`

**Add after `DeleteSessionAsync`:**
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

/// <summary>
/// Checks if the OpenCode server is healthy.
/// </summary>
/// <returns>True if server is responding.</returns>
Task<bool> CheckServerHealthAsync(CancellationToken cancellationToken = default);
```

**Verification:** `dotnet build` should fail (IpcClient doesn't implement new methods yet)

---

### Step 2: Implement Server Methods in IpcClient

**File:** `frontend/desktop/opencode/Services/IpcClient.cs`

**Add after `DeleteSessionAsync` method (~line 479):**

```csharp
public async Task<IpcServerInfo?> DiscoverServerAsync(CancellationToken cancellationToken = default)
{
    var request = new IpcClientMessage
    {
        DiscoverServer = new IpcDiscoverServerRequest()
    };

    var response = await SendRequestAsync(request, cancellationToken: cancellationToken);
    return response.DiscoverServerResponse?.Server;
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

**Verification:** `dotnet build` should succeed

---

### Step 3: Create Components Directory

**Create directory:** `frontend/desktop/opencode/Components/`

```bash
mkdir -p frontend/desktop/opencode/Components
```

---

### Step 4: Create ServerSection Component

**File to create:** `frontend/desktop/opencode/Components/ServerSection.razor`

This component displays server status and provides control buttons.

**Key elements:**
- Status row with icon (green=connected, gray=disconnected, yellow=unhealthy)
- URL, PID, Owned rows (only shown when connected)
- Error alert (dismissible)
- Button row: Refresh, Start Server, Stop Server
- Loading progress bar

**Pattern to follow:** Copy error handling and async patterns from `Home.razor`

**See `Session_10_Plan.md` for complete component code.**

---

### Step 5: Create SettingsModal Component

**File to create:** `frontend/desktop/opencode/Components/SettingsModal.razor`

Simple modal shell using RadzenDialog:
- Title: "Settings"
- Contains: ServerSection component
- Has Show()/Hide() methods for parent to call

**See `Session_10_Plan.md` for complete component code.**

---

### Step 6: Update _Imports.razor

**File:** `frontend/desktop/opencode/_Imports.razor`

**Add:**
```razor
@using OpenCode.Components
```

---

### Step 7: Add Settings Button to MainLayout

**File:** `frontend/desktop/opencode/Layout/MainLayout.razor`

**Modify to:**
1. Add a settings button (gear icon) in top-row
2. Add SettingsModal component reference
3. Wire button click to show modal

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

## Proto Message Reference

The proto messages you'll use are already defined in `proto/ipc.proto`:

```protobuf
// IpcClientMessage.payload options:
IpcDiscoverServerRequest discover_server = 15;
IpcSpawnServerRequest spawn_server = 16;
IpcCheckHealthRequest check_health = 17;
IpcStopServerRequest stop_server = 18;

// IpcServerMessage.payload options:
IpcDiscoverServerResponse discover_server_response = 15;
IpcSpawnServerResponse spawn_server_response = 16;
IpcCheckHealthResponse check_health_response = 17;
IpcStopServerResponse stop_server_response = 18;

// Server info structure:
message IpcServerInfo {
  uint32 pid = 1;
  uint32 port = 2;
  string base_url = 3;
  string name = 4;
  string command = 5;
  bool owned = 6;  // true = we spawned it
}
```

---

## Success Criteria

- [ ] `dotnet build` succeeds
- [ ] Settings button (gear icon) visible in top-right header
- [ ] Clicking settings opens modal dialog
- [ ] Server section displays:
  - [ ] Status with icon (green check = connected, gray X = not connected)
  - [ ] URL, PID, Owned (when connected)
- [ ] "Refresh" button discovers server and updates display
- [ ] "Start Server" button spawns server (disabled when already connected)
- [ ] "Stop Server" button stops server (disabled if not owned or not connected)
- [ ] Loading indicator shown during async operations
- [ ] Errors displayed in alert box
- [ ] ESC key closes modal
- [ ] X button closes modal

---

## Key Files to Reference

**Existing patterns:**
- `frontend/desktop/opencode/Pages/Home.razor` - Async patterns, error handling, Radzen components
- `frontend/desktop/opencode/Services/IpcClient.cs` - SendRequestAsync pattern
- `proto/ipc.proto` - Message definitions

**Full implementation details:**
- `Session_10_Plan.md` - Complete code for all components

---

## Important Reminders

1. **Follow Home.razor patterns** for error handling and loading states
2. **Use Radzen components** (RadzenDialog, RadzenButton, RadzenAlert, etc.)
3. **Handle all exceptions** - wrap async calls in try/catch
4. **Test incrementally** - build after each step
5. **Proto field names** - C# uses PascalCase (e.g., `DiscoverServerResponse`, `BaseUrl`)

---

**Start with:** Step 1 - Add interface methods to `IIpcClient.cs`
