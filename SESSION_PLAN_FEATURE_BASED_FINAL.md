# Session Plan - Feature-Based (FINAL)

**Status:** Ready for implementation  
**Goal:** Replace abstract "backend/frontend" split with concrete user-facing features  
**Based on:** Comprehensive audit of `submodules/opencode-egui/` reference implementation

---

## Philosophy

Each session delivers a **demonstrable feature** that can be tested and shown to users.

**Good (concrete):**
- Session 4.5: "Server Discovery + First Chat" ← Can demo: launch app, send message, see response
- Session 5: "Multi-Tab Sessions + Agent Selection" ← Can demo: open multiple chats, switch agents
- Session 6: "Tool Calls + Permission Dialogs" ← Can demo: see tool execution, approve permissions

---

## Feature Audit Summary

Based on comprehensive audit of egui client (see `EGUI_FEATURE_AUDIT.md`):

### MVP Features (Must Have)
1. ✅ Server discovery + spawn
2. ✅ Single session chat (send message, receive response)
3. ✅ SSE event streaming (message updates)
4. ✅ Basic message display (text, role, bubbles)
5. ✅ Tool call visualization (collapsible blocks)
6. ✅ Permission approval dialogs
7. ✅ Agent selection
8. ✅ Model selection

### Core Features (Should Have)
9. ✅ Multi-tab sessions
10. ✅ Markdown rendering
11. ✅ Reasoning display (collapsible)
12. ✅ Token counts
13. ✅ Message cancellation
14. ✅ Auth sync (API keys to server)

### Enhanced Features (Feature Parity)
15. ✅ OAuth mode toggle (Anthropic)
16. ✅ Settings panel
17. ✅ Config persistence
18. ✅ Tab rename
19. ✅ Clipboard image paste
20. ✅ OAuth countdown timer
21. ✅ Model discovery UI
22. ✅ Audio/STT (Session 9)

**Goal:** Full feature parity with egui reference implementation

---

## Session Breakdown

### Session 4.5: "Server Discovery + Basic Chat" ⭐⭐⭐

**User-facing goal:** Launch app, auto-discover server (or spawn), send a message, see a response.

**What you can demo:**
- App launches
- Server discovered/spawned automatically  
- Type message in input box
- Click send (or Cmd+Enter)
- See assistant response appear (text only)
- See basic tool execution (no permission yet)

**Technical scope:**
1. **gRPC Services** (proto definitions + code gen):
   - `ServerDiscovery` (find/spawn server)
   - `SessionManagement` (create/list/delete sessions)
   - `MessageService` (send message, stream events)

2. **Blazor Components**:
   - `MainLayout.razor` - App shell with status bar
   - `ChatView.razor` - Single tab, message list, input box
   - `MessageBubble.razor` - User/assistant message display (text only, no markdown)
   - `ToolCallBlock.razor` - Basic collapsible tool display (no permissions yet)

3. **State Management** (Fluxor):
   - `AppState` - Server info, single tab state
   - `TabState` - Messages, input, session ID
   - `DisplayMessage` - Message data (role, text, tool_calls)
   - `ToolCall` - Tool execution data (id, name, status, input, output)

