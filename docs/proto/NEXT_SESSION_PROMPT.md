# Next Session: Create JSON Schema for Message Types

## Your Task

Create JSON Schema definitions for message types by reading the **TypeScript/Zod source code** in the OpenCode repository, then update the protobuf documentation to reference these schemas.

---

## ⚠️ CRITICAL: Read This First

**Before doing ANY work, you MUST read:**

1. **`docs/proto/SCHEMA_DEVELOPMENT_PROCESS.md`** — The step-by-step process you MUST follow
2. **This prompt in full** — Contains specific guidance for message types

**The process document contains critical verification steps (4.2 and 5.3) that you MUST NOT skip.** These require explicit field-by-field comparison tables. Do not claim "verified" without creating these tables.

---

## Understanding the Goal

**The purpose of this work:**

```
TypeScript/Zod (SOURCE)  →  JSON Schema (YOU CREATE)  →  Protobuf Doc (YOU UPDATE)
         ↓                          ↓                            ↓
   Read this first           Write these files          Update to reference schemas
```

- **Source of truth:** TypeScript/Zod definitions in `submodules/opencode/packages/opencode/src/session/message-v2.ts`
- **What you create:** JSON Schema files in `submodules/opencode/schema/`
- **What you update:** Protobuf doc at `docs/proto/05-message.md` (THE ULTIMATE GOAL)

---

## Required Reading

| Priority | File | Purpose |
|----------|------|---------|
| 1 | `docs/proto/SCHEMA_DEVELOPMENT_PROCESS.md` | **THE PROCESS** — Follow every step |
| 2 | `docs/proto/05-message.md` | Proto doc you will update (Step 5) |
| 3 | `docs/proto/06-tool.md` | Completed example — reference for quality |
| 4 | `docs/proto/04-session.md` | Completed example — reference for quality |

---

## TypeScript Source Location

**All message types are in a single file:**

```
submodules/opencode/packages/opencode/src/session/message-v2.ts
```

**Key types to schematize:**

### Already Done (DO NOT recreate):
- `FilePart` — `filePart.schema.json` ✅
- `FilePartSource` — `filePartSource.schema.json` ✅
- `FileSource` — `fileSource.schema.json` ✅
- `SymbolSource` — `symbolSource.schema.json` ✅
- `ResourceSource` — `resourceSource.schema.json` ✅
- `FilePartSourceText` — `filePartSourceText.schema.json` ✅
- `ToolPart` — `toolPart.schema.json` ✅
- `ToolState*` — All tool state schemas ✅

### TODO (You create these):

| Type | Location | Description |
|------|----------|-------------|
| `PartBase` | lines 37-41 | Base schema for all parts (id, sessionID, messageID) |
| `TextPart` | lines 60-73 | Text content part |
| `ReasoningPart` | lines 75-85 | Reasoning/thinking content |
| `SnapshotPart` | lines 43-49 | Snapshot reference part |
| `PatchPart` | lines 51-58 | Patch/diff part |
| `AgentPart` | lines 143-155 | Agent reference part |
| `CompactionPart` | lines 157-163 | Compaction marker part |
| `SubtaskPart` | lines 165-172 | Subtask reference part |
| `RetryPart` | lines 174-184 | Retry marker with error |
| `StepStartPart` | lines 186-193 | Step start marker |
| `StepFinishPart` | lines 195-211 | Step finish with tokens/cost |
| `Part` | lines 288-305 | Discriminated union of ALL parts |
| `User` | lines 263-286 | User message |
| `Assistant` | lines 308-330+ | Assistant message |

**Note:** Line numbers reference commit `21dc3c24d`. The current file may have different line numbers due to refactoring. Use `git show 21dc3c24d:packages/opencode/src/session/message-v2.ts` to see the original.

---

## Schemas to Create

Based on the TypeScript source:

```
submodules/opencode/schema/
├── partBase.schema.json          # Optional - can inline fields instead
├── textPart.schema.json
├── reasoningPart.schema.json
├── snapshotPart.schema.json
├── patchPart.schema.json
├── agentPart.schema.json
├── compactionPart.schema.json
├── subtaskPart.schema.json
├── retryPart.schema.json
├── stepStartPart.schema.json
├── stepFinishPart.schema.json
├── part.schema.json              # Discriminated union (oneOf)
├── userMessage.schema.json
└── assistantMessage.schema.json
```

