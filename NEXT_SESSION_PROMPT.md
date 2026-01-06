# Next Session: Session 4A - WebSocket + Proto Foundation

**Date:** 2026-01-05  
**Goal:** Implement WebSocket server + core protobuf messages  
**Token Budget:** ~80K

---

## Architecture Decision

**IPC:** WebSocket + Protobuf (see [ADR-0003](docs/adr/0003-websocket-protobuf-ipc.md))

**NOT gRPC.** We investigated gRPC, ZeroMQ, nng, Tauri invoke - all failed one of:
- Streaming (Tauri invoke)
- Browser sandbox (ZeroMQ, nng, raw sockets)
- Simplicity (gRPC HTTP/2 overhead)

WebSocket + Protobuf is the narrow waist that survives all constraints.

---

## Architecture

```
Blazor WASM (C#)
    ↓ System.Net.WebSockets.ClientWebSocket
    ↓ Binary protobuf frames
WebSocket (ws://127.0.0.1:PORT)
    ↓ tokio-tungstenite + prost
client-core (Rust)
    ↓ reqwest HTTP
OpenCode Server
```

**Key principles:**
- Blazor is "dumb glass" - renders tokens, never interprets logic
- One WebSocket = one session (open on start, close on exit)
- Binary-only protocol (no JSON, no text frames)
- All logic in client-core (Thin Tauri Layer - ADR-0002)

---

## Session 4A Scope

**Build:**
1. WebSocket server in client-core (Rust)
2. Protobuf message envelope (`ClientMessage` / `ServerMessage`)
3. Core request/response handlers (sessions, agents, providers, auth)
4. C# WebSocket client service
5. Smoke test page

**NOT building (Session 4B):**
- Streaming (ChatToken, events)
- SSE → WebSocket bridge
- Tool call messages

---

## Implementation Steps

### Step 1: Create Proto Files

**`proto/messages.proto`** - Single file with envelope + all messages

```protobuf
syntax = "proto3";
package opencode;

// ============================================
// ENVELOPE (wraps all messages)
// ============================================

message ClientMessage {
  uint64 request_id = 1;
  
  oneof payload {
    // Auth
    AuthHandshake auth_handshake = 10;
    
    // Sessions
    ListSessionsRequest list_sessions = 20;
    CreateSessionRequest create_session = 21;
    DeleteSessionRequest delete_session = 22;
    
    // Agents
    ListAgentsRequest list_agents = 30;
    
    // Providers
    GetProviderStatusRequest get_provider_status = 40;
    
    // Auth operations
    SetAuthRequest set_auth = 50;
    GetAuthRequest get_auth = 51;
  }
}

message ServerMessage {
  uint64 request_id = 1;
  
  oneof payload {
    // Auth
    AuthHandshakeResponse auth_handshake_response = 10;
    
    // Sessions
    SessionList session_list = 20;
    SessionInfo session_info = 21;
    
    // Agents
    AgentList agent_list = 30;
    
    // Providers
    ProviderStatus provider_status = 40;
    
    // Auth
    AuthInfo auth_info = 50;
    
    // Errors
    ErrorResponse error = 100;
    
    // Streaming (Session 4B)
    // ChatToken token = 200;
    // ChatCompleted completed = 201;
    // ToolCallEvent tool_call = 202;
  }
}

// ============================================
// AUTH HANDSHAKE
// ============================================

message AuthHandshake {
  string token = 1;
}

message AuthHandshakeResponse {
  bool success = 1;
  optional string error = 2;
}

// ============================================
// SESSIONS
// ============================================

message ListSessionsRequest {}

message CreateSessionRequest {
  optional string title = 1;
}

message DeleteSessionRequest {
  string session_id = 1;
}

message SessionList {
  repeated SessionInfo sessions = 1;
}

message SessionInfo {
  string id = 1;
  optional string title = 2;
  int64 created_at = 3;
  int64 updated_at = 4;
}

// ============================================
// AGENTS
// ============================================

message ListAgentsRequest {}

message AgentList {
  repeated AgentInfo agents = 1;
}

message AgentInfo {
  string name = 1;
  optional string description = 2;
  optional string mode = 3;
  bool built_in = 4;
  optional string color = 5;
}

// ============================================
// PROVIDERS
// ============================================

message GetProviderStatusRequest {}

message ProviderStatus {
  repeated string connected = 1;
}

// ============================================
// AUTH OPERATIONS
// ============================================

message SetAuthRequest {
  string provider_id = 1;
  AuthInfo auth = 2;
}

message GetAuthRequest {
  string provider_id = 1;
}

message AuthInfo {
  oneof auth_type {
    ApiKeyAuth api_key = 1;
    OAuthAuth oauth = 2;
  }
}

message ApiKeyAuth {
  string key = 1;
}

message OAuthAuth {
  string access_token = 1;
  string refresh_token = 2;
  int64 expires_at = 3;
}

// ============================================
// ERRORS
// ============================================

message ErrorResponse {
  string code = 1;
  string message = 2;
}
```

