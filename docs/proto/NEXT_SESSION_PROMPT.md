# Next Session: Create JSON Schema for Agent Types

## Your Task

Create JSON Schema definitions for agent types by reading the **TypeScript/Zod source code** in the OpenCode repository, then update the protobuf documentation to reference these schemas.

---

## ⚠️ CRITICAL: Read This First

**Before doing ANY work, you MUST read:**

1. **`docs/proto/SCHEMA_DEVELOPMENT_PROCESS.md`** — The step-by-step process you MUST follow
2. **This prompt in full** — Contains specific guidance for agent types

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
- **What you update:** Protobuf doc at `docs/proto/07-agent.md` (THE ULTIMATE GOAL)

---

## Required Reading

| Priority | File | Purpose |
|----------|------|---------|
| 1 | `docs/proto/SCHEMA_DEVELOPMENT_PROCESS.md` | **THE PROCESS** — Follow every step |
| 2 | `docs/proto/07-agent.md` | Proto doc you will update (Step 5) |
| 3 | `docs/proto/05-message.md` | Completed example — reference for quality |
| 4 | `docs/proto/06-tool.md` | Completed example — reference for quality |

---

## TypeScript Source Locations

**Find the agent types:**

```bash
cd submodules/opencode
rg "export.*Agent" packages/opencode/src/ --type ts -l
```

**Key files to investigate:**

- `packages/opencode/src/acp/agent.ts` - Main agent definitions
- `packages/opencode/src/config/config.ts` - Agent configuration
- `packages/opencode/src/tool/registry.ts` - Agent registry

---

## Expected Types to Schematize

Based on the current `07-agent.md` proto doc:

| Type | Description |
|------|-------------|
| `AgentInfo` | Agent metadata (name, description, mode, color, built_in) |
| `AgentList` | List of agents |

**Note:** The actual TypeScript source may have more or different types. Read the source first.

---

## Schemas to Create

Based on investigation of the TypeScript source:

```
submodules/opencode/schema/
├── agentInfo.schema.json       # Agent metadata
└── agentList.schema.json       # List of agents (if needed)
```

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
2. Read the protobuf message you wrote in `07-agent.md`
3. Create an explicit field-by-field comparison table
4. Verify field counts match
5. Verify types match (using the mapping table in SCHEMA_DEVELOPMENT_PROCESS.md)
6. Verify required/optional matches

**Do NOT claim "verified" without showing the comparison tables.**

---

## Expected Deliverables

### 1. New Schema Files

All agent-related schemas created in `submodules/opencode/schema/`.

### 2. Refactored TypeScript (if applicable)

If inline Zod definitions exist, replace them with imports from `@generated/validators/*`.

### 3. Updated Proto Doc

Update `docs/proto/07-agent.md`:
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

**The guardrails in SCHEMA_DEVELOPMENT_PROCESS.md exist because of these failures. Follow them.**

---

## Starting Point

1. Read `docs/proto/SCHEMA_DEVELOPMENT_PROCESS.md` in full
2. Find and read the TypeScript source for agent types
3. Create schemas incrementally, running `bun run generate:schemas` after each batch
4. Do verification as you go, not at the end

**Do not skip steps. Do not work from memory. Create explicit verification tables.**

---

## After Agent: What's Next

Once agent schemas are complete, the remaining WIP items are:

| File | Description | Status |
|------|-------------|--------|
| `08-event.md` | SSE event streaming | ⏳ WIP |
| `09-opencode.md` | Main service aggregator | ⏳ WIP |

Update this prompt after completing agent to point to the next item.

---

## Before Any Commit

**STOP. Do not run `git commit` until you have:**

1. Shown the user `git status` and `git diff --stat` output
2. Asked explicitly: "Do you want me to commit these changes?"
3. Received clear approval (e.g., "Yes", "Go ahead", "Commit it")

**If the user hasn't approved, DO NOT COMMIT.**

This applies to BOTH the submodule (`submodules/opencode`) AND the main repo.
