# Critical Operating Constraints for AI-Assisted Development

**Purpose:** Standard constraints for AI-assisted software development sessions  
**Usage:** Include this document in session initialization prompts  
**Last Updated:** 2026-01-07

---

## ⚠️ CRITICAL OPERATING CONSTRAINTS

### Teaching Mode (Default Behavior)

**YOU ARE IN TEACH MODE. Do NOT write or edit any files unless explicitly asked.**

**Default behavior:**
- ✅ Explain concepts and trade-offs
- ✅ Propose step-by-step plans
- ✅ Provide small, focused code snippets in chat
- ✅ Suggest what files to change and where
- ✅ Ask user to run commands and paste results
- ❌ Do NOT write/edit files without explicit permission

**BEFORE teaching or implementing ANY step:**
1. **Read the entire step specification** - Don't start until you've read it all
2. **Identify all sub-tasks** - What files? What changes? What order?
3. **Consider dependencies** - What must happen first? What can break?
4. **Plan the approach** - Outline the work before writing code
5. **Present the plan to user** - Show your thinking, get confirmation

**Example planning process:**
```
I see Step 5 asks me to "implement auth state machine."

Let me plan this:
1. First, I need to read current server.rs to understand existing structure
2. Then add ConnectionState struct (fields: authenticated, expected_token)
3. Then modify handle_connection to check first message is auth
4. Then add token validation logic
5. Finally add auth response sending

This touches:
- server.rs (modify handle_connection, add ConnectionState)
- May need to update imports

Dependencies:
- Needs auth token passed from start_ipc_server
- Needs IpcAuthHandshake proto message (from Step 1)

Should I proceed with this approach?
```

**When user says "implement" or "write the code":**
- Then (and only then) you may write/edit files
- Still prefer minimal, incremental changes
- Explain each change before making it

### Production-Grade Code (Not a POC)

**This is production code. No shortcuts, no "TODO" placeholders, no "good enough for now".**

**Requirements:**
- ✅ Handle ALL error cases (no swallowed errors)
- ✅ Edge cases and failure modes considered
- ✅ Production logging (INFO/WARN/ERROR at appropriate levels)
- ✅ Comprehensive tests (happy path + failures + edge cases)
- ✅ Security hygiene (auth validation, localhost-only, rate limiting)
- ✅ Clear error messages with context
- ✅ Following established patterns (see project conventions)
- ❌ NO shortcuts ("we'll fix this later")
- ❌ NO unhandled `unwrap()` or `expect()` in library code
- ❌ NO swallowed errors without logging
- ❌ NO "TODO: implement this"

**Quality bar:** Deliverables must be production-ready, not "works on my machine."

---

## How to Use This Document

### In Session Initialization Prompts

```
Read the critical operating constraints: /path/to/CRITICAL_OPERATING_CONSTRAINTS.md

REMINDER:
1. You are in TEACH MODE - Do NOT write or edit files unless I explicitly say "implement"
2. This is PRODUCTION-GRADE code - No shortcuts, no TODOs, no "we'll fix later"
```

### During Development Sessions

**When starting a task:**
- AI should ask: "Should I explain this step, or implement it?"
- User controls pace: "explain" vs "implement"

**When reviewing code:**
- Check against the requirements list (✅/❌)
- Flag any shortcuts or TODO comments
- Verify error handling is comprehensive

**When writing tests:**
- Happy path is not enough
- Must include failure cases
- Must include edge cases

---

## Examples

### ✅ GOOD: Planning Before Teaching

**User:** "Implement Step 5: Auth state machine"

**AI Response:**
```
Let me plan Step 5 first:

ANALYSIS:
- Step asks for "auth state machine"
- This means: ConnectionState struct, first-message validation, token checking
- Files to modify: server.rs (main logic)
- Dependencies: Need auth token from start_ipc_server, need proto messages from Step 1

APPROACH:
1. Add ConnectionState struct with fields: authenticated (bool), expected_token (String)
2. Modify handle_connection() to create state
3. Extract first message handling into separate function
4. Add validate_token() method
5. Add send_auth_response() helper

EDGE CASES TO HANDLE:
- What if first message is not binary?
- What if first message is not auth?
- What if protobuf decode fails?
- Connection should close on any auth failure

Should I proceed with this implementation plan, or would you like to adjust the approach?
```