### Step 2: Configure Rust Build

**`backend/client-core/Cargo.toml`** - Add dependencies

```toml
[dependencies]
tokio = { version = "1", features = ["full"] }
tokio-tungstenite = "0.21"
futures-util = "0.3"
prost = "0.12"
reqwest = { version = "0.11", features = ["json"] }
serde_json = "1"
uuid = { version = "1", features = ["v4"] }
log = "0.4"

[build-dependencies]
prost-build = "0.12"
```

**`backend/client-core/build.rs`**

```rust
fn main() -> Result<(), Box<dyn std::error::Error>> {
    prost_build::compile_protos(
        &["../../proto/messages.proto"],
        &["../../proto/"],
    )?;
    Ok(())
}
```

### Step 3: Implement WebSocket Server (client-core)

**`backend/client-core/src/ws/mod.rs`**

```rust
mod server;
mod handlers;

pub use server::start_ws_server;
```

**`backend/client-core/src/ws/server.rs`**

```rust
use std::net::SocketAddr;
use std::sync::Arc;
use tokio::net::TcpListener;
use tokio_tungstenite::accept_async;
use futures_util::{StreamExt, SinkExt};
use prost::Message;
use log::{info, warn, error};

use crate::proto::{ClientMessage, ServerMessage, client_message, server_message};
use super::handlers::handle_request;

pub struct WebSocketServer {
    opencode_base_url: String,
    auth_token: String,
    http_client: reqwest::Client,
}

impl WebSocketServer {
    pub fn new(opencode_base_url: String, auth_token: String) -> Self {
        Self {
            opencode_base_url,
            auth_token,
            http_client: reqwest::Client::new(),
        }
    }
}

pub async fn start_ws_server(
    port: u16,
    opencode_base_url: String,
    auth_token: String,
) -> Result<(), Box<dyn std::error::Error>> {
    // SECURITY: Bind only to localhost
    let addr: SocketAddr = format!("127.0.0.1:{}", port).parse()?;
    let listener = TcpListener::bind(addr).await?;
    
    info!("WebSocket server listening on ws://{}", addr);
    
    let server = Arc::new(WebSocketServer::new(opencode_base_url, auth_token));
    
    while let Ok((stream, peer_addr)) = listener.accept().await {
        // SECURITY: Reject non-local connections
        if !peer_addr.ip().is_loopback() {
            warn!("Rejected non-local connection from {}", peer_addr);
            continue;
        }
        
        let server = Arc::clone(&server);
        tokio::spawn(async move {
            match accept_async(stream).await {
                Ok(ws_stream) => {
                    if let Err(e) = handle_connection(ws_stream, server).await {
                        error!("Connection error: {}", e);
                    }
                }
                Err(e) => error!("WebSocket handshake failed: {}", e),
            }
        });
    }
    
    Ok(())
}

async fn handle_connection(
    ws_stream: tokio_tungstenite::WebSocketStream<tokio::net::TcpStream>,
    server: Arc<WebSocketServer>,
) -> Result<(), Box<dyn std::error::Error>> {
    let (mut write, mut read) = ws_stream.split();
    
    // SECURITY: First message must be auth handshake
    let first_msg = read.next().await
        .ok_or("Connection closed before handshake")??;
    
    if let tokio_tungstenite::tungstenite::Message::Binary(data) = first_msg {
        let client_msg = ClientMessage::decode(&data[..])?;
        
        match client_msg.payload {
            Some(client_message::Payload::AuthHandshake(handshake)) => {
                let success = handshake.token == server.auth_token;
                let response = ServerMessage {
                    request_id: client_msg.request_id,
                    payload: Some(server_message::Payload::AuthHandshakeResponse(
                        crate::proto::AuthHandshakeResponse {
                            success,
                            error: if success { None } else { Some("Invalid token".into()) },
                        }
                    )),
                };
                write.send(tokio_tungstenite::tungstenite::Message::Binary(
                    response.encode_to_vec()
                )).await?;
                
                if !success {
                    return Err("Auth failed".into());
                }
            }
            _ => return Err("First message must be auth handshake".into()),
        }
    } else {
        return Err("Expected binary message".into());
    }
    
    // Main message loop
    while let Some(msg) = read.next().await {
        let msg = msg?;
        
        if let tokio_tungstenite::tungstenite::Message::Binary(data) = msg {
            let client_msg = ClientMessage::decode(&data[..])?;
            let response = handle_request(&server, client_msg).await;
            write.send(tokio_tungstenite::tungstenite::Message::Binary(
                response.encode_to_vec()
            )).await?;
        }
    }
    
    Ok(())
}
```

