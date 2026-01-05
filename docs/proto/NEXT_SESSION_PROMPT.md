# Next Session: Create JSON Schema for Tool Types

## Your Task

Create JSON Schema definitions for tool execution types by reading the **TypeScript/Zod source code** in the OpenCode repository, then update the protobuf documentation to reference these schemas.

---

## Understanding the Goal

**The purpose of this work:**

```
TypeScript/Zod (SOURCE)  →  JSON Schema (YOU CREATE)  →  Protobuf Doc (YOU UPDATE)
         ↓                          ↓                            ↓
   Read this first           Write these files          Update to reference schemas
```

- **Source of truth:** TypeScript/Zod definitions in `submodules/opencode/packages/opencode/src/`
- **What you create:** JSON Schema files in `submodules/opencode/schema/`
- **What you update:** Protobuf doc at `docs/proto/06-tool.md` (Step 5 — the ultimate goal)

---

## Required Reading (Read These First)

1. **Process Guide:** `docs/proto/SCHEMA_DEVELOPMENT_PROCESS.md` — Follow this step-by-step
2. **Proto Doc to Update:** `docs/proto/06-tool.md` — Update this AFTER creating schemas (Step 5)
3. **Completed Examples:** `docs/proto/01-model.md`, `docs/proto/04-session.md` — Reference for expected output quality

---

## CRITICAL: Actual TypeScript Source Locations

**The tool types are NOT in a separate tool file. They are in `message-v2.ts`:**

```
submodules/opencode/packages/opencode/src/session/message-v2.ts
```

**Key types to schematize (lines 214-291):**

1. `ToolStatePending` (lines 214-224) — Status: pending, has input/raw
2. `ToolStateRunning` (lines 226-239) — Status: running, has input/title/metadata/time.start
3. `ToolStateCompleted` (lines 241-258) — Status: completed, has input/output/title/metadata/time/attachments
4. `ToolStateError` (lines 260-274) — Status: error, has input/error/metadata/time
5. `ToolState` (lines 276-280) — Discriminated union of the above 4 states
6. `ToolPart` (lines 282-291) — Tool call part with callID, tool name, state

**Permission types are in `permission/next.ts` (lines 52-74):**

7. `PermissionRequest` (lines 52-69) — Permission request with patterns/metadata/tool context
8. `Reply` (line 73) — Enum: "once", "always", "reject"

---

## Important Discovery: Proto Doc Mismatch

The current `06-tool.md` proto definition does NOT match the TypeScript source:

| Proto Doc Field | TypeScript Equivalent | Notes |
|-----------------|----------------------|-------|
| `status = 3` (string) | `status` (literal union) | TS uses discriminated union, not string |
| `input_json = 5` | `input` (z.record) | TS uses typed record, not JSON string |
| `metadata_json = 9` | `metadata` (z.record) | TS uses typed record, not JSON string |
| `logs = 8` | (not present) | Proto has logs, TS doesn't |
| `call_id = 4` | `callID` in ToolPart | Different location |

**You will need to update the proto doc to match the TypeScript source, not the other way around.**

---

## Schemas to Create

Based on the TypeScript source, create these schemas:

1. `toolStatePending.schema.json` — Pending tool state
2. `toolStateRunning.schema.json` — Running tool state  
3. `toolStateCompleted.schema.json` — Completed tool state
4. `toolStateError.schema.json` — Error tool state
5. `toolState.schema.json` — Discriminated union (oneOf)
6. `toolPart.schema.json` — Tool call part (extends PartBase)
7. `permissionRequest.schema.json` — Permission request
8. `permissionReply.schema.json` — Reply enum

**Note:** ToolPart references FilePart (already exists in session schemas).

---

## Steps to Follow

1. **Read the TypeScript source** — `message-v2.ts` lines 214-291 and `permission/next.ts` lines 52-74
2. **Create JSON Schema files** — In `submodules/opencode/schema/`
3. **Fresh Eyes Review** — Compare schema to TypeScript field-by-field (DO NOT SKIP)
4. **Run generator** — `bun run generate:schemas`
5. **Verify equivalence** — Compare generated Zod to original TypeScript Zod
6. **Refactor TypeScript** — Replace inline Zod with generated validators
7. **Run typecheck + tests** — Verify refactored code works
8. **Update proto doc (THE GOAL)** — Update `docs/proto/06-tool.md` to match TypeScript source and reference JSON Schemas

---

## Handling Discriminated Unions

The `ToolState` type uses Zod's `discriminatedUnion` on `status`:

```typescript
export const ToolState = z
  .discriminatedUnion("status", [ToolStatePending, ToolStateRunning, ToolStateCompleted, ToolStateError])
```

In JSON Schema, use `oneOf`:

```json
{
  "oneOf": [
    { "$ref": "toolStatePending.schema.json" },
    { "$ref": "toolStateRunning.schema.json" },
    { "$ref": "toolStateCompleted.schema.json" },
    { "$ref": "toolStateError.schema.json" }
  ]
}
```

---

## Handling PartBase Extension

Tool parts extend a base schema with common fields:

```typescript
const PartBase = z.object({
  id: z.string(),
  sessionID: z.string(),
  messageID: z.string(),
})

export const ToolPart = PartBase.extend({
  type: z.literal("tool"),
  // ...
})
```

You may want to create `partBase.schema.json` for reuse, or inline the fields.

---

## Commands You'll Use

```bash
cd submodules/opencode
bun run generate:schemas   # Validate + generate
bun run typecheck          # Verify types
bun test                   # Run tests (544+ should pass)
```

---

## Expected Deliverables

1. New schema files in `submodules/opencode/schema/`:
   - `toolStatePending.schema.json`
   - `toolStateRunning.schema.json`
   - `toolStateCompleted.schema.json`
   - `toolStateError.schema.json`
   - `toolState.schema.json`
   - `toolPart.schema.json`
   - `permissionRequest.schema.json`
   - `permissionReply.schema.json`

2. Updated `docs/proto/06-tool.md`:
   - Proto definition updated to match TypeScript source
   - Source of Truth section pointing to JSON Schemas
   - JSON Schema Cross-Reference table
   - Status changed to ✅ Complete

3. Refactored TypeScript:
   - `message-v2.ts` tool types use generated validators
   - `permission/next.ts` Request/Reply use generated validators

4. All verification passing:
   - `bun run generate:schemas` ✓
   - `bun run typecheck` ✓
   - `bun test` ✓

---

## Cross-References to Existing Schemas

When a field references an existing type, use `$ref`:

```json
{
  "attachments": {
    "type": "array",
    "items": { "$ref": "filePart.schema.json" }
  }
}
```

Note: FilePart schema may need to be created as part of message schemas (check if it exists).

---

**Start by reading `SCHEMA_DEVELOPMENT_PROCESS.md`, then read the TypeScript source at `message-v2.ts` lines 214-291.**
