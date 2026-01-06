# Session Plan: OpenCode Tauri-Blazor Desktop Client

**Goal:** Feature parity with egui reference implementation  
**Reference:** `submodules/opencode-egui/` and `docs/EGUI_ARCHITECTURE.md`  
**Token limit:** 120K per session (hard cap)

---

## Naming Conventions

### Proto Message Prefixes
- **`Ipc*`** - IPC messages (Blazor ↔ client-core WebSocket)
  - `IpcClientMessage`, `IpcServerMessage`, `IpcAuthHandshake`, `IpcChatToken`, etc.
- **`Oc*`** - OpenCode data models (mirroring OpenCode server types)
  - `OcSession`, `OcMessage`, `OcAgent`, `OcProvider`, etc.

### Field/Variable Names
- **`opencode_*`** - OpenCode server (the TypeScript AI server on ports 4008-4018)
  - `opencode_url`, `opencode_port`, `opencode_client`, `OpencodeClient`
- **`ipc_*`** - IPC server (the WebSocket server in client-core)
  - `ipc_url`, `ipc_port`, `ipc_server`, `IpcServer`, `IpcClient`

**Never use ambiguous names like `server_url`, `port`, `client`, or `base_url`.**

---

## Completed Sessions (1-4)

| Session | Deliverable | Status |
|---------|-------------|--------|
| 1 | Rust client-core (discovery, spawn, health) | DONE |
| 2 | Tauri app shell + server commands | DONE |
| 3 | Blazor scaffold + server status UI | DONE |
| 4 | Proto documentation (72+ JSON schemas) | DONE |

**Current state:** App launches, discovers/spawns server, shows status page.

---

## Phase 1: Communication Foundation (Sessions 5-8)

### Session 5: IPC Server - Echo
**Demo:** Connect with wscat, send text, receive echo.

- Create `backend/client-core/src/ipc/server.rs`
- Bind to `127.0.0.1:{ipc_port}`
- Accept connections
- Echo received text back
- Plain text, no protobuf yet

**Success:** `wscat -c ws://127.0.0.1:{ipc_port}` → send "hello" → receive "hello"

---

### Session 6: IPC Server - Auth + Protobuf
**Demo:** Auth handshake works, protobuf messages parse.

- **Apply naming conventions:** Rename existing protos to `Ipc*` and `Oc*` prefixes
- **Apply naming conventions:** Rename fields to `ipc_*` and `opencode_*`
- Add auth token validation (first message must be `IpcAuthHandshake`)
- Switch to binary frames (protobuf)
- Parse `IpcClientMessage` envelope
- Send `IpcServerMessage` responses
- Reject invalid auth

**Success:** Auth handshake succeeds, invalid token rejected

---

### Session 7: IPC Server - Session Handlers
**Demo:** Can list/create/delete sessions via IPC.

- Implement `IpcListSessions` handler → `GET {opencode_url}/session`
- Implement `IpcCreateSession` handler → `POST {opencode_url}/session`
- Implement `IpcDeleteSession` handler → `DELETE {opencode_url}/session/{id}`
- HTTP client in client-core for OpenCode server calls (`OpencodeClient`)

**Success:** Create session via wscat, verify in OpenCode server

---

### Session 8: C# IPC Client
**Demo:** Blazor connects to IPC server, lists sessions in UI.

- Create `IpcClientService.cs` using `System.Net.WebSockets.ClientWebSocket`
- `IpcAuthHandshake` on connect
- Background receive loop
- Protobuf serialization (`Google.Protobuf`)
- Simple test page showing session list

**Success:** Blazor UI shows list of sessions from OpenCode server

---

## Phase 2: Config & Auth (Sessions 9-12)

**Why this phase comes before chat:** You can't send a message without a model selected and auth configured.

### Session 9: Config Loading
**Demo:** App loads settings from disk on startup.