**`backend/client-core/src/ws/handlers.rs`**

```rust
use crate::proto::*;
use super::server::WebSocketServer;

pub async fn handle_request(
    server: &WebSocketServer,
    msg: ClientMessage,
) -> ServerMessage {
    let request_id = msg.request_id;
    
    let payload = match msg.payload {
        Some(client_message::Payload::ListSessions(_)) => {
            handle_list_sessions(server).await
        }
        Some(client_message::Payload::CreateSession(req)) => {
            handle_create_session(server, req).await
        }
        Some(client_message::Payload::DeleteSession(req)) => {
            handle_delete_session(server, req).await
        }
        Some(client_message::Payload::ListAgents(_)) => {
            handle_list_agents(server).await
        }
        Some(client_message::Payload::GetProviderStatus(_)) => {
            handle_get_provider_status(server).await
        }
        Some(client_message::Payload::SetAuth(req)) => {
            handle_set_auth(server, req).await
        }
        Some(client_message::Payload::GetAuth(req)) => {
            handle_get_auth(server, req).await
        }
        _ => {
            Some(server_message::Payload::Error(ErrorResponse {
                code: "UNKNOWN_REQUEST".into(),
                message: "Unknown request type".into(),
            }))
        }
    };
    
    ServerMessage { request_id, payload }
}

async fn handle_list_sessions(server: &WebSocketServer) -> Option<server_message::Payload> {
    let url = format!("{}/session", server.opencode_base_url);
    
    match server.http_client.get(&url).send().await {
        Ok(resp) => {
            match resp.json::<Vec<serde_json::Value>>().await {
                Ok(json) => {
                    let sessions = json.iter().map(|s| SessionInfo {
                        id: s["id"].as_str().unwrap_or("").to_string(),
                        title: s["title"].as_str().map(String::from),
                        created_at: s["createdAt"].as_i64().unwrap_or(0),
                        updated_at: s["updatedAt"].as_i64().unwrap_or(0),
                    }).collect();
                    
                    Some(server_message::Payload::SessionList(SessionList { sessions }))
                }
                Err(e) => Some(server_message::Payload::Error(ErrorResponse {
                    code: "PARSE_ERROR".into(),
                    message: e.to_string(),
                })),
            }
        }
        Err(e) => Some(server_message::Payload::Error(ErrorResponse {
            code: "HTTP_ERROR".into(),
            message: e.to_string(),
        })),
    }
}

async fn handle_create_session(
    server: &WebSocketServer,
    req: CreateSessionRequest,
) -> Option<server_message::Payload> {
    let url = format!("{}/session", server.opencode_base_url);
    
    let body = serde_json::json!({
        "title": req.title.unwrap_or_default()
    });
    
    match server.http_client.post(&url).json(&body).send().await {
        Ok(resp) => {
            match resp.json::<serde_json::Value>().await {
                Ok(s) => {
                    Some(server_message::Payload::SessionInfo(SessionInfo {
                        id: s["id"].as_str().unwrap_or("").to_string(),
                        title: s["title"].as_str().map(String::from),
                        created_at: s["createdAt"].as_i64().unwrap_or(0),
                        updated_at: s["updatedAt"].as_i64().unwrap_or(0),
                    }))
                }
                Err(e) => Some(server_message::Payload::Error(ErrorResponse {
                    code: "PARSE_ERROR".into(),
                    message: e.to_string(),
                })),
            }
        }
        Err(e) => Some(server_message::Payload::Error(ErrorResponse {
            code: "HTTP_ERROR".into(),
            message: e.to_string(),
        })),
    }
}

async fn handle_delete_session(
    server: &WebSocketServer,
    req: DeleteSessionRequest,
) -> Option<server_message::Payload> {
    let url = format!("{}/session/{}", server.opencode_base_url, req.session_id);
    
    match server.http_client.delete(&url).send().await {
        Ok(_) => None, // Success, no payload needed
        Err(e) => Some(server_message::Payload::Error(ErrorResponse {
            code: "HTTP_ERROR".into(),
            message: e.to_string(),
        })),
    }
}

async fn handle_list_agents(server: &WebSocketServer) -> Option<server_message::Payload> {
    let url = format!("{}/agent", server.opencode_base_url);
    
    match server.http_client.get(&url).send().await {
        Ok(resp) => {
            match resp.json::<Vec<serde_json::Value>>().await {
                Ok(json) => {
                    let agents = json.iter().map(|a| AgentInfo {
                        name: a["name"].as_str().unwrap_or("").to_string(),
                        description: a["description"].as_str().map(String::from),
                        mode: a["mode"].as_str().map(String::from),
                        built_in: a["builtIn"].as_bool().unwrap_or(false),
                        color: a["color"].as_str().map(String::from),
                    }).collect();
                    
                    Some(server_message::Payload::AgentList(AgentList { agents }))
                }
                Err(e) => Some(server_message::Payload::Error(ErrorResponse {
                    code: "PARSE_ERROR".into(),
                    message: e.to_string(),
                })),
            }
        }
        Err(e) => Some(server_message::Payload::Error(ErrorResponse {
            code: "HTTP_ERROR".into(),
            message: e.to_string(),
        })),
    }
}

async fn handle_get_provider_status(server: &WebSocketServer) -> Option<server_message::Payload> {
    let url = format!("{}/provider", server.opencode_base_url);
    
    match server.http_client.get(&url).send().await {
        Ok(resp) => {
            match resp.json::<serde_json::Value>().await {
                Ok(json) => {
                    let connected = json["connected"]
                        .as_array()
                        .map(|arr| arr.iter()
                            .filter_map(|v| v.as_str())
                            .map(String::from)
                            .collect())
                        .unwrap_or_default();
                    
                    Some(server_message::Payload::ProviderStatus(ProviderStatus { connected }))
                }
                Err(e) => Some(server_message::Payload::Error(ErrorResponse {
                    code: "PARSE_ERROR".into(),
                    message: e.to_string(),
                })),
            }
        }
        Err(e) => Some(server_message::Payload::Error(ErrorResponse {
            code: "HTTP_ERROR".into(),
            message: e.to_string(),
        })),
    }
}

async fn handle_set_auth(
    server: &WebSocketServer,
    req: SetAuthRequest,
) -> Option<server_message::Payload> {
    let url = format!("{}/auth/{}", server.opencode_base_url, req.provider_id);
    
    let body = match req.auth.and_then(|a| a.auth_type) {
        Some(auth_info::AuthType::ApiKey(api_key)) => {
            serde_json::json!({ "type": "api", "key": api_key.key })
        }
        Some(auth_info::AuthType::Oauth(oauth)) => {
            serde_json::json!({
                "type": "oauth",
                "access": oauth.access_token,
                "refresh": oauth.refresh_token,
                "expires": oauth.expires_at
            })
        }
        None => {
            return Some(server_message::Payload::Error(ErrorResponse {
                code: "INVALID_REQUEST".into(),
                message: "Missing auth info".into(),
            }));
        }
    };
    
    match server.http_client.put(&url).json(&body).send().await {
        Ok(_) => None, // Success
        Err(e) => Some(server_message::Payload::Error(ErrorResponse {
            code: "HTTP_ERROR".into(),
            message: e.to_string(),
        })),
    }
}

async fn handle_get_auth(
    server: &WebSocketServer,
    req: GetAuthRequest,
) -> Option<server_message::Payload> {
    // Auth is stored locally, not fetched from server
    // This would need to read from local auth.json
    Some(server_message::Payload::Error(ErrorResponse {
        code: "NOT_IMPLEMENTED".into(),
        message: "GetAuth not yet implemented".into(),
    }))
}
```

