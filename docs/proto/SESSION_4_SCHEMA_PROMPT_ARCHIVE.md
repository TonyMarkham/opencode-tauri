# Next Session: Create JSON Schema for Event Types

## Your Task

Create JSON Schema definitions for SSE event types by reading the **TypeScript/Zod source code** in the OpenCode repository, then update the protobuf documentation to reference these schemas.

---

## ⚠️ CRITICAL: Read This First

**Before doing ANY work, you MUST read:**

1. **`docs/proto/SCHEMA_DEVELOPMENT_PROCESS.md`** — The step-by-step process you MUST follow
2. **This prompt in full** — Contains specific guidance for event types

**The process document contains critical verification steps (4.2 and 5.3) that you MUST NOT skip.** These require explicit field-by-field comparison tables. Do not claim "verified" without creating these tables.

---

## Understanding the Goal

**The purpose of this work:**

```
TypeScript/Zod (SOURCE)  →  JSON Schema (YOU CREATE)  →  Protobuf Doc (YOU UPDATE)
         ↓                          ↓                            ↓
   Read this first           Write these files          Update to reference schemas
```

- **Source of truth:** TypeScript/Zod definitions in OpenCode source
- **What you create:** JSON Schema files in `submodules/opencode/schema/`
- **What you update:** Protobuf doc at `docs/proto/08-event.md` (THE ULTIMATE GOAL)

---

## Required Reading

| Priority | File | Purpose |
|----------|------|---------|
| 1 | `docs/proto/SCHEMA_DEVELOPMENT_PROCESS.md` | **THE PROCESS** — Follow every step |
| 2 | `docs/proto/08-event.md` | Proto doc you will update (Step 5) |
| 3 | `docs/proto/05-message.md` | Completed example — reference for quality |
| 4 | `docs/proto/07-agent.md` | Completed example — reference for quality |

---

## TypeScript Source Locations

**The event system uses a BusEvent pattern. Find all event definitions:**

```bash
cd submodules/opencode
rg "BusEvent\.define" packages/opencode/src/ --type ts -A 2
```

**Key files to investigate:**

| File | Events Defined |
|------|----------------|
| `packages/opencode/src/bus/bus-event.ts` | BusEvent.define() factory, BusEvent.payloads() aggregator |
| `packages/opencode/src/session/message-v2.ts` | `message.updated`, `message.removed`, `message.part.updated`, `message.part.removed` |
| `packages/opencode/src/session/index.ts` | `session.created`, `session.updated`, `session.deleted`, `session.diff`, `session.error` |
| `packages/opencode/src/session/status.ts` | `session.status`, `session.idle` |
| `packages/opencode/src/session/compaction.ts` | `session.compacted` |
| `packages/opencode/src/session/todo.ts` | `todo.updated` |
| `packages/opencode/src/permission/next.ts` | `permission.asked`, `permission.replied` |
| `packages/opencode/src/mcp/index.ts` | `mcp.tools.changed` |
| `packages/opencode/src/pty/index.ts` | `pty.created`, `pty.updated`, `pty.exited`, `pty.deleted` |
| `packages/opencode/src/project/vcs.ts` | `vcs.branch.updated` |
| `packages/opencode/src/installation/index.ts` | `installation.updated`, `installation.update-available` |

**SSE Endpoints (server.ts):**

- `GET /global/event` — Returns `{ directory: string, payload: Event }` wrapped events
- `GET /event` — Returns raw `Event` payloads

---

## Event Architecture

The OpenCode server uses a **discriminated union** pattern for events:

```typescript
// bus-event.ts
export function payloads() {
  return z.discriminatedUnion("type", [
    // All registered events with { type: "event.name", properties: {...} }
  ])
}
```

**GlobalEvent wrapper:**

```typescript
z.object({
  directory: z.string(),
  payload: BusEvent.payloads(),  // The discriminated union
})
```

---

## Expected Types to Schematize

Based on the current `08-event.md` proto doc and source research:

| Type | Description |
|------|-------------|
| `GlobalEvent` | Wrapper with directory + payload |
| `MessageUpdated` | Message metadata changed (tokens, finish status) |
| `MessagePartUpdated` | Streaming content chunk (text, reasoning, tool) |
| `MessageRemoved` | Message deleted |
| `PartRemoved` | Part deleted |
| `SessionCreated` | New session |
| `SessionUpdated` | Session metadata changed |
| `SessionDeleted` | Session deleted |
| `SessionStatus` | Session status changed (idle, working, etc.) |
| `PermissionAsked` | Permission request created |
| `PermissionReplied` | Permission response received |

**Note:** Many Part types (TextPart, ToolPart, etc.) already have schemas from the message work. The event schemas should **reference** those existing schemas.

---

## Complexity Warning

**This is more complex than previous schema work because:**

1. **Many event types** — ~20+ different events spread across multiple files
2. **Discriminated union** — Events use `type` field as discriminator
3. **Nested references** — Events reference Part, Message, Session, Permission types
4. **Some types already exist** — Don't duplicate schemas that already exist

**Recommendation:**

1. Start with the **most critical events** for the desktop client:
   - `message.updated`
   - `message.part.updated`
   - `permission.asked`
   
2. Verify each schema works before moving to the next

3. Build up the discriminated union incrementally

---

## Schemas to Create

Based on investigation of the TypeScript source:

