# Next Session: Create JSON Schema for Session Types

## Your Task

Create JSON Schema definitions for session/tab types by reading the **TypeScript/Zod source code** in the OpenCode repository, then update the protobuf documentation to reference these schemas.

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
- **What you update:** Protobuf doc at `docs/proto/04-session.md` (Step 5 — the ultimate goal)

The proto doc (`04-session.md`) is NOT the source — it's the documentation you update AFTER creating schemas from the TypeScript source.

---

## Required Reading (Read These First)

1. **Process Guide:** `docs/proto/SCHEMA_DEVELOPMENT_PROCESS.md` — Follow this step-by-step
2. **Proto Doc to Update:** `docs/proto/04-session.md` — Update this AFTER creating schemas (Step 5)
3. **Completed Examples:** `docs/proto/01-model.md`, `docs/proto/02-provider.md`, `docs/proto/03-auth.md` — Reference for expected output quality

---

## Context

- The JSON Schema infrastructure is already in place (generator, build command, paths)
- You are NOT creating new tooling — just adding schema files
- The generator is at `submodules/opencode/script/generate-from-schemas.ts`
- Run `bun run generate:schemas` to validate and generate
- Model, Provider, and Auth schemas are complete — you can reference them with `$ref`

---

## Schemas to Create

Based on `04-session.md`, you need to create schemas for:

1. `sessionInfo.schema.json` — Core session identity and metadata
2. `sessionTime.schema.json` — Created/updated timestamps
3. `tabInfo.schema.json` — Client-side tab state (session + UI selections)
4. `sessionList.schema.json` — List of sessions (API response)

**Note:** `TabInfo` references `ModelSelection` from `modelSelection.schema.json` (already exists).

---

## Steps to Follow

1. **Find the TypeScript source** — Look in `submodules/opencode/packages/opencode/src/session/` for session-related Zod definitions. **This is your source of truth.**
2. **Create JSON Schema files** — In `submodules/opencode/schema/`, translating from the TypeScript/Zod you found
3. **Fresh Eyes Review** — Compare schema to TypeScript field-by-field (DO NOT SKIP)
4. **Run generator** — `bun run generate:schemas`
5. **Verify equivalence** — Compare generated Zod to original TypeScript Zod
6. **Update proto doc (THE GOAL)** — Update `docs/proto/04-session.md` to reference the new JSON Schemas, add 1:1 cross-reference table
7. **Mark complete** — Update status in `docs/proto/README.md`

**Steps 1-5 are preparation. Step 6 is the deliverable.**

---

## Critical Reminders

- **No shortcuts, no guesses** — Every field must be verified
- **Read the TypeScript source** — Don't assume, read the actual code
- **1:1 mapping required** — Every JSON property ↔ every protobuf field
- **Run all verification steps** — `generate:schemas`, `typecheck`, `test`
- **If something doesn't match, STOP and fix it**

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
   - `sessionInfo.schema.json`
   - `sessionTime.schema.json`
   - `tabInfo.schema.json`
   - `sessionList.schema.json`

2. Updated `docs/proto/04-session.md`:
   - Source of Truth section pointing to JSON Schemas
   - JSON Schema Cross-Reference table
   - Status changed to ✅ Complete

3. Updated `docs/proto/README.md`:
   - Status table updated for session.proto

4. All verification passing:
   - `bun run generate:schemas` ✓
   - `bun run typecheck` ✓
   - `bun test` ✓

---

## Likely Source Locations (START HERE)

**Your first task is to find and read the TypeScript/Zod definitions:**

```
submodules/opencode/packages/opencode/src/session/
submodules/opencode/packages/opencode/src/session/index.ts
submodules/opencode/packages/opencode/src/server/server.ts  (API response types)
```

Use `rg` to search:
```bash
rg "SessionInfo|TabInfo|z\.object" submodules/opencode/packages/opencode/src/session/
```

**Read these files carefully. The Zod definitions you find are what you translate to JSON Schema.**

---

## Cross-References to Existing Schemas

When a field references an existing type, use `$ref`:

```json
{
  "selected_model": {
    "$ref": "modelSelection.schema.json"
  }
}
```

Existing schemas you may reference:
- `modelSelection.schema.json` — For TabInfo.selected_model
- `providerInfo.schema.json` — If provider context needed

---

**Start by reading `SCHEMA_DEVELOPMENT_PROCESS.md`, then proceed with Step 1: Find the TypeScript source.**