### Step 4: Export from client-core

**`backend/client-core/src/lib.rs`**

```rust
pub mod discovery;
pub mod error;
pub mod ws;

mod proto {
    include!(concat!(env!("OUT_DIR"), "/opencode.rs"));
}

pub use ws::start_ws_server;
```

### Step 5: Wire into Tauri

**`apps/desktop/opencode/src/main.rs`**

```rust
use client_core::ws::start_ws_server;
use uuid::Uuid;

fn main() {
    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![
            commands::server::discover_server,
            commands::server::spawn_server,
            commands::server::check_health,
            commands::server::stop_server,
            commands::ws::get_ws_info,  // NEW
        ])
        .setup(|app| {
            // ... existing setup ...
            
            // Generate auth token
            let auth_token = Uuid::new_v4().to_string();
            
            // Find available port
            let ws_port = find_available_port(50051..50061);
            
            // Store for Blazor to query
            app.manage(WsInfo { port: ws_port, auth_token: auth_token.clone() });
            
            // Start WebSocket server (after OpenCode server discovered/spawned)
            let opencode_url = "http://localhost:4008".to_string(); // TODO: from discovery
            tokio::spawn(async move {
                if let Err(e) = start_ws_server(ws_port, opencode_url, auth_token).await {
                    error!("WebSocket server failed: {}", e);
                }
            });
            
            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
```

