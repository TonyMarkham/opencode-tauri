# Next Session: Chat UI & Message Streaming

---

## ⚠️ CRITICAL ARCHITECTURAL PRINCIPLE

**This implementation differs fundamentally from egui by using gRPC + Protobuf as the IPC mechanism.**

### The Three-Layer Architecture

```
Blazor (C#)
    ↓ [gRPC/Protobuf]
Client Core (Rust) ← THE ENGINE
    ├── Session management
    ├── Message handling
    ├── OpenCode server communication (HTTP REST)
    ├── Process discovery (sysinfo, netstat2)
    ├── Process spawning (tokio::process)
    └── Server health checks
         ↑ (uses)
Tauri App Layer (Rust)
    └── Webview host only
        (hosts Blazor, initializes services)
```

### Why This Matters

- **egui:** Uses Tauri IPC (JavaScript message passing) to communicate with the Rust backend
- **Tauri + Blazor:** Uses **gRPC + Protobuf** for ALL C# ↔ Rust communication
- **Advantage:** Type-safe, generated code, zero JSON parsing bugs, separation of concerns

### The Separation of Concerns

1. **Blazor (C#):** UI layer only
   - Receives user input
   - Calls gRPC methods
   - Renders responses

2. **Client Core (Rust):** Application layer (all the heavy lifting)
   - Session management
   - Message handling
   - OpenCode server communication
   - Process discovery and spawning
   - Server health checks
   - **This is where the real logic lives**

3. **Tauri App (Rust):** Webview host + OS bridge
   - Hosts the Blazor webview
   - Initializes and runs gRPC server
   - Manages app state
   - **Thin orchestrator, not business logic**

**Key Principle:** Client Core does the work. Tauri just hosts it and provides the webview.

### Pattern: ServerInfo as Template

Session 3 established the pattern with `ServerInfo`:

- Defined once in `server.proto`
- Rust: Generated via prost, wrapped with validation
- C#: Generated via protobuf compiler
- **Both sides always work with the same type**

**Session 4 applies this pattern to all chat messages, sessions, and events.**

---

## Quick Context

**What We Completed (Session 3 - 2026-01-04):**

- ✅ Created Blazor WASM project at `clients/tauri-blazor/frontend/desktop/opencode/`
- ✅ Implemented `IServerService` + `ServerService` with IJSRuntime Tauri commands
- ✅ Built Home.razor with server discovery/spawn/stop UI
- ✅ Wired up Radzen components (Cards, Buttons, Alerts, Notifications)
- ✅ Published Blazor to `apps/desktop/opencode/frontend/`
- ✅ All 4 server commands working end-to-end via Tauri IPC

**Current State:**

- Tauri backend fully functional (spawn/stop server) ✅
- Blazor frontend loads successfully ✅
- Server discovery/management working ✅
- **BUT:** No chat UI yet - need sessions, messages, and streaming

---

## Your Mission: Session 4 - Chat UI & Message Streaming

Build the chat interface using **gRPC + Protobuf** for all C# ↔ Rust communication.

### Data Flow

```
Blazor (C#)
    ↓ [gRPC/Protobuf]
Client Core (Rust) ← THE APPLICATION ENGINE
    ├── Session Management
    ├── Message Handling
    ├── OpenCode Server Communication (HTTP REST)
    └── Process Discovery & Spawning
         ↓ [uses]
    Tauri App Layer (just the host)

OpenCode Server (localhost:PORT)
    ↓ [HTTP REST]
Claude API / Other Providers
```

### What Blazor Does (UI Layer)

1. Accept user input (text messages, file uploads)
2. Call gRPC methods on Tauri backend
3. Receive protobuf messages (sessions, messages, events)
4. Render UI with markdown support
5. **Never directly calls OpenCode server**

### What Client Core Does (Application Layer)

1. Implements gRPC service (all chat logic lives here)
2. Manages sessions via OpenCode server HTTP REST API
3. Streams response events back to Blazor via gRPC
4. Handles OS-level operations (spawn/stop server, process discovery)
5. **This is the actual engine—all the real work happens here**

### What Tauri Does (Webview Host + OS Bridge)

1. Hosts the Blazor webview
2. Initializes and runs the gRPC server (from client-core)
3. Manages application state
4. **Thin orchestrator, not business logic**

### The Key Difference from egui

- **egui + Tauri IPC:** Blazor sends JSON strings via Tauri commands, loses type safety
- **Blazor + gRPC:** Blazor sends/receives protobuf messages, type-safe end-to-end
- Same ServerInfo proto is shared. Same pattern applies to all messages.

### Step 1: Define Message Protocol in Protobuf

**Goal:** Define all chat messages and session types in protobuf, generate code for both Rust and C#

**Pattern:** Same approach as `server.proto` (SessionInfo already proves this works)

**Tasks:**

1. Extend `clients/tauri-blazor/proto/server.proto` with chat message definitions.

   Add to the existing `server.proto` file:

   ```protobuf
   // Session information
   message SessionInfo {
     string id = 1;
     string title = 2;
     int64 created_at = 3;
     int64 updated_at = 4;
   }

   // A single message in a conversation
   message Message {
     string id = 1;
     string session_id = 2;
     string role = 3;        // "user" or "assistant"
     string content = 4;     // Markdown-formatted text
     int64 created_at = 5;
     string error = 6;       // Optional: error message if failed
   }

   // Request to create a session
   message CreateSessionRequest {
     string title = 1;
   }

   // Request to send a message
   message SendMessageRequest {
     string session_id = 1;
     string content = 2;
   }

   // Server event (streamed responses)
   message ServerEvent {
     oneof event {
       TextChunk text_chunk = 1;
       MessageComplete message_complete = 2;
       Error error = 3;
     }
   }

   message TextChunk {
     string session_id = 1;
     string content = 2;
   }

   message MessageComplete {
     string session_id = 1;
     string message_id = 2;
   }

   message Error {
     string code = 1;
     string message = 2;
   }
   ```

   **Why:** This is the single source of truth. Both Rust and C# will generate code from it, ensuring type safety.

2. **Build step:** The build system will:
   - Rust: Generate via `prost` in `build.rs` (like ServerInfo)
   - C#: Generate via protobuf compiler (like existing ServerInfo.cs)
   - No manual C# models needed; they're generated

3. **Result:** After build, both sides have identical types for all messages. No JSON parsing, no string interpolation.

**Technical Details:**

- **Pattern:** Same as ServerInfo (Session 3 proved this works)
- **Build system:** Csproj and Cargo.toml already handle proto generation
- **Generated code:** Don't hand-edit, regenerate from proto changes
- **Validation:** Rust side adds validation wrappers (like ServerInfoBuilder), C# uses generated types directly

---

### Step 2: Implement gRPC Service in Client Core

**Goal:** Create Rust gRPC service in client-core that handles all chat operations

**Key:** The gRPC service is the main application layer. It handles sessions, messages, and OpenCode server communication. Tauri just hosts it.

**Tasks:**

1. Create `clients/tauri-blazor/backend/client-core/src/chat/mod.rs`:

   Implement the gRPC service (following the pattern of discovery module):

   ```rust
   // This service will be implemented as a gRPC service
   // Methods:
   // - create_session(CreateSessionRequest) -> CreateSessionResponse
   // - list_sessions() -> ListSessionsResponse
   // - get_messages(GetMessagesRequest) -> GetMessagesResponse
   // - delete_session(DeleteSessionRequest) -> Empty
   // - send_message(SendMessageRequest) -> Empty
   // - stream_events() -> stream ServerEvent
   ```

   **Implementation approach:**
   - Use `tonic` crate (Rust gRPC framework)
   - Implement handlers that:
     - Validate requests
     - Call OpenCode server via HTTP REST API
     - Stream responses back to Blazor
   - Reuse existing HTTP client from egui if available

2. Export from client-core:

   Update `clients/tauri-blazor/backend/client-core/src/lib.rs` to expose chat service.

3. Update Tauri app startup:

   Tauri app initializes the gRPC server from client-core on startup.

**Technical Details:**

- **gRPC over what?** Likely gRPC over HTTP/2 on localhost (TLS optional for local dev)
- **Library:** `tonic` + `tonic-build` for Rust
- **Protobuf:** Already building for C#, Rust will use same protos
- **Blazor client:** Will use `Grpc.Net.Client` to call the service

---

### Step 3: Build Chat UI & Implement Markdown Rendering

**Goal:** Create Razor components for chat interface + markdown support

**Tasks:**

1. Create `frontend/Services/ChatService.cs` (gRPC client wrapper):

   ```csharp
   namespace OpenCode.Services;

   using Grpc.Net.Client;
   using Opencode.Server;

   /// <summary>
   /// Service for communicating with Tauri gRPC backend.
   /// </summary>
   public class ChatService : IChatService
   {
       private readonly GrpcChannel _channel;

       public ChatService()
       {
           // Connect to local gRPC server (started by Tauri)
           _channel = GrpcChannel.ForAddress("http://localhost:50051");
       }

       public async Task<SessionInfo> CreateSessionAsync(string title)
       {
           // Call gRPC method
           // var client = new TauriService.TauriServiceClient(_channel);
           // var response = await client.CreateSessionAsync(new CreateSessionRequest { Title = title });
           // return response.Session;
           throw new NotImplementedException();
       }

       public async Task<List<SessionInfo>> GetSessionsAsync()
       {
           // Call gRPC method
           throw new NotImplementedException();
       }

       // ... other methods
   }
   ```

   **Key:** This is a thin wrapper around gRPC client, no JSON parsing.

2. Create `frontend/Pages/Chat.razor`:

   ```razor
   @page "/chat"
   @using OpenCode.Services
   @using Opencode.Models
   @using Radzen
   @inject ISessionService SessionService
   @inject NotificationService NotificationService

   <PageTitle>OpenCode - Chat</PageTitle>

   <RadzenStack Orientation="Orientation.Horizontal" Gap="1rem" Class="chat-container">
       <!-- Session sidebar -->
       <RadzenCard Class="session-sidebar">
           <RadzenStack Gap="1rem">
               <RadzenButton Text="+ New Session" Click="OnNewSessionAsync" />

               @foreach (var session in sessions)
               {
                   <RadzenButton Text="@session.Title"
                                 Click="@((args) => OnSelectSessionAsync(session))"
                                 Class="@(selectedSession?.Id == session.Id ? "selected" : "")" />
               }
           </RadzenStack>
       </RadzenCard>

       <!-- Chat area -->
       <RadzenStack Class="chat-area" Gap="0">
           @if (selectedSession != null)
           {
               <!-- Message list -->
               <div class="message-list">
                   @foreach (var message in messages)
                   {
                       <div class="message @message.Role">
                           <RadzenText TextStyle="TextStyle.Body2">
                               @if (message.Role == "assistant")
                               {
                                   <!-- Render markdown -->
                                   @((MarkupString)MarkdownToHtml(message.Content))
                               }
                               else
                               {
                                   @message.Content
                               }
                           </RadzenText>
                       </div>
                   }
               </div>

               <!-- Input area -->
               <RadzenCard Class="input-area">
                   <RadzenStack Orientation="Orientation.Horizontal" Gap="0.5rem">
                       <RadzenTextBox @bind-Value="userInput"
                                      Placeholder="Type your message..."
                                      Class="message-input" />
                       <RadzenButton Text="Send"
                                     Click="OnSendMessageAsync"
                                     IsBusy="isStreaming"
                                     Disabled="@(string.IsNullOrWhiteSpace(userInput) || isStreaming)" />
                   </RadzenStack>
               </RadzenCard>
           }
           else
           {
               <RadzenAlert AlertStyle="AlertStyle.Info">
                   Select a session or create a new one to start chatting
               </RadzenAlert>
           }
       </RadzenStack>
   </RadzenStack>

   @code {
       private List<SessionInfo> sessions = new();
       private SessionInfo? selectedSession;
       private List<Message> messages = new();
       private string userInput = "";
       private bool isStreaming;

       protected override async Task OnInitializedAsync()
       {
           await LoadSessionsAsync();
       }

       private async Task LoadSessionsAsync()
       {
           try
           {
               sessions = await SessionService.GetSessionsAsync();
               if (sessions.Count > 0)
                   await OnSelectSessionAsync(sessions[0]);
           }
           catch (Exception ex)
           {
               NotificationService.Notify(NotificationSeverity.Error, "Load sessions failed", ex.Message);
           }
       }

       private async Task OnNewSessionAsync()
       {
           try
           {
               var newSession = await SessionService.CreateSessionAsync();
               sessions.Add(newSession);
               await OnSelectSessionAsync(newSession);
           }
           catch (Exception ex)
           {
               NotificationService.Notify(NotificationSeverity.Error, "Create session failed", ex.Message);
           }
       }

       private async Task OnSelectSessionAsync(SessionInfo session)
       {
           selectedSession = session;
           try
           {
               messages = await SessionService.GetMessagesAsync(session.Id);
           }
           catch (Exception ex)
           {
               NotificationService.Notify(NotificationSeverity.Error, "Load messages failed", ex.Message);
           }
       }

       private async Task OnSendMessageAsync()
       {
           if (selectedSession == null || string.IsNullOrWhiteSpace(userInput))
               return;

           var userMsg = userInput;
           userInput = "";
           isStreaming = true;

           try
           {
               // Add user message to UI
               messages.Add(new Message(
                   Guid.NewGuid().ToString(),
                   selectedSession.Id,
                   "user",
                   userMsg,
                   DateTime.UtcNow
               ));

               // Stream assistant response
               var assistantMsg = new Message(
                   Guid.NewGuid().ToString(),
                   selectedSession.Id,
                   "assistant",
                   "",
                   DateTime.UtcNow
               );
               messages.Add(assistantMsg);

               await foreach (var chunk in SessionService.SendMessageAndStreamAsync(selectedSession.Id, userMsg))
               {
                   // Update last message with chunk
                   var lastIdx = messages.Count - 1;
                   if (lastIdx >= 0 && messages[lastIdx].Role == "assistant")
                   {
                       messages[lastIdx] = messages[lastIdx] with { Content = messages[lastIdx].Content + chunk };
                       StateHasChanged();
                   }
               }
           }
           catch (Exception ex)
           {
               NotificationService.Notify(NotificationSeverity.Error, "Send message failed", ex.Message);
           }
           finally
           {
               isStreaming = false;
           }
       }

       private string MarkdownToHtml(string markdown)
       {
           // Use Markdig to convert markdown to HTML
           // Implementation in next section
           return markdown; // Placeholder
       }
   }
   ```

3. Add CSS to `frontend/wwwroot/css/app.css`:

   ```css
   .chat-container {
     height: 100vh;
     background-color: var(--bg-color);
   }

   .session-sidebar {
     width: 250px;
     overflow-y: auto;
     background-color: var(--sidebar-bg);
     padding: 1rem;
   }

   .chat-area {
     flex: 1;
     display: flex;
     flex-direction: column;
     height: 100vh;
   }

   .message-list {
     flex: 1;
     overflow-y: auto;
     padding: 1rem;
     display: flex;
     flex-direction: column;
     gap: 1rem;
   }

   .message {
     padding: 0.75rem 1rem;
     border-radius: 0.5rem;
     max-width: 80%;
   }

   .message.user {
     background-color: var(--user-msg-bg);
     color: var(--user-msg-color);
     align-self: flex-end;
   }

   .message.assistant {
     background-color: var(--assistant-msg-bg);
     color: var(--assistant-msg-color);
     align-self: flex-start;
   }

   .input-area {
     padding: 1rem;
     border-top: 1px solid var(--border-color);
     background-color: var(--input-area-bg);
   }

   .message-input {
     flex: 1;
     padding: 0.5rem;
   }
   ```

**Technical Details:**

- **Markdown rendering:** Markdig already in dependencies (from Session 3)
- **Streaming UI:** Use `StateHasChanged()` to trigger UI updates for each chunk
- **Message records:** Use `with` operator for immutable updates
- **Async enumerable:** C# 8+ feature for streaming chunks

---

4. Implement markdown rendering via Markdig:

   Create `frontend/Services/MarkdownService.cs`:

   ```csharp
   namespace OpenCode.Services;

   using Markdig;

   /// <summary>
   /// Service for converting markdown to HTML.
   /// </summary>
   public static class MarkdownService
   {
       private static readonly MarkdownPipeline Pipeline = new MarkdownPipelineBuilder()
           .UseAdvancedExtensions()  // Tables, footnotes, etc.
           .Build();

       /// <summary>
       /// Converts markdown string to HTML.
       /// </summary>
       public static string ToHtml(string markdown)
       {
           if (string.IsNullOrWhiteSpace(markdown))
               return "";

           return Markdig.Markdown.ToHtml(markdown, Pipeline);
       }
   }
   ```

5. Update Chat.razor to use MarkdownService:

   ```razor
   private string MarkdownToHtml(string markdown)
   {
       return MarkdownService.ToHtml(markdown);
   }
   ```

6. Add CSS for markdown elements in `app.css`:

   ```css
   .message.assistant h1,
   .message.assistant h2,
   .message.assistant h3 {
     margin-top: 1rem;
     margin-bottom: 0.5rem;
     font-weight: 600;
   }

   .message.assistant code {
     background-color: var(--code-bg);
     padding: 0.2rem 0.4rem;
     border-radius: 0.25rem;
     font-family: monospace;
   }

   .message.assistant pre {
     background-color: var(--code-bg);
     padding: 1rem;
     border-radius: 0.5rem;
     overflow-x: auto;
   }

   .message.assistant pre code {
     background-color: transparent;
     padding: 0;
   }

   .message.assistant ul,
   .message.assistant ol {
     margin-left: 1.5rem;
     margin-top: 0.5rem;
     margin-bottom: 0.5rem;
   }

   .message.assistant blockquote {
     border-left: 3px solid var(--quote-color);
     padding-left: 1rem;
     margin-left: 0;
     color: var(--quote-color);
   }
   ```

**Technical Details:**

- **Pipeline configuration:** `.UseAdvancedExtensions()` enables tables, footnotes, math, etc.
- **MarkupString:** Use `@((MarkupString)html)` in Razor to render HTML safely
- **Styling:** Add CSS custom properties for theme colors (dark/light mode in future session)

---

### Step 4: Implement gRPC Streaming for Real-Time Message Responses

**Goal:** Stream message responses from Tauri backend to Blazor UI in real-time

**How it works:**

1. **Blazor sends message** via gRPC `SendMessageRequest` to Tauri backend
2. **Tauri backend:**
   - Validates request
   - Calls OpenCode server HTTP REST API: `POST /session/{id}/message`
   - Listens to OpenCode server's SSE stream (`GET /global/event`)
   - Accumulates response chunks
   - Streams chunks back to Blazor via gRPC `stream ServerEvent`
3. **Blazor receives streaming chunks** via gRPC and updates UI in real-time

**Tasks:**

1. **In Tauri backend:** Implement the streaming handler

   The gRPC service will have:

   ```rust
   // Pseudo-code structure (actual implementation in Rust)
   async fn stream_events(&self) -> impl Stream<Item = ServerEvent> {
       // 1. Connect to OpenCode server
       // 2. Open SSE stream at /global/event
       // 3. For each chunk received:
       //    - Parse into protobuf ServerEvent
       //    - Yield to Blazor
   }
   ```

2. **In Blazor:** Implement streaming reception

   ```csharp
   public async IAsyncEnumerable<string> SendMessageAndStreamAsync(string sessionId, string userMessage)
   {
       // 1. Create gRPC client
       // var client = new TauriService.TauriServiceClient(_channel);

       // 2. Send message
       // await client.SendMessageAsync(new SendMessageRequest
       // {
       //     SessionId = sessionId,
       //     Content = userMessage
       // });

       // 3. Open streaming connection
       // using var stream = client.StreamEvents(new Empty());
       // await foreach (var @event in stream.ResponseStream.ReadAllAsync())
       // {
       //     if (@event.EventCase == ServerEvent.EventOneofCase.TextChunk)
       //         yield return @event.TextChunk.Content;
       // }
   }
   ```

   **Key:** `gRPC.Net.Client` handles the streaming protocol automatically. No custom JS, no polling—pure protobuf streaming.

**Technical Details:**

- **gRPC streaming:** Built into protobuf services via `stream` keyword
- **Blazor client:** Use `Grpc.Net.Client` (NuGet package)
- **Rust server:** Use `tonic` crate
- **Protobuf:** `service` definition includes `rpc stream_events() returns (stream ServerEvent)`
- **No SSE complexity:** gRPC handles multiplexing and streaming transparently

---

## Success Criteria for Session 4

- [ ] Protobuf message definitions added to `server.proto` (SessionInfo, Message, ServerEvent, etc.)
- [ ] Rust and C# code generated from proto (no manual models needed)
- [ ] gRPC service implemented in Tauri backend (at least stubbed)
- [ ] Chat.razor displays session list and selected session
- [ ] Can create new sessions via gRPC
- [ ] Can select sessions and view message history via gRPC
- [ ] Message input box and send button working
- [ ] User messages appear in chat immediately
- [ ] Assistant responses stream via gRPC (protobuf ServerEvent)
- [ ] Markdown renders correctly (headers, code, lists, blockquotes)
- [ ] Radzen components consistent with Home.razor
- [ ] Error notifications for failed operations
- [ ] Loading states while streaming
- [ ] NO JavaScript—all C# ↔ Rust via gRPC/Protobuf

---

## Key Files to Reference

**Existing (Read these first):**

- `clients/tauri-blazor/proto/server.proto` - Protobuf schema template (ServerInfo proves the pattern)
- `clients/tauri-blazor/models/src/server_info/` - Rust: How proto generates code + validation builder
- `clients/tauri-blazor/frontend/desktop/opencode/obj/Release/net10.0/Server.cs` - C#: Generated protobuf code
- `frontend/Services/ServerService.cs` - gRPC client pattern (similar to how you'll wrap gRPC)
- `frontend/Pages/Home.razor` - UI pattern with Radzen components

**To Create:**

- `clients/tauri-blazor/proto/server.proto` - **Extend with:** SessionInfo, Message, ServerEvent, CreateSessionRequest, SendMessageRequest
- `frontend/Services/ChatService.cs` - gRPC client wrapper (uses `Grpc.Net.Client`)
- `frontend/Pages/Chat.razor` - Chat UI page (similar to Home.razor)
- `frontend/Services/MarkdownService.cs` - Markdown rendering via Markdig

**Rust (To Create/Modify):**

- `clients/tauri-blazor/backend/client-core/src/chat/mod.rs` - gRPC service implementation (uses `tonic`)
- `clients/tauri-blazor/backend/client-core/src/lib.rs` - Export chat service
- `clients/tauri-blazor/backend/client-core/Cargo.toml` - Add `tonic`, `tonic-build`, `tokio` dependencies
- `apps/desktop/opencode/src/main.rs` - Initialize gRPC server from client-core on startup

---

## Important Reminders

1. **Zero custom JavaScript** - All Tauri IPC via C# IJSRuntime only
2. **Streaming is complex** - Consider starting with polling backend, refactor to events later
3. **Markdown safety** - `@((MarkupString)...)` in Razor renders HTML (verify no XSS)
4. **Error handling** - Create custom exception types for session/message failures
5. **State management** - Keep messages in component state, update via streaming chunks
6. **Performance** - Use `ConfigureAwait(false)` in all async calls
7. **Testing** - Test with multiple sessions and long messages before moving on

---

## Technical Constraints

- **.NET Version:** 10.0+
- **IPC Mechanism:** gRPC + Protobuf (ALL C# ↔ Rust communication)
- **gRPC Library:** `Grpc.Net.Client` for Blazor, `tonic` for Rust
- **Message Definitions:** Extend `proto/server.proto` (single source of truth)
- **Code Generation:** Both Rust and C# auto-generate from protobuf
- **Markdown:** Markdig (already in dependencies)
- **Zero Custom JS:** No JavaScript for IPC (gRPC is binary protocol)
- **OpenCode Server API:** Uses HTTP REST from Tauri backend only
  - Sessions: `POST /session`, `GET /session`, `DELETE /session/{id}`
  - Messages: `POST /session/{id}/message`
  - Events: `GET /global/event` (SSE stream, handled by Tauri internally)
- **Tauri Role:** OS bridge only (process spawning, file access) + gRPC server host

---

## Estimated Token Budget

**~120K tokens:**

- Reading context: ~20K tokens (existing code patterns)
- Service layer: ~20K tokens (ISessionService + SessionService)
- UI components: ~30K tokens (Chat.razor + CSS)
- Markdown integration: ~15K tokens (MarkdownService + styling)
- Tauri event streaming: ~20K tokens (backend + frontend)
- Testing/verification: ~15K tokens

---

## Known Challenges & Solutions

**Challenge 1:** gRPC setup complexity

**Solution:** Start with message definitions in proto (Step 1), then implement gRPC service in Rust (Step 2). Use `tonic` crate; it's the standard Rust gRPC framework. Blazor side uses `Grpc.Net.Client` (standard NuGet).

**Challenge 2:** gRPC streaming coordination

**Solution:** Protobuf `stream` keyword handles all coordination. Rust side iterates SSE chunks internally, sends via gRPC stream. Blazor iterates gRPC stream with `await foreach`. Simple and type-safe.

**Challenge 3:** XSS with markdown HTML

**Solution:** Use `MarkupString` safely (Markdig output is already sanitized). Consider additional HTML sanitization if needed.

**Challenge 4:** OpenCode server is HTTP REST, not gRPC

**Solution:** That's correct. Tauri backend calls OpenCode server via HTTP REST (proven in egui). Tauri then adapts those responses to gRPC for Blazor. Separation of concerns: Tauri is the translator.

**Challenge 5:** gRPC over what transport?

**Solution:** gRPC over HTTP/2 on localhost. `tonic` and `Grpc.Net.Client` handle negotiation automatically. No TLS needed for local dev (can be added later).

---

**Start with:** "Let me review the existing ServerService pattern and create SessionInfo/Message models, then build the ISessionService interface matching that pattern."