- Create `AppConfig` class (mirrors egui's config.json)
- Create `ModelsConfig` class (mirrors egui's models.toml)
- Load on startup, save on change
- Platform-specific paths (use Tauri app data directory)

**Success:** Change setting, restart app, setting persists

---

### Session 10: Settings Panel - Server Section
**Demo:** Settings modal opens, shows server status.

- Settings button in footer
- Modal dialog with Server section
- Display: base URL, PID, owned status
- Buttons: Reconnect, Start, Stop

**Success:** Can see server status in settings

---

### Session 11: Settings Panel - Models Section  
**Demo:** Can see and select default model.

- Models section in settings
- Curated models list display
- Default model dropdown
- Model selector in footer (per-tab selection)

**Success:** Select model from dropdown, verify it's sent with message

---

### Session 12: Auth Sync
**Demo:** API keys sync to OpenCode server on connect.

- Read `.env` file for `*_API_KEY` variables
- On OpenCode server connect: `PUT {opencode_url}/auth/{provider}` for each key
- Display sync status in settings (success/failure per provider)
- Skip Anthropic if OAuth detected

**Success:** See "✓ Synced: openai, anthropic" in settings

---

## Phase 3: Basic Chat (Sessions 13-16)

### Session 13: Chat UI Shell
**Demo:** Input box and message list visible.

- Chat page with input area
- Send button (disabled when no session)
- Message list (empty initially)
- Create session on page load
- Display session ID in footer

**Success:** Page loads, session created, input box visible

---

### Session 14: Send Message - Non-Streaming
**Demo:** Send message, see complete response.

- `IpcSendMessage` proto message
- client-core forwards to `POST {opencode_url}/session/{id}/message`
- Wait for response, display in message list
- User bubble (right), assistant bubble (left)
- Include model and agent in request

**Success:** Send "hello" → see Claude's response

---

### Session 15: SSE Subscription
**Demo:** App receives real-time events from OpenCode server.

- Subscribe to `GET {opencode_url}/global/event` in client-core
- Parse SSE events
- Forward to IPC as `IpcServerMessage`
- Log events in Blazor console

**Success:** Send message, see SSE events logged

---

### Session 16: Streaming Display
**Demo:** Response appears word-by-word.

- `IpcChatToken` proto message
- Update message text incrementally
- Show "Thinking..." spinner while waiting
- Clear spinner when text arrives

**Success:** Response streams visibly

---

## Phase 4: Agents (Sessions 17-19)

### Session 17: Agent Fetch + List
**Demo:** Agent pane shows list of agents.

- Fetch agents on OpenCode server connect (`GET {opencode_url}/agent`)
- Left sidebar pane (collapsible)
- Display agent name, color dot, badges
- Filter out subagents by default

**Success:** See agent list in sidebar

---

### Session 18: Agent Selection
**Demo:** Select agent, verify it's sent with message.

- Click agent to select
- Per-tab agent selection
- Show selected agent in footer
- Include agent in `IpcSendMessage` request

**Success:** Select "expert-developer", send message, verify in OpenCode server logs

---

### Session 19: Agent Filtering
**Demo:** Toggle shows/hides subagents.

- "Show subagents" toggle in settings
- Re-filter list when toggled
- Reset invalid selections to default

**Success:** Toggle on → see subagents, toggle off → hidden

---

## Phase 5: Tools & Permissions (Sessions 20-24)

### Session 20: Tool Call Display - Basic
**Demo:** See tool calls in chat.

- `IpcToolCallEvent` proto message
- Tool block component (collapsible)
- Show: status icon, name, command summary
- Collapsed by default

**Success:** Ask "list files", see tool block appear

---

### Session 21: Tool Call Display - Details
**Demo:** Expand tool to see full details.

- Expanded view: COMMAND, INPUT, OUTPUT, ERROR, LOGS
- Scrollable output section
- Duration display
- Auto-expand when running or error

**Success:** Expand tool, see full input/output

---

### Session 22: Permission Dialog
**Demo:** Permission request appears in tool block.

- `permission.updated` event handling
- Red warning box in tool header
- Buttons: Reject, Allow Once, Always Allow
- Send response to server

**Success:** Tool triggers permission, can approve

---

### Session 23: Permission Auto-Reject
**Demo:** Cancelled messages don't show permission dialogs.

- Track cancelled_messages, cancelled_calls, cancelled_after
- Auto-reject permissions for cancelled work
- 5-condition filter logic (see EGUI_ARCHITECTURE.md)

**Success:** Cancel response, no stale permission dialogs

---

### Session 24: Message Cancellation
**Demo:** Stop button cancels active response.

- Stop button (visible when streaming)
- Mark message/tools as cancelled
- Send `POST {opencode_url}/session/{id}/abort`
- Set suppress_incoming flag

**Success:** Click Stop, response stops, tools marked cancelled

---

## Phase 6: Multi-Tab (Sessions 25-27)

### Session 25: Tab Bar
**Demo:** Create and switch between tabs.

- Tab bar component
- "+" button to create tab
- Click tab to switch
- Active tab highlighting

**Success:** Create 3 tabs, switch between them

---

### Session 26: Tab State Isolation
**Demo:** Each tab has independent chat.

- Per-tab session ID
- Per-tab message list
- Per-tab model/agent selection
- Route SSE events by session ID

**Success:** Different conversations in different tabs

---

### Session 27: Tab Close + Rename
**Demo:** Close tabs, rename via context menu.

- "X" button to close tab
- Delete session on close
- Right-click context menu
- Inline rename with Enter/Escape

**Success:** Rename tab, close tab

---

## Phase 7: Advanced Auth (Sessions 28-30)

### Session 28: Provider Status
**Demo:** See which providers are connected.

- Fetch `GET {opencode_url}/provider` on OpenCode server connect
- Show indicators in model selector
- 🟢 for connected OAuth providers

**Success:** See provider status indicators

---

### Session 29: OAuth Mode Toggle
**Demo:** Switch Anthropic between API key and OAuth.

- Checkbox in footer: API Key / Subscription
- Read OAuth tokens from .env cache
- `PUT {opencode_url}/auth/anthropic` with OAuth tokens
- `POST {opencode_url}/instance/dispose` to reload

**Success:** Toggle to OAuth, verify in OpenCode server logs

---

### Session 30: OAuth Countdown
**Demo:** See OAuth expiry timer.

- Display: "⏱ 23h 59m remaining"
- Color coding: 🟢 >5m, 🟡 0-5m, 🔴 expired
- Update every second
- Refresh button

**Success:** See countdown ticking

---

## Phase 8: Model Discovery (Sessions 31-33)

### Session 31: Model Discovery UI
**Demo:** Open discovery modal, see provider list.

- "+ Add Model" button in settings
- Modal with provider buttons (OpenAI, Anthropic, Google, OpenRouter)
- Click provider to start discovery

**Success:** Click OpenAI, see loading state

---

### Session 32: Provider API Calls
**Demo:** Fetch models from provider API.

- Read API key from environment
- Call provider's models endpoint
- Parse response using provider config
- Display model list with search

**Success:** See 50+ OpenAI models listed

---

### Session 33: Add/Remove Models
**Demo:** Add model to curated list.

- "+" button next to each discovered model
- Add to models.toml
- Remove button in curated list
- Prevent duplicates

**Success:** Add GPT-4, see it in curated list

---

## Phase 9: Rendering & Polish (Sessions 34-38)

### Session 34: Markdown Rendering
**Demo:** Code blocks and formatting display correctly.

- Markdig for markdown parsing
- Code fence normalization
- Render to HTML in Blazor

**Success:** Send "show me a code example", see formatted code

---

### Session 35: Syntax Highlighting
**Demo:** Code blocks have colored syntax.

- Highlight.js or similar
- Language detection
- Theme matching app style

**Success:** Python code has colored keywords

---

### Session 36: Reasoning Sections
**Demo:** Extended thinking in collapsible section.

- `reasoning_parts` handling
- Collapsible "Reasoning" section
- Default open if no text yet
- Auto-collapse when done

**Success:** See reasoning expand/collapse

---

### Session 37: Token Counts
**Demo:** See token usage below messages.

- Display: "tokens: in 123, out 456, reason 78"
- Small gray text
- Only for assistant messages

**Success:** See token counts after response

---

### Session 38: UI Preferences
**Demo:** Change font size, density.

- Font size: Small/Standard/Large
- Chat density: Compact/Normal/Comfortable
- Live preview
- Persist to config

**Success:** Change to Large font, restart, still Large

---

## Phase 10: Attachments & Audio (Sessions 39-42)

### Session 39: Clipboard Image Paste
**Demo:** Paste image from clipboard.

- "📋 Paste" button
- Tauri clipboard API
- PNG encoding
- Preview with remove button

**Success:** Paste screenshot, see preview

---

### Session 40: Send Attachments
**Demo:** Image sent with message.

- Base64 encode image
- Include in message parts as `{ type: "file", mime: "image/png", url: "data:..." }`
- Clear attachments after send

**Success:** Send image, Claude describes it

---

### Session 41: Audio Capture
**Demo:** Push-to-talk records audio.

- Tauri audio plugin
- Configurable hotkey
- Recording indicator
- State machine: Idle → Recording → Processing

**Success:** Hold key, see "🎙 Recording..."

---

### Session 42: Whisper Transcription
**Demo:** Speech appears as text.

- Whisper model loading
- Resample to 16kHz mono
- Local inference
- Append to input box

**Success:** Speak, see transcription in input

---

## Phase 11: Ship (Sessions 43-45)

### Session 43: Error Handling
**Demo:** Errors display gracefully.

- Network errors
- Server errors
- Validation errors
- Toast notifications

**Success:** Disconnect network, see error message

---

### Session 44: Cross-Platform Testing
**Demo:** Works on Mac, Windows, Linux.

- Test on each platform
- Fix platform-specific issues
- Build verification

**Success:** All platforms work

---

### Session 45: Documentation + Release
**Demo:** Ready for users.

- README updates
- Build instructions
- Release process

**Success:** Someone else can build and use it

---

## Summary

| Phase | Sessions | Features |
|-------|----------|----------|
| 1. Communication | 5-8 | IPC server + client |
| 2. Config & Auth | 9-12 | Settings, models, API key sync |
| 3. Basic Chat | 13-16 | Send/receive messages, streaming |
| 4. Agents | 17-19 | Agent pane, selection, filtering |
| 5. Tools | 20-24 | Tool display, permissions, cancellation |
| 6. Multi-Tab | 25-27 | Tab bar, isolation, rename |
| 7. Advanced Auth | 28-30 | Provider status, OAuth toggle, countdown |
| 8. Model Discovery | 31-33 | Dynamic model fetching |
| 9. Rendering | 34-38 | Markdown, syntax, reasoning, tokens |
| 10. Attachments | 39-42 | Clipboard paste, audio/STT |
| 11. Ship | 43-45 | Polish and release |

**Total: 41 sessions (Sessions 5-45)**