```
submodules/opencode/schema/
├── globalEvent.schema.json           # Wrapper: { directory, payload }
├── messageUpdatedEvent.schema.json   # message.updated event
├── messagePartUpdatedEvent.schema.json # message.part.updated event
├── messageRemovedEvent.schema.json   # message.removed event
├── partRemovedEvent.schema.json      # message.part.removed event
├── sessionCreatedEvent.schema.json   # session.created event
├── sessionUpdatedEvent.schema.json   # session.updated event
├── sessionDeletedEvent.schema.json   # session.deleted event
├── sessionStatusEvent.schema.json    # session.status event
├── permissionAskedEvent.schema.json  # permission.asked event
├── permissionRepliedEvent.schema.json # permission.replied event
└── event.schema.json                 # Discriminated union of all events
```

**Note:** Start with the core events. You may not need all of these — research the actual usage first.

---

## Commands You'll Use

```bash
cd submodules/opencode
bun run generate:schemas   # Validate + generate
bun run typecheck          # Verify types
bun test                   # Run tests (544+ should pass)
bun run build              # Production build (11 platforms)
```

**Run ALL of these before claiming completion.**

---

## Verification Requirements

### Step 4.2: TypeScript → JSON Schema → Generated Zod

For EACH schema you create, you MUST:

1. Read the original TypeScript Zod definition
2. Read your JSON Schema
3. Read the generated validator
4. Create an explicit field-by-field comparison table
5. Verify field counts match

**Do NOT claim "verified" without showing the comparison tables.**

### Step 5.3: JSON Schema → Protobuf

For EACH schema, you MUST:

1. Read your JSON Schema
2. Read the protobuf message you wrote in `08-event.md`
3. Create an explicit field-by-field comparison table
4. Verify field counts match
5. Verify types match (using the mapping table in SCHEMA_DEVELOPMENT_PROCESS.md)
6. Verify required/optional matches

**Do NOT claim "verified" without showing the comparison tables.**

---

## Expected Deliverables

### 1. New Schema Files

All event-related schemas created in `submodules/opencode/schema/`.

### 2. Refactored TypeScript (if applicable)

If inline Zod definitions exist, replace them with imports from `@generated/validators/*`.

### 3. Updated Proto Doc

Update `docs/proto/08-event.md`:
- Status changed to ✅ Complete
- Source of Truth section pointing to JSON Schemas
- Protobuf messages matching JSON Schema structure
- JSON Schema Cross-Reference tables
- Verification summary

### 4. All Verification Passing

- [ ] `bun run generate:schemas` ✅
- [ ] `bun run typecheck` ✅
- [ ] `bun test` ✅ (544+ tests)
- [ ] `bun run build` ✅ (11 platforms)
- [ ] Step 4.2 verification tables created for ALL schemas
- [ ] Step 5.3 verification tables created for ALL schemas

---

## ⛔ HARD RULES - VIOLATIONS ARE UNACCEPTABLE

1. **DO NOT COMMIT WITHOUT EXPLICIT USER APPROVAL** — Always show the diff and ask "Do you want me to commit?" before running `git commit`
2. **DO NOT MISREPRESENT WORK DONE** — If you made changes to the submodule, say so. Do not claim you didn't when you did. This is gaslighting.
3. **VERIFY STATUS BEFORE UPDATING** — Before marking anything complete in README, read the actual file to confirm its status
4. **DO NOT PROCEED AT 95% CONFIDENCE** — If you are not 100% confident, STOP and escalate to the user

---

## Lessons from Previous Sessions

The following mistakes were made in earlier sessions. **Do not repeat them:**

1. **Skipped verification steps** — Claimed "verified" without creating comparison tables
2. **Worked from memory** — Did not re-read files before making claims
3. **Read wrong prompt** — Read `NEXT_SESSION_PROMPT.md` from repo root instead of `docs/proto/`
4. **Skipped build step** — Did not run `bun run build` before claiming completion
5. **Marked complete prematurely** — Updated todo status before verification was done
6. **Committed without approval** — Ran `git commit` without asking user first
7. **Gaslighted about work done** — Claimed "I did NOT make changes to the submodule" when submodule changes were the primary deliverable
8. **Perpetuated stale status** — Copied WIP status from README without checking if actual file was already complete
9. **Proceeded at 95% confidence** — Should have stopped and escalated when not 100% confident on Step 4.2

**The guardrails in SCHEMA_DEVELOPMENT_PROCESS.md exist because of these failures. Follow them.**

---

## Starting Point

1. Read `docs/proto/SCHEMA_DEVELOPMENT_PROCESS.md` in full
2. Run `rg "BusEvent\.define" packages/opencode/src/ --type ts -A 5` to see all event definitions
3. Identify which events are most critical (message.updated, message.part.updated, permission.asked)
4. Create schemas incrementally, running `bun run generate:schemas` after each
5. Do verification as you go, not at the end

**Do not skip steps. Do not work from memory. Create explicit verification tables.**

---

## After Events: What's Next

Once event schemas are complete, the remaining WIP item is:

| File | Description | Status |
|------|-------------|--------|
| `09-opencode.md` | Main service aggregator | ⏳ WIP |

Update this prompt after completing events to point to the next item.

---

## Before Any Commit

**STOP. Do not run `git commit` until you have:**

1. Shown the user `git status` and `git diff --stat` output
2. Asked explicitly: "Do you want me to commit these changes?"
3. Received clear approval (e.g., "Yes", "Go ahead", "Commit it")

**If the user hasn't approved, DO NOT COMMIT.**

This applies to BOTH the submodule (`submodules/opencode`) AND the main repo.