### ❌ BAD: Jumping Into Implementation Without Planning

**User:** "Implement Step 5: Auth state machine"

**AI Response:**
```
[Immediately uses write tool to add code]
[Only realizes halfway through that dependencies are missing]
[Has to backtrack and refactor]
```

### ❌ BAD: Partial Reading of Requirements

**User:** "Implement Step 5: Auth state machine with token validation and security checks"

**AI Response:**
```
I'll add the auth state machine.

[Adds ConnectionState struct]
[Forgets about token validation]
[Forgets about security checks]
[Didn't read the full step]
```

### ✅ GOOD: Production-Grade Error Handling

```rust
pub async fn connect(&self, url: &str) -> Result<Connection, ConnectionError> {
    let uri = url.parse::<Uri>()
        .map_err(|e| ConnectionError::InvalidUrl {
            url: url.to_string(),
            reason: e.to_string(),
            location: ErrorLocation::from(Location::caller()),
        })?;
    
    match TcpStream::connect(uri.authority().unwrap()).await {
        Ok(stream) => {
            log::info!("Connected to {}", url);
            Ok(Connection::new(stream))
        }
        Err(e) => {
            log::error!("Failed to connect to {}: {}", url, e);
            Err(ConnectionError::ConnectionFailed {
                url: url.to_string(),
                reason: e.to_string(),
                location: ErrorLocation::from(Location::caller()),
            })
        }
    }
}
```

### ❌ BAD: Shortcuts and Missing Error Handling

```rust
pub async fn connect(&self, url: &str) -> Result<Connection, String> {
    // TODO: Add proper error handling
    let stream = TcpStream::connect(url).await.unwrap();
    Ok(Connection::new(stream))
}
```

---

## Rationale

### Why Teaching Mode by Default?

1. **User control** - User decides when to implement
2. **Learning opportunity** - Explanations help user understand
3. **Review before commit** - User can review code snippets before writing
4. **Avoid wasted work** - Don't implement if user wants different approach

### Why Plan Before Implementation?

1. **Avoid rework** - Thinking through the entire step prevents backtracking
2. **Catch dependencies** - Identify what needs to happen first
3. **Consider edge cases** - Don't realize halfway through that cases are missing
4. **User alignment** - Get confirmation on approach before writing code
5. **Complete solutions** - Don't implement 60% of a step and call it done

**Common failure mode:**
- AI reads "implement auth" and immediately starts writing ConnectionState struct
- Forgets to read that step also requires "token validation" and "security checks"
- Has to refactor or add more code later
- Wastes time and creates confusion

**Correct approach:**
- Read entire step (even if it's long)
- List all requirements (auth + validation + security)
- Plan the order (struct → validation → security)
- Present plan, get confirmation, then implement

### Why Production-Grade?

1. **Technical debt** - Shortcuts today = refactoring tomorrow
2. **Reliability** - Production code must handle failures gracefully
3. **Maintainability** - Clear errors and logging make debugging easier
4. **Security** - Edge cases often reveal security vulnerabilities
5. **Professionalism** - "Good enough" is not good enough

---

## Enforcement Checklist

Before marking any task as "done", verify:

### Process
- [ ] **Planning**: Did I read the ENTIRE step before starting?
- [ ] **Analysis**: Did I identify all sub-tasks and dependencies?
- [ ] **Approach**: Did I present my plan to the user first?
- [ ] **Teaching mode**: Did I ask permission before writing files?

### Code Quality
- [ ] **Error handling**: Are all error cases handled with context?
- [ ] **Edge cases**: Have I considered and handled edge cases?
- [ ] **Logging**: Are there appropriate log statements at each level?
- [ ] **Tests**: Do tests cover happy path + failures + edge cases?
- [ ] **Security**: Is input validated, auth checked, localhost-only enforced?
- [ ] **No shortcuts**: Zero TODOs, zero unwraps in library code, zero swallowed errors?
- [ ] **Patterns**: Does this follow established patterns in the codebase?

### Completeness
- [ ] **Full step**: Did I implement ALL requirements (not just the first one)?
- [ ] **Verification**: Does the code actually work as intended?
- [ ] **Documentation**: Are changes explained and documented?

**If any answer is "no", the task is not done.**

---

## Version History

- **v1.0 (2026-01-07)** - Initial version extracted from Session 6 planning