---

## Handling PartBase Extension

All parts extend `PartBase`:

```typescript
const PartBase = z.object({
  id: z.string(),
  sessionID: z.string(),
  messageID: z.string(),
})

export const TextPart = PartBase.extend({
  type: z.literal("text"),
  text: z.string(),
  // ...
})
```

**Two options:**

1. **Inline the fields** (simpler) — Each part schema includes `id`, `sessionID`, `messageID` directly
2. **Use `allOf`** (DRY but complex) — Reference a `partBase.schema.json`

**Recommendation:** Inline the fields. It's what was done for `toolPart.schema.json` and `filePart.schema.json`.

---

## Handling the Part Discriminated Union

The `Part` type uses Zod's `discriminatedUnion` on `type`:

```typescript
export const Part = z
  .discriminatedUnion("type", [
    TextPart,
    SubtaskPart,
    ReasoningPart,
    FilePart,
    ToolPart,
    StepStartPart,
    StepFinishPart,
    SnapshotPart,
    PatchPart,
    AgentPart,
    RetryPart,
    CompactionPart,
  ])
```

In JSON Schema:

```json
{
  "oneOf": [
    { "$ref": "textPart.schema.json" },
    { "$ref": "reasoningPart.schema.json" },
    ...
  ]
}
```

The generator will auto-detect the `type` discriminator and generate `z.discriminatedUnion()`.

---

## Handling Nested Types

Some parts have nested objects that may need their own schemas:

- `StepFinishPart.tokens` — Object with `input`, `output`, `reasoning`, `cache`
- `RetryPart.error` — References `APIError.Schema`
- `User.summary` — Optional object with `title`, `body`, `diffs`
- `User.model` — Object with `providerID`, `modelID`
- `Assistant.time` — Object with `created`, `completed?`
- `Assistant.error` — Discriminated union of error types

**Decision point:** Create separate schemas for these, or inline them. For complex types used in multiple places, create separate schemas.

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
2. Read the protobuf message you wrote in `05-message.md`
3. Create an explicit field-by-field comparison table
4. Verify field counts match
5. Verify types match (using the mapping table in SCHEMA_DEVELOPMENT_PROCESS.md)
6. Verify required/optional matches

**Do NOT claim "verified" without showing the comparison tables.**

---

## Expected Deliverables

### 1. New Schema Files

All schemas listed in "Schemas to Create" section above.

### 2. Refactored TypeScript

Replace inline Zod definitions in `message-v2.ts` with imports from `@generated/validators/*`:

```typescript
// Before
export const TextPart = PartBase.extend({
  type: z.literal("text"),
  text: z.string(),
  // ... 10+ lines
})

// After
import { textPartSchema, type TextPart as GeneratedTextPart } from "@generated/validators/textPart"

export const TextPart = textPartSchema
export type TextPart = GeneratedTextPart
```

### 3. Updated Proto Doc

Update `docs/proto/05-message.md`:
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

## Lessons from Previous Session

The following mistakes were made in the tool schema session. **Do not repeat them:**

1. **Skipped verification steps** — Claimed "verified" without creating comparison tables
2. **Worked from memory** — Did not re-read files before making claims
3. **Read wrong prompt** — Read `NEXT_SESSION_PROMPT.md` from repo root instead of `docs/proto/`
4. **Skipped build step** — Did not run `bun run build` before claiming completion
5. **Marked complete prematurely** — Updated todo status before verification was done

**The guardrails in SCHEMA_DEVELOPMENT_PROCESS.md exist because of these failures. Follow them.**

---

## Starting Point

1. Read `docs/proto/SCHEMA_DEVELOPMENT_PROCESS.md` in full
2. Read the TypeScript source: `git show 21dc3c24d:packages/opencode/src/session/message-v2.ts`
3. Start with simpler types first (TextPart, ReasoningPart) before complex ones (Part, User, Assistant)
4. Create schemas incrementally, running `bun run generate:schemas` after each batch
5. Do verification as you go, not at the end

**Do not skip steps. Do not work from memory. Create explicit verification tables.**