4. **Server Discovery** (C# port of egui logic):
   - Try ports 4008-4018 with HTTP health check (`GET /doc`)
   - Spawn `opencode server` if not found
   - Wait for ready (health check in loop)

5. **Event Streaming** (gRPC server-side streaming):
   - Subscribe to `GlobalEvents` stream
   - Route events by type: `message.updated`, `message.part.updated`
   - Update state via Fluxor actions

**Out of scope:**
- Multi-tab (Session 5)
- Permissions (Session 6)
- Markdown rendering (Session 8)
- Settings/config (Session 8)

**Token estimate:** ~120K

**Success criteria:**
- [ ] App auto-discovers server on launch
- [ ] Can create session and send text message
- [ ] Messages appear in chat (user + assistant)
- [ ] Tool calls display with status (collapsed by default)
- [ ] Streaming updates work (text appears incrementally)

---

### Session 5: "Multi-Tab + Agent Selection" ⭐⭐

**User-facing goal:** Open multiple chat tabs, switch between them, select different agent per tab.

**What you can demo:**
- Click "+" to create new tab
- Switch between tabs (tab bar)
- Close tab with "X" button
- Each tab has independent chat history
- Select agent from sidebar (per tab)
- Agent name shown in footer

**Technical scope:**
1. **Tab Management**:
   - `TabBar.razor` - Tab list with +/X buttons
   - Multiple `TabState` in `AppState.Tabs`
   - Active tab index tracking
   - Session create/delete on tab open/close

2. **Agent System**:
   - `AgentPane.razor` - Left sidebar with agent list
   - Fetch agents via `GET /agent` (convert to gRPC)
   - Filter by mode (hide subagents by default)
   - Per-tab agent selection
   - Send agent with message: `agent: "build"`

3. **State Updates**:
   - `CreateTabAction` / `CloseTabAction` / `SwitchTabAction`
   - `SelectAgentAction(tabId, agentName)`
   - Route events to correct tab by `sessionID`

**Out of scope:**
- Model selection (Session 6)
- Permissions (Session 6)
- Tab rename (Session 8)

**Token estimate:** ~100K

**Success criteria:**
- [ ] Can create/close/switch tabs
- [ ] Each tab has unique session ID
- [ ] Agent pane shows list of agents
- [ ] Can select agent per tab
- [ ] Agent sent with message (verify in server logs)

---

### Session 6: "Tool Calls + Permissions" ⭐⭐⭐

**User-facing goal:** See tool execution in real-time, approve permission requests.

**What you can demo:**
- Send message that triggers tool use (e.g., "list files")
- Watch tool call status update (pending → running → success)
- See tool logs streaming in
- Permission dialog appears for restricted tools
- Click "Allow Once" / "Reject" / "Always Allow"
- See tool output after permission granted

**Technical scope:**
1. **Enhanced Tool Display**:
   - `ToolCallBlock.razor` - Full tool visualization
     - Header: status icon + name + command summary + duration
     - Body (expanded): COMMAND, INPUT, OUTPUT, ERROR, LOGS sections
     - Auto-expand if: running OR has_permission OR has_error
   - Smart command summary (extract `command`, `filePath`, `url` from input)
   - Scrollable output/logs sections

2. **Permission System**:
   - `PermissionDialog.razor` - Inline in tool block
   - Listen for `permission.updated` events
   - Store in `AppState.PendingPermissions`
   - Send `POST /session/{id}/permissions/{perm_id}` with response
   - Remove after response
   - **Auto-reject logic** (replicate egui complexity):
     ```csharp
     bool is_cancelled = 
         tab.CancelledMessages.Contains(perm.MessageId) ||
         tab.CancelledCalls.Contains(perm.CallId) ||
         perm.Time.Created <= tab.CancelledAfter ||
         perm.Time.Created <= tab.LastSendAt ||
         tab.SuppressIncoming;
     ```

3. **Message Cancellation**:
   - "Stop" button (shown when `tab.ActiveAssistant != null`)
   - Cancel active message → mark tools as "cancelled"
   - Send `POST /session/{id}/abort` (twice, 200ms apart)
   - Set `tab.CancelledAfter = now()` to block future events

4. **Event Handling** (Tool Updates):
   - `message.part.updated` (type=tool) → Update/create ToolCall
   - Incremental updates: status, logs, output, error
   - Find tool by `id` OR `call_id` (need dual index)

**Out of scope:**
- Markdown rendering (Session 8)
- Model selection (deferred to Session 7)

**Token estimate:** ~110K

**Success criteria:**
- [ ] Tool calls show in collapsed blocks
- [ ] Click to expand → see full details
- [ ] Auto-expand when permission needed
- [ ] Permission dialog shows inline
- [ ] Can approve/reject permissions
- [ ] Tool output appears after approval
- [ ] Can cancel active response

---

### Session 7: "Model Selection + Provider Status" ⭐⭐

**User-facing goal:** Change model/provider per session, see provider connection status.

**What you can demo:**
- Model picker dropdown in footer (shows providers + models)
- Select different model per tab
- See OAuth subscription indicator (🟢)
- Switch auth mode (OAuth ↔ API key) for Anthropic
- See OAuth expiry countdown

**Technical scope:**
1. **Model Management**:
   - `ModelSelector.razor` - Dropdown in footer
   - Curated models list (hardcoded or from config)
   - Per-tab model selection
   - Send model with message:
     ```json
     { "model": { "providerID": "anthropic", "modelID": "claude-3-5-sonnet-20241022" } }
     ```
   - Display current model in footer

2. **Auth Sync**:
   - `AuthSyncService` - Background service
   - Load `.env` file from app directory
   - Extract `{PROVIDER}_API_KEY` variables
   - Send to server: `PUT /auth/{provider}` with `{ "type": "api", "key": "..." }`
   - Display sync status in settings

3. **OAuth Mode Toggle** (Anthropic only):
   - Checkbox in footer: ☐ API Key / ☑ Subscription
   - **Switch to OAuth**:
     1. Read OAuth tokens from `.env` cache
     2. `PUT /auth/anthropic` with `{ "type": "oauth", "access": "...", "refresh": "...", "expires": ... }`
     3. `POST /instance/dispose` to reload server
   - **Switch to API Key**:
     1. Read `ANTHROPIC_API_KEY` from `.env`
     2. `PUT /auth/anthropic` with `{ "type": "api", "key": "..." }`
     3. `POST /instance/dispose`

4. **Provider Status**:
   - Fetch `GET /provider` → `{ "connected": ["anthropic", "openai"] }`
   - Show 🟢 indicator for OAuth providers in model selector
   - OAuth expiry countdown (⏱ 23h 59m remaining)
   - Color-coded: 🟢 green (>5m), 🟡 yellow (0-5m), 🔴 red (expired)

**Out of scope:**
- Model discovery (use hardcoded list for MVP)

**Token estimate:** ~90K

**Success criteria:**
- [ ] Model selector shows curated models
- [ ] Can select model per tab
- [ ] Model sent with message (verify in server logs)
- [ ] Auth sync runs on startup
- [ ] Can toggle OAuth mode for Anthropic
- [ ] OAuth expiry countdown shows when enabled

---

### Session 8: "Markdown + Full UX Parity" ⭐⭐⭐

**User-facing goal:** Beautiful message rendering + complete egui feature parity.

**What you can demo:**
- Markdown in messages (code blocks, lists, headers)
- Syntax highlighting in code blocks
- Reasoning sections (collapsible)
- Token counts displayed below messages
- **Tab rename** (right-click → rename)
- **Clipboard image paste** (📋 button)
- OAuth countdown timer (⏱ 23h 59m)
- Settings panel (server, UI, models)
- Config persistence across restarts

**Technical scope:**
1. **Markdown Rendering**:
   - Use **Markdig** library (C# markdown parser)
   - Custom Blazor components for rendering:
     - `MarkdownText.razor` - Render markdown to HTML
     - `CodeBlock.razor` - Syntax highlighting (use **Highlight.js**)
   - Normalize code fences (ensure `\n` before ` ``` `)

2. **Reasoning Display**:
   - `ReasoningSection.razor` - Collapsible section
   - Default open if message text is empty
   - Auto-collapse when message finishes

3. **Token Counts**:
   - Display below message text
   - Format: `tokens: in 1234, out 567, reason 89`
   - Small, gray text

4. **Settings Panel**:
   - `SettingsDialog.razor` - Modal dialog
   - **Server Preferences**:
     - Base URL override
     - Directory override
     - Auto-start toggle
     - Server status (URL, PID, owned)
     - Reconnect / Start / Stop buttons
   - **UI Preferences**:
     - Font size (small/standard/large)
     - Chat density (compact/normal/comfortable)
     - Show subagents toggle
   - **Models Preferences**:
     - Curated models list
     - Default model selector

5. **Config Persistence**:
   - Save config to Tauri app data directory
   - Auto-load on startup
   - Structure:
     ```json
     {
       "server": {
         "lastBaseUrl": "http://localhost:4008",
         "autoStart": true,
         "directoryOverride": null
       },
       "ui": {
         "fontSize": "standard",
         "chatDensity": "normal"
       },
       "models": {
         "defaultModel": "anthropic/claude-3-5-sonnet-20241022",
         "curatedModels": [...]
       }
     }
     ```

6. **Tab Rename**:
   - Right-click context menu on tab
   - Inline text edit with focus + select all
   - Enter/Tab to confirm, Escape to cancel

7. **Clipboard Image Paste**:
   - "📋 Paste Image" button in input area
   - Tauri clipboard API for cross-platform access
   - PNG encoding + base64 data URI
   - Preview list with remove buttons

8. **OAuth Countdown Timer**:
   - Footer display: `⏱ 23h 59m remaining`
   - Color coding: 🟢 >5m, 🟡 0-5m, 🔴 expired
   - Update every second when enabled
   - "🔄 Refresh" button

**Token estimate:** ~100K (+20K for full UX parity)

**Success criteria:**
- [ ] Markdown renders correctly (lists, code, headers)
- [ ] Code blocks have syntax highlighting
- [ ] Reasoning sections collapse/expand
- [ ] Token counts displayed
- [ ] **Tab rename works** (right-click, inline edit)
- [ ] **Clipboard paste works** (images appear in attachments)
- [ ] **OAuth timer displays** (color-coded countdown)
- [ ] Settings panel opens/closes
- [ ] Config persists across app restarts

---

## Total Token Budget

| Session | Feature | Estimate | Running Total |
|---------|---------|----------|---------------|
| 4.5 | Server + Basic Chat | 120K | 120K |
| 5 | Multi-Tab + Agents | 100K | 220K |
| 6 | Tool Calls + Permissions | 110K | 330K |
| 7 | Model Selection + Discovery + Auth | **120K** | 450K |
| 8 | Markdown + **Full UX Parity** | **100K** | **550K** |
| 9 | Audio/STT | 80K | 630K |

**Full Feature Parity: ~630K tokens**

**Goal:** Match egui reference implementation feature-for-feature

---

### Session 9: "Audio/STT Integration" ⭐⭐

**User-facing goal:** Hands-free input via push-to-talk audio transcription.

**What you can demo:**
- Hold AltRight (or configured key) to record
- Release to transcribe
- Transcribed text appears in input box
- Visual feedback ("🎙 Recording...")
- Configurable hotkey in settings

**Technical scope:**
1. **Tauri Audio Plugin**:
   - Cross-platform audio capture (Windows/macOS/Linux)
   - Real-time audio streaming to buffer

2. **Whisper Integration**:
   - Download `ggml-base.en.bin` (74MB) on first use
   - Load model in background thread
   - Whisper.cpp or whisper-rs bindings for C#
   - Inference on recorded audio

3. **Audio Processing**:
   - Resample to 16kHz mono (Whisper requirement)
   - VAD (Voice Activity Detection) for cleaner transcripts
   - Background processing (don't block UI)

4. **UI Components**:
   - Push-to-talk state machine (Idle → Recording → Processing → Done)
   - Recording indicator in input area
   - Audio settings in Settings panel:
     - Push-to-talk key configuration
     - Whisper model path
     - Auto-download model toggle

5. **Config**:
   ```json
   {
     "audio": {
       "pushToTalkKey": "AltRight",
       "whisperModelPath": null,  // null = auto-download
       "autoDownloadModel": true
     }
   }
   ```

**Token estimate:** ~80K

**Success criteria:**
- [ ] Push-to-talk works (hold key, record, release, transcribe)
- [ ] Whisper model auto-downloads on first use
- [ ] Transcription appears in input box
- [ ] Recording indicator shows during capture
- [ ] Hotkey configurable in settings
- [ ] Works on all platforms (Windows, macOS, Linux)

**Why include audio:**
- ✅ Egui has it - we should have parity
- ✅ Accessibility feature (mobility-impaired users)
- ✅ Productivity boost (faster than typing)
- ✅ Tauri makes it achievable (audio plugins exist)

**Required for full feature parity with egui.**

---

## Implementation Notes

### State Management Architecture

Use **Fluxor** (Redux-like state management for Blazor):

```csharp
// State
public record AppState
{
    public ServerInfo? Server { get; init; }
    public List<TabState> Tabs { get; init; } = new();
    public int ActiveTabIndex { get; init; }
    public List<AgentInfo> Agents { get; init; } = new();
    public List<PermissionInfo> PendingPermissions { get; init; } = new();
}

public record TabState
{
    public string TabId { get; init; }
    public string Title { get; init; }
    public string? SessionId { get; init; }
    public List<DisplayMessage> Messages { get; init; } = new();
    public string Input { get; init; } = "";
    public string? SelectedAgent { get; init; }
    public ModelSelection? SelectedModel { get; init; }
    public string? ActiveAssistant { get; init; }
    public List<string> CancelledMessages { get; init; } = new();
    public List<string> CancelledCalls { get; init; } = new();
    public long? CancelledAfter { get; init; }
    public bool SuppressIncoming { get; init; }
}

// Actions
public record AddMessageAction(string TabId, DisplayMessage Message);
public record UpdateMessageTextAction(string TabId, string MessageId, string Text);
public record UpdateToolCallAction(string TabId, string MessageId, ToolCall ToolCall);
public record AddPermissionAction(PermissionInfo Permission);
public record RemovePermissionAction(string PermissionId);

// Reducers
public class AppReducers
{
    [ReducerMethod]
    public static AppState ReduceAddMessage(AppState state, AddMessageAction action)
    {
        var tabIndex = state.Tabs.FindIndex(t => t.TabId == action.TabId);
        if (tabIndex == -1) return state;
        
        var updatedTab = state.Tabs[tabIndex] with
        {
            Messages = state.Tabs[tabIndex].Messages.Append(action.Message).ToList()
        };
        
        return state with
        {
            Tabs = state.Tabs.Select((t, i) => i == tabIndex ? updatedTab : t).ToList()
        };
    }
}
```

### Event Streaming Pattern

```csharp
public class EventStreamService : BackgroundService
{
    private readonly IDispatcher _dispatcher;
    private readonly GrpcChannel _channel;
    
    protected override async Task ExecuteAsync(CancellationToken stoppingToken)
    {
        var client = new EventService.EventServiceClient(_channel);
        
        var stream = client.SubscribeGlobalEvents(new Empty(), cancellationToken: stoppingToken);
        
        await foreach (var evt in stream.ResponseStream.ReadAllAsync(stoppingToken))
        {
            // Route by event type
            switch (evt.Type)
            {
                case "message.updated":
                    _dispatcher.Dispatch(new MessageUpdatedAction(evt));
                    break;
                case "message.part.updated":
                    _dispatcher.Dispatch(new MessagePartUpdatedAction(evt));
                    break;
                case "permission.updated":
                    _dispatcher.Dispatch(new PermissionUpdatedAction(evt));
                    break;
            }
        }
    }
}
```

### Tool Call Updates (Efficient Lookup)

```csharp
// Use dual index for fast lookup by ID or CallID
public class ToolCallIndex
{
    private readonly Dictionary<string, ToolCall> _byId = new();
    private readonly Dictionary<string, ToolCall> _byCallId = new();
    
    public void AddOrUpdate(ToolCall tool)
    {
        _byId[tool.Id] = tool;
        if (tool.CallId != null)
            _byCallId[tool.CallId] = tool;
    }
    
    public ToolCall? Find(string? id, string? callId)
    {
        if (id != null && _byId.TryGetValue(id, out var tool))
            return tool;
        if (callId != null && _byCallId.TryGetValue(callId, out tool))
            return tool;
        return null;
    }
}
```

---

## Deferred Features (Post-MVP)

### Audio/STT ❌
**Why defer**: Very complex
- Requires Whisper model (74MB binary)
- Platform-specific audio capture
- Resampling to 16kHz mono
- Local ML inference (heavy CPU/GPU)

**Alternative**: Text input only for MVP

---

### Model Discovery ❌
**Why defer**: Requires provider API clients
- Need API client for each provider
- Dynamic model fetching from provider APIs
- Search/filter UI

**Alternative**: Hardcoded curated models list in config

---

### Clipboard Image Paste ❌
**Why defer**: Platform-specific, minor feature
- Requires Tauri plugin
- Platform-specific clipboard access
- Image encoding

**Alternative**: Text-only for MVP

---

### Tab Rename ❌
**Why defer**: Minor UX polish
- Context menu
- Inline edit
- Keyboard shortcuts

**Alternative**: Use session ID as tab title for MVP

---

## Next Steps

1. ✅ **Review this plan** - Does it match your vision?
2. ✅ **Finalize Session 4.5 scope** - Ready to start implementation?
3. ✅ **Create NEXT_SESSION_PROMPT.md** - Detailed instructions for Session 4.5
4. ⏳ **Start Session 4.5** - Implement server discovery + basic chat

---

**This plan is ready for implementation. Proceed to Session 4.5!**