**`apps/desktop/opencode/src/commands/ws.rs`**

```rust
use tauri::State;

#[derive(Clone)]
pub struct WsInfo {
    pub port: u16,
    pub auth_token: String,
}

#[tauri::command]
pub fn get_ws_info(ws_info: State<WsInfo>) -> (u16, String) {
    (ws_info.port, ws_info.auth_token.clone())
}
```

### Step 6: C# WebSocket Client

**`frontend/desktop/opencode/Services/WebSocketService.cs`**

```csharp
using System.Net.WebSockets;
using Google.Protobuf;
using Opencode;

public class WebSocketService : IAsyncDisposable
{
    private readonly ClientWebSocket _socket = new();
    private ulong _nextRequestId = 1;
    
    public event Action<ServerMessage>? OnMessageReceived;
    
    public async Task ConnectAsync(int port, string authToken)
    {
        var url = $"ws://127.0.0.1:{port}";
        await _socket.ConnectAsync(new Uri(url), CancellationToken.None);
        
        // Send auth handshake
        var handshake = new ClientMessage 
        { 
            RequestId = _nextRequestId++,
            AuthHandshake = new AuthHandshake { Token = authToken }
        };
        await SendRawAsync(handshake);
        
        // Wait for response
        var response = await ReceiveOneAsync();
        if (!response.AuthHandshakeResponse.Success)
        {
            throw new Exception($"Auth failed: {response.AuthHandshakeResponse.Error}");
        }
        
        // Start receive loop
        _ = ReceiveLoopAsync();
    }
    
    public async Task<ServerMessage> SendAsync(ClientMessage message)
    {
        message.RequestId = _nextRequestId++;
        await SendRawAsync(message);
        // For request/response, we'd need to correlate by request_id
        // For now, just fire and forget - responses come via OnMessageReceived
        return await Task.FromResult<ServerMessage>(null!);
    }
    
    private async Task SendRawAsync(ClientMessage message)
    {
        var bytes = message.ToByteArray();
        await _socket.SendAsync(bytes, WebSocketMessageType.Binary, true, CancellationToken.None);
    }
    
    private async Task<ServerMessage> ReceiveOneAsync()
    {
        var buffer = new byte[8192];
        var result = await _socket.ReceiveAsync(buffer, CancellationToken.None);
        return ServerMessage.Parser.ParseFrom(buffer, 0, result.Count);
    }
    
    private async Task ReceiveLoopAsync()
    {
        var buffer = new byte[8192];
        while (_socket.State == WebSocketState.Open)
        {
            try
            {
                var result = await _socket.ReceiveAsync(buffer, CancellationToken.None);
                if (result.MessageType == WebSocketMessageType.Binary)
                {
                    var msg = ServerMessage.Parser.ParseFrom(buffer, 0, result.Count);
                    OnMessageReceived?.Invoke(msg);
                }
            }
            catch (Exception ex)
            {
                Console.WriteLine($"WebSocket receive error: {ex.Message}");
                break;
            }
        }
    }
    
    public async ValueTask DisposeAsync()
    {
        if (_socket.State == WebSocketState.Open)
        {
            await _socket.CloseAsync(WebSocketCloseStatus.NormalClosure, null, CancellationToken.None);
        }
        _socket.Dispose();
    }
}
```

