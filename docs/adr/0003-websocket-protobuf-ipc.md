# ADR-0003: WebSocket + Protobuf for Blazor-to-Rust IPC

**Status:** Accepted

**Date:** 2026-01-05

**Deciders:** Tony (repository owner)

**Context Owners:** Desktop client development team

---

## Context

The Tauri + Blazor desktop client needs a communication layer between:
- **Blazor WASM frontend** (C#) - runs in browser sandbox inside Tauri webview
- **client-core backend** (Rust) - application logic, OpenCode server communication

### Requirements

1. **Streaming support** - LLM chat requires token-by-token streaming
2. **Cross-platform** - Must work on Windows, macOS, Linux
3. **No custom JavaScript** - Universal constraint (see [NO_CUSTOM_JAVASCRIPT_POLICY.md](./NO_CUSTOM_JAVASCRIPT_POLICY.md))
4. **Type safety** - Shared schema between C# and Rust
5. **Production-grade** - No roll-your-own protocols

### The Browser Sandbox Constraint

Blazor WASM runs inside a **browser sandbox** (even within Tauri's webview). This sandbox **cannot**:

- Load native libraries (libzmq, nng, etc.)
- Open arbitrary file descriptors
- Use shared memory
- Call FFI directly
- Use non-web transports

Blazor WASM can **only** communicate via:
- HTTP / fetch
- WebSockets
- WebRTC data channels
- Tauri invoke (but this requires JavaScript bridge)

### Why Not gRPC?

ADR-0001 originally specified gRPC for IPC. Investigation revealed:

1. **gRPC requires HTTP/2** - Even over Unix Domain Sockets, gRPC uses HTTP/2 framing
2. **HTTP/2 is still "web"** - Conceptually not what we want for desktop IPC
3. **Complexity** - gRPC adds significant overhead for local communication
4. **Streaming works, but...** - gRPC streaming is designed for network, not local IPC

### Why Not Tauri Invoke?

Current implementation uses Tauri's built-in invoke system:

```csharp
await _jsRuntime.InvokeAsync<JsonElement>(
    "window.__TAURI_INTERNALS__.invoke",
    "command_name"
);
```

Problems:
1. **No streaming** - Request/response only, no bidirectional streaming
2. **Goes through JavaScript** - Violates spirit of zero-JS policy
3. **Not suitable for LLM chat** - Can't stream tokens

### Why Not ZeroMQ / nng?

These are excellent native IPC libraries, but:
1. **Not accessible from WASM** - Browser sandbox blocks native libraries
2. **Would require a bridge** - Adds complexity, loses advantages
3. **Effectively reinvents WebSockets** - With more moving parts

---

## Decision

**Use WebSockets with binary Protobuf messages for Blazor-to-Rust IPC.**

### Architecture

```
Blazor WASM (C#)
    ↓ System.Net.WebSockets.ClientWebSocket
    ↓ Binary protobuf frames
WebSocket (ws://127.0.0.1:PORT)
    ↓ tokio-tungstenite
    ↓ prost
client-core (Rust)
    ↓ reqwest HTTP
OpenCode Server
```

### Key Properties

- **No JavaScript** - C# `ClientWebSocket` is native, no JSInterop needed
- **Bidirectional streaming** - Perfect for LLM token streaming
- **Binary protocol** - Protobuf, not JSON (no stringly-typed nonsense)
- **Cross-platform** - WebSocket works identically everywhere
- **Local only** - Binds to `127.0.0.1`, no network exposure

### Design Principles

1. **Blazor is dumb glass** - Renders tokens, errors, progress. Never interprets domain logic, decides state transitions, or reassembles messages. Rust owns everything semantic.

2. **One WebSocket = one session** - Open on app start, close on app exit, reconnect on crash. No per-request sockets. Feels "in-process".

3. **Binary-only protocol** - No JSON, no text frames. Every frame is a protobuf message.

### Message Protocol

Binary WebSocket frames with protobuf messages. Use `oneof` for message envelope:

```protobuf
// Client → Server
message ClientMessage {
  uint64 request_id = 1;
  
  oneof payload {
    // Session management
    ListSessionsRequest list_sessions = 10;
    CreateSessionRequest create_session = 11;
    DeleteSessionRequest delete_session = 12;
    
    // Chat
    SendMessageRequest send_message = 20;
    CancelRequest cancel = 21;
    
    // Other operations...
  }
}

// Server → Client
message ServerMessage {
  uint64 request_id = 1;
  
  oneof payload {
    // Responses
    SessionList session_list = 10;
    SessionInfo session_info = 11;
    
    // Streaming (LLM tokens)
    ChatToken token = 20;
    ChatCompleted completed = 21;
    ChatError error = 22;
    
    // Tool execution
    ToolCallEvent tool_call = 30;
    ToolResultEvent tool_result = 31;
  }
}

// Streaming messages
message ChatToken {
  string text = 1;
}

message ChatCompleted {}

message CancelRequest {}

message ChatError {
  string message = 1;
  string code = 2;
}
```

- `request_id` enables multiplexing (multiple concurrent requests)
- Tokens stream independently as separate messages
- Cancellation is a simple message send

### Libraries

**Rust (client-core):**
- `tokio-tungstenite` - Async WebSocket server
- `prost` - Protobuf serialization
- `prost-build` - Code generation

**C# (Blazor):**
- `System.Net.WebSockets.ClientWebSocket` - Built-in, no package needed
- `Google.Protobuf` - Protobuf serialization

---

## Alternatives Considered

### Alternative 1: gRPC over IPC (Unix Domain Sockets / Named Pipes)

**Description:** Use gRPC with custom transport over local sockets

**Pros:**
- Full gRPC feature set (streaming, metadata, deadlines)
- Official library support on both sides
- Well-documented patterns

**Cons:**
- Still uses HTTP/2 framing (conceptually "web")
- More complex than needed for local IPC
- Heavier dependencies

**Why rejected:** Adds unnecessary complexity. WebSocket + Protobuf gives us streaming without HTTP/2 overhead.

### Alternative 2: Tauri Invoke with Protobuf

**Description:** Pass protobuf bytes through Tauri's invoke system

**Pros:**
- Already works (current implementation)
- Cross-platform by default
- No additional server to manage

**Cons:**
- **No streaming** - Dealbreaker for LLM chat
- Goes through JavaScript bridge
- Request/response only

**Why rejected:** Cannot stream tokens. Fundamental limitation.

### Alternative 3: ZeroMQ / nanomsg (nng)

**Description:** Use message queue libraries for IPC

**Pros:**
- Excellent native IPC performance
- Built-in patterns (PUB/SUB, REQ/REP)
- Cross-platform abstraction

**Cons:**
- **Not accessible from WASM** - Browser sandbox blocks native libs
- Would require JavaScript bridge (defeating the purpose)
- Adds complexity without benefit

**Why rejected:** Cannot be used directly from Blazor WASM.

### Alternative 4: Raw TCP with Protobuf

**Description:** Direct TCP socket with length-prefixed protobuf

**Pros:**
- Simple protocol
- No HTTP overhead
- Full control

**Cons:**
- Blazor WASM cannot open raw TCP sockets
- Browser sandbox limitation

**Why rejected:** Not possible from WASM environment.

### Alternative 5: HTTP + Server-Sent Events (SSE)

**Description:** REST for requests, SSE for streaming responses

**Pros:**
- Well-understood pattern
- Works in browser

**Cons:**
- Two protocols to maintain (HTTP + SSE)
- SSE is text-only (would need base64 for binary)
- Unidirectional streaming only

**Why rejected:** WebSocket is simpler (single bidirectional connection).

---

## Consequences

### Positive

- **Streaming works** - Token-by-token LLM responses
- **No JavaScript** - Pure C# WebSocket client
- **Type safety** - Shared protobuf schemas
- **Simple mental model** - One bidirectional connection
- **Cross-platform** - Identical behavior everywhere
- **Cancellation** - Can send cancel message mid-stream
- **Backpressure** - WebSocket flow control built-in
- **Future-proof** - Can swap Blazor for native UI without rewriting backend
- **Blazor stays dumb** - All logic in Rust, UI just renders

### Negative

- **Port management** - Need to pick/manage a local port
- **Connection lifecycle** - Must handle connect/disconnect/reconnect
- **Slightly more code** - Than Tauri invoke (but not much)
- **Local server** - client-core runs a WebSocket server

### Neutral

- **Different from Tauri invoke** - But cleaner for our use case
- **Binary protocol** - Harder to debug than JSON (use logging)

---

## Implementation Notes

### Rust WebSocket Server (client-core)

```rust
use tokio::net::TcpListener;
use tokio_tungstenite::accept_async;
use prost::Message;

pub async fn start_ws_server(port: u16, auth_token: &str) -> Result<(), Error> {
    // SECURITY: Bind only to localhost
    let listener = TcpListener::bind(format!("127.0.0.1:{}", port)).await?;
    
    while let Ok((stream, addr)) = listener.accept().await {
        // SECURITY: Reject non-local connections
        if !addr.ip().is_loopback() {
            continue;
        }
        
        let ws_stream = accept_async(stream).await?;
        let token = auth_token.to_string();
        tokio::spawn(handle_connection(ws_stream, token));
    }
    Ok(())
}

async fn handle_connection(ws: WebSocketStream<TcpStream>, auth_token: String) {
    let (mut write, mut read) = ws.split();
    
    // SECURITY: First message must be auth handshake
    if !validate_handshake(&mut read, &auth_token).await {
        return;
    }
    
    while let Some(msg) = read.next().await {
        if let Ok(Message::Binary(data)) = msg {
            let request = ClientMessage::decode(&data[..])?;
            
            // Spawn task for streaming responses
            let response_tx = /* channel to write */;
            tokio::spawn(handle_request(request, response_tx));
        }
    }
}
```

### C# WebSocket Client (Blazor)

```csharp
public class WebSocketService : IAsyncDisposable
{
    private readonly ClientWebSocket _socket = new();
    
    public async Task ConnectAsync(string url)
    {
        await _socket.ConnectAsync(new Uri(url), CancellationToken.None);
        _ = ReceiveLoop(); // Start background receive
    }
    
    public async Task SendAsync<T>(T message) where T : IMessage<T>
    {
        var bytes = message.ToByteArray();
        await _socket.SendAsync(bytes, WebSocketMessageType.Binary, true, CancellationToken.None);
    }
    
    private async Task ReceiveLoop()
    {
        var buffer = new byte[4096];
        while (_socket.State == WebSocketState.Open)
        {
            var result = await _socket.ReceiveAsync(buffer, CancellationToken.None);
            if (result.MessageType == WebSocketMessageType.Binary)
            {
                var envelope = Envelope.Parser.ParseFrom(buffer, 0, result.Count);
                OnMessageReceived?.Invoke(envelope);
            }
        }
    }
    
    public event Action<Envelope>? OnMessageReceived;
}
```

### Tauri Integration

Tauri's role is minimal - just start the WebSocket server:

```rust
// apps/desktop/opencode/src/main.rs
tauri::Builder::default()
    .setup(|app| {
        // Start WebSocket server in client-core
        let port = find_available_port();
        tokio::spawn(async move {
            client_core::ws::start_ws_server(port).await.unwrap();
        });
        
        // Store port for Blazor to connect
        app.manage(WebSocketPort(port));
        Ok(())
    })
```

### Port Discovery

Blazor needs to know which port to connect to. Options:

1. **Fixed port** - Simple, but risks conflicts
2. **Tauri command** - Query port via invoke (one-time JS call, acceptable)
3. **Environment variable** - Set by Tauri, read by Blazor

Recommended: Use a Tauri command to get the port on startup.

### Security

Even though this is local IPC, enforce security hygiene:

1. **Bind to `127.0.0.1` only** - Never `0.0.0.0`
2. **Random startup token** - Generate on app start, require in handshake
3. **Reject non-local clients** - Check `addr.ip().is_loopback()`
4. **Close socket on window close** - Clean shutdown
5. **Rate-limit inbound messages** - Prevent DoS from malicious local process

```rust
// Generate random token on startup
let auth_token = uuid::Uuid::new_v4().to_string();

// Pass to WebSocket server
client_core::ws::start_ws_server(port, &auth_token).await;

// Pass to Blazor via Tauri command (one-time)
app.manage(AuthToken(auth_token));
```

Blazor sends token in first WebSocket message to authenticate.

### Testing

1. **Unit tests** - Test protobuf encoding/decoding
2. **Integration tests** - Spin up WebSocket server, connect client
3. **Streaming tests** - Verify token-by-token delivery
4. **Reconnection tests** - Handle disconnect/reconnect gracefully
5. **Security tests** - Verify non-local connections rejected, invalid tokens rejected

---

## Related ADRs

- [ADR-0001](./0001-tauri-blazor-desktop-client.md) - Parent decision (Tauri + Blazor) - **Updated to reference this ADR for IPC**
- [ADR-0002](./0002-thin-tauri-layer-principle.md) - WebSocket server lives in client-core, not Tauri

---

## References

- [System.Net.WebSockets.ClientWebSocket](https://docs.microsoft.com/en-us/dotnet/api/system.net.websockets.clientwebsocket) - C# WebSocket client (built-in)
- [tokio-tungstenite](https://crates.io/crates/tokio-tungstenite) - Async WebSocket for Rust
- [prost](https://crates.io/crates/prost) - Protobuf for Rust
- [Google.Protobuf](https://www.nuget.org/packages/Google.Protobuf) - Protobuf for C#

---

## Future Considerations

This architecture doesn't box you in:

- **Swap Blazor for native UI** - Protocol stays the same
- **Swap webview host** - client-core is independent of Tauri
- **Expose backend externally** - Add TLS, auth, same protocol
- **Add CLI client** - Connect via same WebSocket

The protobuf protocol is the stable contract. Everything else is swappable.
