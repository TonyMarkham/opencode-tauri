# Architecture Principles - Implementation Guide

**Last Updated:** 2026-01-05

> **📖 This is the implementation guide. For the decision rationale, see [ADR-0002: Thin Tauri Layer Principle](./adr/0002-thin-tauri-layer-principle.md)**

---

## Core Principle: Thin Tauri Layer

> **Tauri is ONLY for hosting the webview. Everything else lives in client-core.**

This is the fundamental architectural rule that guides all design decisions.

**See ADR-0002 for:**
- Why this decision was made
- Alternatives considered
- Consequences and trade-offs

**This guide provides:**
- How to implement the principle
- Code examples (good vs bad)
- Anti-patterns to avoid

---

## Layer Responsibilities

### 1. Tauri App Layer (`apps/desktop/opencode/`)

**Role:** Minimal webview host + OS bridge

**Allowed:**
- ✅ Host the Blazor webview
- ✅ Initialize services from client-core (call `client_core::*::start()`)
- ✅ Provide OS-specific APIs (file dialogs, system tray, notifications)
- ✅ Spawn background tasks (but implementation lives in client-core)
- ✅ Handle app lifecycle (startup, shutdown)
- ✅ Production logging setup

**Not Allowed:**
- ❌ Business logic
- ❌ HTTP client implementations
- ❌ gRPC service implementations
- ❌ State management (beyond what's OS-specific)
- ❌ OpenCode server communication
- ❌ Session/message/tool/agent logic

**Code smell:** If `main.rs` or `commands/*.rs` contains logic that could be tested without Tauri, it belongs in client-core.

---

### 2. Client Core (`backend/client-core/`)

**Role:** Self-contained application engine

**Responsibilities:**
- ✅ All gRPC service implementations
- ✅ All HTTP communication with OpenCode server
- ✅ SSE event streaming and parsing
- ✅ Session/message/tool/agent management
- ✅ OpenCode server discovery and spawning
- ✅ State management (sessions, tabs, working directory)
- ✅ Business logic for all features

**Key Characteristic:** Can be tested without Tauri, GUI, or webview.

**Exports to Tauri:**
```rust
// All Tauri does is call these startup functions
pub mod grpc {
    pub async fn start_grpc_server(addr: impl Into<SocketAddr>) -> Result<(), Error>;
}

pub mod discovery {
    pub async fn discover() -> Result<ServerInfo, DiscoveryError>;
    pub async fn spawn_and_wait(path: &Path) -> Result<ServerInfo, SpawnError>;
}
```

---

### 3. Blazor UI (`frontend/desktop/opencode/`)

**Role:** User interface only

**Responsibilities:**
- ✅ Razor components (UI rendering)
- ✅ gRPC client services (thin wrappers around `Grpc.Net.Client`)
- ✅ User input handling
- ✅ State display (read-only view of server state)
- ✅ Markdown rendering (Markdig)

**Not Allowed:**
- ❌ Direct OpenCode server communication (goes through gRPC → client-core)
- ❌ Business logic (that's client-core's job)
- ❌ Custom JavaScript (Tauri IPC via IJSRuntime only)

---

## Data Flow

```
User Action
    ↓
Blazor Component (C#)
    ↓ [gRPC call]
client-core gRPC Server (Rust)
    ↓ [HTTP REST]
OpenCode Server
    ↓ [HTTP/SSE response]
client-core gRPC Server
    ↓ [gRPC stream]
Blazor Component
    ↓
UI Update
```

**Tauri is not in this flow.** It just hosts the webview.

---

## Testability

### Good: client-core Tests (No Tauri Required)

```rust
#[cfg(test)]
mod tests {
    use client_core::grpc::OpenCodeGrpcServer;
    
    #[tokio::test]
    async fn test_list_sessions() {
        let server = OpenCodeGrpcServer::new();
        let result = server.list_sessions(Request::new(Empty {})).await;
        assert!(result.is_ok());
    }
}
```

### Bad: Logic in Tauri (Requires Full App)

```rust
// Don't do this - can't test without spinning up Tauri
#[tauri::command]
async fn list_sessions() -> Result<SessionList, String> {
    // HTTP call to OpenCode server
    // Parsing logic
    // Error handling
    // ❌ This should all be in client-core
}
```

---

## Reusability

Because client-core is self-contained:

### Future CLI Tool
```rust
// src/main.rs
use client_core::grpc;

#[tokio::main]
async fn main() {
    // Start gRPC server
    grpc::start_grpc_server("127.0.0.1:50051").await.unwrap();
}
```

### Future egui Integration
```rust
// egui app
use client_core::grpc;

fn main() {
    // Spawn gRPC server in background
    std::thread::spawn(|| {
        tokio::runtime::Runtime::new()
            .unwrap()
            .block_on(grpc::start_grpc_server("127.0.0.1:50051"))
            .unwrap();
    });
    
    // Start egui app
    eframe::run_native(...);
}
```

**If gRPC server lived in Tauri, these wouldn't be possible.**

---

## Decision Checklist

When adding new functionality, ask:

### Question 1: Does this need OS-level access?

- **Yes** (file dialogs, system tray, notifications) → Tauri layer
- **No** (HTTP, gRPC, business logic) → client-core

### Question 2: Can this be tested without Tauri?

- **Yes** → client-core
- **No** (webview-specific) → Tauri layer

### Question 3: Would a CLI tool want this?

- **Yes** (session management, OpenCode server communication) → client-core
- **No** (webview hosting) → Tauri layer

### Question 4: Is this pure UI rendering?

- **Yes** → Blazor frontend
- **No** (data fetching, state management) → client-core via gRPC

---

## Examples

### ✅ Good: Discovery Lives in client-core

**Tauri layer (minimal):**
```rust
// apps/desktop/opencode/src/commands/server.rs
use client_core::discovery;

#[tauri::command]
pub async fn discover_server() -> Result<ServerInfo, String> {
    discovery::discover()
        .await
        .map_err(|e| e.to_string())
}
```

**client-core (where logic lives):**
```rust
// backend/client-core/src/discovery/process.rs
pub async fn discover() -> Result<ServerInfo, DiscoveryError> {
    // All the implementation
}
```

### ✅ Good: gRPC Server in client-core

**Tauri layer (minimal glue):**
```rust
// apps/desktop/opencode/src/main.rs
use client_core::grpc;

tauri::Builder::default()
    .setup(|app| {
        tokio::spawn(async {
            grpc::start_grpc_server("127.0.0.1:50051").await.unwrap();
        });
        Ok(())
    })
    .run(...)
```

**client-core (where implementation lives):**
```rust
// backend/client-core/src/grpc/mod.rs
pub async fn start_grpc_server(addr: impl Into<SocketAddr>) -> Result<(), Error> {
    let server = OpenCodeGrpcServer::new();
    Server::builder()
        .add_service(OpenCodeServiceServer::new(server))
        .serve(addr.into())
        .await?;
    Ok(())
}
```

### ❌ Bad: Business Logic in Tauri

```rust
// DON'T DO THIS
#[tauri::command]
async fn send_message(session_id: String, content: String) -> Result<Message, String> {
    // ❌ HTTP client here
    // ❌ Request building here
    // ❌ Response parsing here
    // ❌ Error handling here
    
    // All of this should be in client-core
}
```

**Instead:**
```rust
// Tauri (minimal wrapper)
#[tauri::command]
async fn send_message(session_id: String, content: String) -> Result<Message, String> {
    client_core::messages::send_message(session_id, content)
        .await
        .map_err(|e| e.to_string())
}
```

---

## Dependency Direction

```
Good (thin depends on thick):
    Tauri App → client-core
    Blazor → client-core (via gRPC)

Bad (thick depends on thin):
    client-core → Tauri App  ❌ Never do this
```

---

## Historical Context

### Session 1 & 2: Established the Pattern

- **Session 1:** Built client-core with discovery, spawn, health checks
- **Session 2:** Tauri **used** client-core functions (didn't reimplement)

### Session 4.5: Continue the Pattern

- **client-core:** Implements all gRPC services
- **Tauri:** Calls `client_core::grpc::start_grpc_server()` and that's it

---

## Benefits

### 1. Testability
- client-core fully testable without Tauri
- No GUI needed for unit/integration tests
- Fast test execution

### 2. Reusability
- CLI tools can use client-core directly
- Future GUI frameworks can use client-core
- Shared with egui client (if desired)

### 3. Separation of Concerns
- Tauri handles OS integration
- client-core handles application logic
- Blazor handles UI rendering

### 4. Maintainability
- Clear boundaries between layers
- Changes to Tauri don't affect business logic
- Business logic changes don't require Tauri rebuild

### 5. Code Organization
- Easy to find where logic lives
- No "should this be in Tauri or client-core?" questions
- Rule is simple: "Is it webview hosting? No? → client-core"

---

## Anti-Patterns to Avoid

### ❌ Anti-Pattern 1: Logic in Tauri Commands

```rust
// DON'T DO THIS
#[tauri::command]
async fn complex_operation() -> Result<Foo, String> {
    // 100 lines of business logic
}
```

**Fix:** Extract to client-core.

### ❌ Anti-Pattern 2: client-core Imports from Tauri

```rust
// backend/client-core/src/lib.rs
use apps_desktop_opencode::something;  // ❌ WRONG DIRECTION
```

**Rule:** Dependency arrows point from Tauri → client-core, never the reverse.

### ❌ Anti-Pattern 3: Blazor Direct OpenCode Server Calls

```csharp
// DON'T DO THIS
public async Task<Session> CreateSession()
{
    // ❌ Direct HTTP call to OpenCode server
    var response = await httpClient.PostAsync("http://localhost:8080/session");
}
```

**Fix:** Call gRPC service (which lives in client-core).

---

## Summary

**The Rule:**

> Tauri = webview host + service launcher
> client-core = everything else

**When in doubt:** Put it in client-core.

**Exception:** Only if it requires OS-level APIs that Tauri provides (file dialogs, system tray, etc.).