### Step 7: C# Protobuf Setup

**`frontend/desktop/opencode/Opencode.csproj`** - Add protobuf

```xml
<ItemGroup>
  <PackageReference Include="Google.Protobuf" Version="3.25.1" />
  <PackageReference Include="Grpc.Tools" Version="2.60.0" PrivateAssets="All" />
</ItemGroup>

<ItemGroup>
  <Protobuf Include="../../../proto/messages.proto" GrpcServices="None" />
</ItemGroup>
```

### Step 8: Smoke Test Page

**`frontend/desktop/opencode/Pages/WebSocketTest.razor`**

```razor
@page "/ws-test"
@using Opencode
@inject IJSRuntime JS

<h3>Session 4A WebSocket Test</h3>

<RadzenStack Orientation="Orientation.Vertical" Gap="1rem">
    <RadzenButton Text="Connect" Click="@Connect" Disabled="@_connected" />
    <RadzenButton Text="List Sessions" Click="@ListSessions" Disabled="@(!_connected)" />
    <RadzenButton Text="List Agents" Click="@ListAgents" Disabled="@(!_connected)" />
    <RadzenButton Text="Get Provider Status" Click="@GetProviderStatus" Disabled="@(!_connected)" />
    
    <RadzenCard>
        <pre>@_output</pre>
    </RadzenCard>
</RadzenStack>

@code {
    private WebSocketService? _ws;
    private bool _connected = false;
    private string _output = "";
    
    private async Task Connect()
    {
        try
        {
            // Get WS info from Tauri
            var result = await JS.InvokeAsync<JsonElement>(
                "__TAURI_INTERNALS__.invoke",
                "get_ws_info"
            );
            var port = result[0].GetInt32();
            var authToken = result[1].GetString()!;
            
            _ws = new WebSocketService();
            _ws.OnMessageReceived += OnMessage;
            await _ws.ConnectAsync(port, authToken);
            
            _connected = true;
            _output = "✅ Connected to WebSocket server\n";
        }
        catch (Exception ex)
        {
            _output = $"❌ Connection failed: {ex.Message}";
        }
    }
    
    private void OnMessage(ServerMessage msg)
    {
        InvokeAsync(() =>
        {
            _output += $"\n📨 Received: {msg.PayloadCase}\n";
            StateHasChanged();
        });
    }
    
    private async Task ListSessions()
    {
        var msg = new ClientMessage { ListSessions = new ListSessionsRequest() };
        await _ws!.SendAsync(msg);
        _output += "📤 Sent ListSessions request\n";
    }
    
    private async Task ListAgents()
    {
        var msg = new ClientMessage { ListAgents = new ListAgentsRequest() };
        await _ws!.SendAsync(msg);
        _output += "📤 Sent ListAgents request\n";
    }
    
    private async Task GetProviderStatus()
    {
        var msg = new ClientMessage { GetProviderStatus = new GetProviderStatusRequest() };
        await _ws!.SendAsync(msg);
        _output += "📤 Sent GetProviderStatus request\n";
    }
}
```

---

## Success Criteria

**Rust (client-core):**
- [ ] `proto/messages.proto` created
- [ ] `prost-build` generates Rust code
- [ ] WebSocket server starts on localhost
- [ ] Auth handshake works
- [ ] All 4 handlers implemented (sessions, agents, providers, auth)

**C# (Blazor):**
- [ ] Protobuf generates C# code
- [ ] `WebSocketService` connects and authenticates
- [ ] Can send/receive binary messages

**Integration:**
- [ ] Tauri starts WebSocket server on startup
- [ ] Blazor gets port/token via Tauri command
- [ ] Smoke test page shows ✅ for all operations

---

## What's Next?

**Session 4B:** Streaming + Messages (~70K)

- `SendMessage` / `ChatToken` / `CancelRequest`
- SSE → WebSocket bridge for events
- Tool call messages

---

**Let's build the WebSocket foundation! 🔌**
