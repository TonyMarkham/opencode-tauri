# ADR-0002: Thin Tauri Layer Principle

**Status:** Accepted  
**Date:** 2026-01-05  
**Supersedes:** N/A  
**Superseded by:** N/A  

---

## Context

During implementation of the Tauri + Blazor client (ADR-0001), a critical architectural question emerged:

**Where should application logic live?**

Options considered:
1. **Tauri layer** (`apps/desktop/opencode/src/`) - Commands, state management, HTTP client
2. **client-core layer** (`backend/client-core/`) - Pure Rust library with all logic
3. **Mixed approach** - Some logic in Tauri, some in client-core

Sessions 1 & 2 established a pattern:
- Session 1: Built `client-core/` with discovery, spawn, health check logic
- Session 2: Tauri **used** `client-core` functions (thin wrapper)

Session 4 planning raised the question again for gRPC services:
- Should Tauri host the gRPC server?
- Or should client-core host it, with Tauri just calling `start()`?

## Decision

**Tauri layer is ONLY for hosting the webview. All application logic lives in client-core.**

This is the fundamental architectural principle that guides all design decisions.

### Layer Responsibilities

#### Tauri Layer (`apps/desktop/opencode/`)

**Allowed:**
- ✅ Host the Blazor webview
- ✅ Initialize services from client-core (call `client_core::*::start()`)
- ✅ Provide OS-specific APIs (file dialogs, system tray, notifications)
- ✅ Spawn background tasks (implementation in client-core)
- ✅ Handle app lifecycle (startup, shutdown)
- ✅ Production logging setup

**Not Allowed:**
- ❌ Business logic
- ❌ HTTP client implementations
- ❌ gRPC service implementations
- ❌ State management (beyond OS-specific)
- ❌ OpenCode server communication
- ❌ Session/message/tool/agent logic

**Code smell:** If code in Tauri could be tested without the webview, it belongs in client-core.

#### client-core Layer (`backend/client-core/`)

**Responsibilities:**
- ✅ All gRPC service implementations
- ✅ All HTTP communication with OpenCode server
- ✅ SSE event streaming and parsing
- ✅ Session/message/tool/agent management
- ✅ OpenCode server discovery and spawning
- ✅ State management (sessions, tabs, working directory)
- ✅ All business logic

**Key characteristic:** Can be tested without Tauri, GUI, or webview.

## Rationale

### 1. Testability

**Good (client-core):**
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

No Tauri, no webview, no GUI - just pure Rust testing.

**Bad (logic in Tauri):**
```rust
#[tauri::command]
async fn list_sessions() -> Result<SessionList, String> {
    // ❌ Business logic here requires full Tauri app to test
}
```

### 2. Reusability

Because client-core is self-contained, future use cases are simple:

**CLI tool:**
```rust
use client_core::grpc;

#[tokio::main]
async fn main() {
    grpc::start_grpc_server("127.0.0.1:50051").await.unwrap();
}
```

**Alternative GUI (egui, iced, etc.):**
```rust
use client_core::grpc;

fn main() {
    std::thread::spawn(|| {
        tokio::runtime::Runtime::new()
            .unwrap()
            .block_on(grpc::start_grpc_server("127.0.0.1:50051"))
            .unwrap();
    });
    
    // Start GUI
}
```

**If gRPC server lived in Tauri, these would be impossible.**

### 3. Separation of Concerns

Clear boundaries:
- **Tauri** handles OS integration (webview, file dialogs, system tray)
- **client-core** handles application logic (sessions, messages, OpenCode server)
- **Blazor** handles UI rendering (components, styling, user input)

### 4. Dependency Direction

```
Good (thin depends on thick):
    Tauri → depends on → client-core
    Blazor → depends on → client-core (via gRPC)

Bad (thick depends on thin):
    client-core → Tauri  ❌ Never
```

### 5. Maintainability

- **Easy to find:** "Is it webview hosting? No? → client-core"
- **Easy to change:** Tauri changes don't affect business logic
- **Easy to test:** No GUI needed for business logic tests

## Implementation

### Example 1: Server Discovery (Session 1 & 2)

**client-core exports the function:**
```rust
// backend/client-core/src/discovery/process.rs
pub async fn discover() -> Result<ServerInfo, DiscoveryError> {
    // All implementation here
}
```

**Tauri is minimal wrapper:**
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

### Example 2: gRPC Server (Session 4.5)

**client-core hosts the gRPC server:**
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

**Tauri just spawns it:**
```rust
// apps/desktop/opencode/src/main.rs
use client_core::grpc;

tauri::Builder::default()
    .setup(|app| {
        tokio::spawn(async {
            if let Err(e) = grpc::start_grpc_server("127.0.0.1:50051").await {
                error!("gRPC server failed: {}", e);
            }
        });
        Ok(())
    })
    .run(...)
```

That's it. ~5 lines of glue code.

## Consequences

### Positive

1. **Testability** - client-core fully testable without GUI
2. **Reusability** - CLI tools, alternative GUIs can use client-core
3. **Separation of concerns** - Clear boundaries between layers
4. **Maintainability** - Easy to find where logic lives
5. **Performance** - No unnecessary abstraction layers
6. **Type safety** - Rust type system throughout

### Negative

1. **Slight indirection** - Tauri commands are thin wrappers
   - *Mitigation:* Trivial cost, huge testing benefit
2. **Initial setup complexity** - Need to structure code properly
   - *Mitigation:* Pattern established in Sessions 1 & 2, just follow it

### Risks

None identified. This is a well-established architectural pattern.

## Alternatives Considered

### Alternative 1: Logic in Tauri Layer

**Pros:**
- Simpler initial setup (no client-core crate)
- Direct Tauri command implementation

**Cons:**
- ❌ Cannot test without full Tauri app
- ❌ Cannot reuse in CLI or other clients
- ❌ Tight coupling to Tauri framework
- ❌ Violates separation of concerns

**Verdict:** Rejected. Testing and reusability are critical.

### Alternative 2: Mixed Approach

**Pros:**
- "Flexibility" to put logic wherever convenient

**Cons:**
- ❌ No clear rule for where code should live
- ❌ Leads to inconsistent codebase
- ❌ Hard to maintain over time

**Verdict:** Rejected. Clear rules are better than flexibility.

### Alternative 3: Thin Tauri Layer (Selected)

**Pros:**
- ✅ Testable (client-core isolated)
- ✅ Reusable (client-core self-contained)
- ✅ Clear rule ("Is it webview hosting? No? → client-core")
- ✅ Maintainable (easy to find code)

**Cons:**
- Minimal (~5 lines of wrapper code per feature)

**Verdict:** Selected. Benefits far outweigh costs.

## Decision Checklist

When adding new functionality, ask:

### Question 1: Does this need OS-level access?

- **Yes** (file dialogs, system tray) → Tauri layer
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

## Validation

This principle was validated during Session 4 planning:

**Initial plan:** Tauri hosts gRPC server (Step 4: "Wire gRPC Server into Tauri")

**Question raised:** "Should this interact with client-core and not Tauri?"

**Analysis:**
- gRPC server is not webview-specific ✓
- gRPC server can be tested without Tauri ✓
- CLI tool would want gRPC server ✓
- gRPC server is business logic, not OS integration ✓

**Decision:** Move to client-core, Tauri just calls `start()`.

**Result:**
- Reduced complexity (10K → 5K token estimate)
- Improved testability (no Tauri needed)
- Enabled reusability (CLI/other clients can use)

## Enforcement

### Code Review Checklist

- [ ] Does new Tauri code contain business logic? → Move to client-core
- [ ] Could this be tested without the webview? → Move to client-core
- [ ] Does this make HTTP/gRPC calls? → Move to client-core
- [ ] Is this just calling a client-core function? → Good, keep it thin

### Architecture Documentation

- This ADR - Decision rationale (why, what, when, consequences)
- `docs/ARCHITECTURE.md` - Implementation guide (how, examples, anti-patterns)
- `SESSION_PLAN.md` - Implementation notes (Sessions 1-6)

**Division of labor:**
- **This ADR:** Records the decision for posterity ("Why did we choose this?")
- **ARCHITECTURE.md:** Practical guide for developers ("How do I implement this?")

## Related ADRs

- ADR-0001: Tauri + Blazor WebAssembly Desktop Client (why Tauri was chosen)

## References

- **Separation of Concerns:** https://en.wikipedia.org/wiki/Separation_of_concerns
- **Clean Architecture:** https://blog.cleancoder.com/uncle-bob/2012/08/13/the-clean-architecture.html
- **Hexagonal Architecture:** https://alistair.cockburn.us/hexagonal-architecture/

## Decision Makers

- Project Lead: Tony
- Validated: Session 4 planning (2026-01-05)

## Review Date

None needed - This is a foundational principle, not a technology choice that might change.

## Amendments

None.
