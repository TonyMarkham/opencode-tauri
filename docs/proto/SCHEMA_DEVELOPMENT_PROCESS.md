# Schema Development Process

This document describes the process for creating JSON Schema definitions and updating protobuf schemas for OpenCode API types. Follow this process when adding schema support for any of the WIP proto files (auth, session, message, tool, agent, event).

---

## Quick Reference

**The only commands you need:**

```bash
# After creating/editing schema/*.schema.json files:
cd submodules/opencode
bun run generate:schemas   # Validate + generate
bun run typecheck          # Verify types
bun test                   # Run tests (optional)
```

**The only files you create:**
```
submodules/opencode/schema/<typeName>.schema.json
```

**The only docs you update:**
```
docs/proto/XX-name.md      # Update source of truth section
docs/proto/README.md       # Update status table
```

Everything else is already configured.

---

## What's Already Done (Don't Recreate)

The following infrastructure is **already in place** from the model/provider work:

| Component | Location | Purpose |
|-----------|----------|---------|
| Custom generator | `script/generate-from-schemas.ts` | Validates schemas + generates Zod validators |
| Build command | `bun run generate:schemas` | Runs the generator |
| Output directory | `packages/opencode/generated/validators/` | Where validators are written |
| Path alias | `@generated/*` in `tsconfig.json` | Import alias for generated code |
| Turborepo caching | Configured | Fast regeneration |

**You do NOT need to:**
- Create or modify the generator script
- Change build configuration
- Modify tsconfig paths
- Update Turborepo config

---

## Understanding the Workflow Direction

**This is critical — do not get this backwards:**

```
TypeScript/Zod (SOURCE)  →  JSON Schema (CREATE)  →  Protobuf Doc (UPDATE)
         ↓                          ↓                          ↓
   Read this first           Write these files       Update to reference schemas
```

- **Source of truth:** TypeScript/Zod definitions in `submodules/opencode/packages/opencode/src/`
- **What you create:** JSON Schema files in `submodules/opencode/schema/`
- **What you update:** Protobuf docs at `docs/proto/XX-name.md`

**The protobuf doc is NOT the source.** It's documentation that you update AFTER creating schemas from the TypeScript source. The ultimate goal is Step 5 (updating the proto doc to reference the new schemas).

---

## What You Actually Do

For each new schema domain (auth, session, message, etc.):

```
1. Read TypeScript source       → Understand the Zod types (THIS IS YOUR SOURCE)
2. Write JSON Schema files      → Create schema/*.schema.json (TRANSLATE FROM TYPESCRIPT)
3. Fresh Eyes Review (CRITICAL) → Compare schema to TypeScript field-by-field
4. Generate and verify          → bun run generate:schemas (creates validators)
5. Refactor TypeScript (NEW)    → Replace inline Zod with imports from @generated/validators
6. Run typecheck + tests        → Verify refactored code works
7. Update proto markdown        → docs/proto/XX-name.md (THE GOAL — reference new schemas)
```

**Steps 1-6 are preparation. Step 7 is the deliverable.**

The tooling handles the rest.

---

## Prerequisites

- OpenCode submodule at `submodules/opencode`
- Bun installed
- Access to run `bun run generate:schemas`

---

## Step 1: Identify Source TypeScript

Find the authoritative Zod definitions in the OpenCode source code.

**Location pattern:**
```
submodules/opencode/packages/opencode/src/<domain>/<domain>.ts
```

**Example for Session:**
```bash
# Find session-related types
rg "export const.*Session" submodules/opencode/packages/opencode/src/
rg "z\.object" submodules/opencode/packages/opencode/src/session/
```

**Document what you find:**
- File path
- Line numbers
- Commit hash (for traceability)

---

## Step 2: Create JSON Schema Files

### 2.1 Schema File Naming

```
submodules/opencode/schema/<typeName>.schema.json
```

Examples:
- `sessionInfo.schema.json`
- `messageContent.schema.json`
- `toolCallState.schema.json`

### 2.2 Schema Template

```json
{
  "$schema": "http://json-schema.org/draft-07/schema#",
  "$id": "https://opencode.ai/schemas/v1/<typeName>.json",
  "$comment": "Source: packages/opencode/src/<path>.ts @ <commit> (<date>) lines <start>-<end>",
  "title": "<TypeName>",
  "description": "<Brief description of what this type represents>",
  "type": "object",
  "required": ["field1", "field2"],
  "properties": {
    "field1": {
      "type": "string",
      "description": "Description of field1"
    },
    "field2": {
      "$ref": "otherType.schema.json"
    }
  }
}
```

### 2.3 Type Mapping: Zod → JSON Schema

| Zod | JSON Schema |
|-----|-------------|
| `z.string()` | `{ "type": "string" }` |
| `z.number()` | `{ "type": "number" }` |
| `z.boolean()` | `{ "type": "boolean" }` |
| `z.array(z.string())` | `{ "type": "array", "items": { "type": "string" } }` |
| `z.object({...})` | `{ "type": "object", "properties": {...} }` |
| `z.enum(["a", "b"])` | `{ "type": "string", "enum": ["a", "b"] }` |
| `z.optional(z.string())` | `{ "type": "string" }` (not in `required`) |
| `z.record(z.string(), z.any())` | `{ "type": "object", "additionalProperties": true }` |
| `z.union([z.boolean(), z.object({...})])` | `{ "oneOf": [{...}, {...}] }` |

### 2.4 Cross-File References

Use relative file paths for `$ref`:

```json
{
  "properties": {
    "capabilities": {
      "$ref": "modelCapabilities.schema.json"
    }
  }
}
```

---

## Step 3: Fresh Eyes Review (CRITICAL)

> **⚠️ DO NOT SKIP THIS STEP ⚠️**
>
> This review is critically important. No shortcuts. No guesses. Be diligent.
> Compare EVERY field between the JSON Schema and the TypeScript source.

### 3.1 Side-by-Side Comparison

Open both files and compare field-by-field:

1. **TypeScript source:** `submodules/opencode/packages/opencode/src/<path>.ts`
2. **Your JSON Schema:** `submodules/opencode/schema/<typeName>.schema.json`

### 3.2 Verification Checklist

For EACH field in the TypeScript Zod definition, verify:

- [ ] Field exists in JSON Schema with correct name
- [ ] Type matches exactly (string, number, boolean, array, object)
- [ ] Required vs optional matches (`z.optional()` → not in `required` array)
- [ ] Enum values match exactly (same strings, same order)
- [ ] Nested objects have correct `$ref` to other schemas
- [ ] Array item types are correct
- [ ] Default values documented (if any)

### 3.3 Common Mistakes to Check

| Mistake | How to Catch |
|---------|--------------|
| Missing field | Count fields in both files — must match |
| Wrong optionality | Check `required` array vs Zod `.optional()` |
| Typo in field name | Character-by-character comparison |
| Wrong `$ref` path | Verify referenced file exists |
| Missing enum value | Compare enum arrays exactly |
| Wrong type | `z.number()` → `"type": "number"`, not `"integer"` |

### 3.4 Read the Generated Validator

After generation, read the output:

```bash
cat submodules/opencode/packages/opencode/generated/validators/<typeName>.ts
```

Compare the generated Zod schema back to the original TypeScript. They should be functionally equivalent.

**If anything looks wrong, STOP. Fix the JSON Schema. Re-run generation.**

---

## Step 4: Generate and Verify Equivalence

### 4.1 Run the Generator

```bash
cd submodules/opencode
bun run generate:schemas
```

**Expected output:**
```
🚀 JSON Schema Validation & Code Generation

🔍 Validating JSON Schemas...
✓ yourNewSchema.schema.json - valid
...
✓ All N schemas validated successfully

🔧 Generating Zod validators...
✓ yourNewSchema.ts - generated
...
✓ Code generation complete

✨ Done!
```

**If validation fails:** Fix the JSON Schema syntax errors shown in the output.

**If generation fails:** Usually a `$ref` path issue — check the referenced file exists.

### 4.2 Verify Generated Zod Matches Original Zod

> **⚠️ THIS IS THE PROOF STEP ⚠️**
>
> The generated Zod validator MUST be functionally equivalent to the original TypeScript Zod definition.

**Open both files side-by-side:**

1. **Original:** `submodules/opencode/packages/opencode/src/<domain>/<file>.ts`
2. **Generated:** `submodules/opencode/packages/opencode/generated/validators/<typeName>.ts`

**Compare line-by-line:**

| Check | Original Zod | Generated Zod | Must Match |
|-------|--------------|---------------|------------|
| Field names | `fieldName: z.string()` | `fieldName: z.string()` | ✓ Exact |
| Types | `z.number()` | `z.number()` | ✓ Exact |
| Optional | `z.string().optional()` | `z.string().optional()` | ✓ Exact |
| Enums | `z.enum(["a", "b"])` | `z.enum(["a", "b"])` | ✓ Same values |
| Arrays | `z.array(z.string())` | `z.array(z.string())` | ✓ Same item type |
| Nested | `z.object({...})` | References other schema | ✓ Same structure |

**If they don't match:**
1. STOP
2. Identify the discrepancy
3. Fix the JSON Schema
4. Re-run `bun run generate:schemas`
5. Compare again

**Do not proceed until generated Zod matches original Zod.**

### 4.3 Run Typecheck (Before Refactoring)

```bash
cd submodules/opencode
bun run typecheck
```

All packages should pass. If typecheck fails, the generated types don't integrate correctly.

### 4.4 Refactor TypeScript Source to Use Generated Validators

> **⚠️ CRITICAL STEP - DO NOT SKIP ⚠️**
>
> This is what makes JSON Schema the single source of truth. Without this step,
> you have TWO sources of truth (inline Zod + JSON Schema) that will drift apart.

**Example refactoring** (based on model/provider work in commit 7c7c8475f):

**Before (inline Zod - 33 lines):**
```typescript
export namespace Auth {
  export const Oauth = z.object({
    type: z.literal("oauth"),
    refresh: z.string(),
    access: z.string(),
    expires: z.number(),
    enterpriseUrl: z.string().optional(),
  }).meta({ ref: "OAuth" })

  export const Api = z.object({
    type: z.literal("api"),
    key: z.string(),
  }).meta({ ref: "ApiAuth" })

  export const WellKnown = z.object({
    type: z.literal("wellknown"),
    key: z.string(),
    token: z.string(),
  }).meta({ ref: "WellKnownAuth" })

  export const Info = z.discriminatedUnion("type", [Oauth, Api, WellKnown]).meta({ ref: "Auth" })
  export type Info = z.infer<typeof Info>
}
```

**After (using generated validators - 11 lines):**
```typescript
import { authSchema, type Auth } from "@generated/validators/auth"
import { oauthSchema, type Oauth } from "@generated/validators/oauth"
import { apiAuthSchema, type ApiAuth } from "@generated/validators/apiAuth"
import { wellKnownAuthSchema, type WellKnownAuth } from "@generated/validators/wellKnownAuth"

export namespace Auth {
  // Generated from JSON Schema - see schema/oauth.schema.json
  export const Oauth = oauthSchema
  
  // Generated from JSON Schema - see schema/apiAuth.schema.json
  export const Api = apiAuthSchema
  
  // Generated from JSON Schema - see schema/wellKnownAuth.schema.json
  export const WellKnown = wellKnownAuthSchema
  
  // Generated from JSON Schema - see schema/auth.schema.json
  export const Info = authSchema
  export type Info = Auth
}
```

**Steps:**

1. Add imports at top of file from `@generated/validators/<typeName>`
2. Replace inline Zod definitions with references to generated schemas
3. Keep export names the same (maintains backwards compatibility)
4. Add comments referencing the JSON Schema source files
5. Delete all inline Zod definitions

**For each schema:**
- Import both the schema validator AND the TypeScript type
- Re-export the validator with the original export name
- Re-export the type with the original type name

### 4.5 Run Typecheck (After Refactoring)

```bash
cd submodules/opencode
bun run typecheck
```

All packages should pass. This now validates that:
- The generated validators can be imported correctly
- The refactored code type-checks properly
- No breaking changes were introduced

**If typecheck fails after refactoring:**
1. Check import paths are correct (`@generated/validators/<typeName>`)
2. Verify export names match the original names
3. Ensure TypeScript types are imported and re-exported

### 4.6 Run Tests

```bash
cd submodules/opencode/packages/opencode
bun test
```

All 544+ tests should pass. Zero behavior change expected.

**If tests fail after refactoring:**
- The generated validators may not be functionally equivalent to the original Zod
- Go back to Step 4.2 and compare more carefully
- Check for subtle differences (discriminatedUnion vs union, optional fields, etc.)

### 4.7 Build

```bash
cd submodules/opencode/packages/opencode
bun run build
```

Confirms the generated code works in a production build across all 11 platform targets.

---

## Step 5: Update Protobuf Definition with 1:1 Cross-Reference

> **⚠️ EVERY PROTOBUF FIELD MUST MAP TO A JSON SCHEMA PROPERTY ⚠️**
>
> This is not just updating text. You must verify each protobuf field has a 
> corresponding JSON Schema property with matching type and optionality.

Update the corresponding `docs/proto/XX-name.md` file:

### 5.1 Update Source of Truth Section

**Before:**
```markdown
## Source of Truth

**Note:** No dedicated JSON Schema yet for <type> types.
```

**After:**
```markdown
## Source of Truth

**JSON Schema (canonical):**

- `submodules/opencode/schema/<typeName>.schema.json` - <description>
- `submodules/opencode/schema/<relatedType>.schema.json` - <description>

**Previously derived from (now superseded):**

- `packages/opencode/src/<path>.ts` @ `<commit>` lines <start>-<end>
```

### 5.2 Update Protobuf Comments

Add source references to each message:

```protobuf
// <TypeName>
// Source: submodules/opencode/schema/<typeName>.schema.json (canonical)
message TypeName {
  ...
}
```

### 5.3 Verify 1:1 Field Mapping (REQUIRED)

For EACH message in the protobuf, create a verification table:

**Open both files side-by-side:**
1. **JSON Schema:** `submodules/opencode/schema/<typeName>.schema.json`
2. **Protobuf Doc:** `docs/proto/XX-name.md`

**Verify every field:**

| # | JSON Schema Property | Type | Required | Protobuf Field | Type | Optional | Match? |
|---|---------------------|------|----------|----------------|------|----------|--------|
| 1 | `id` | string | ✓ | `string id = 1` | string | no | ✓ |
| 2 | `name` | string | ✓ | `string name = 2` | string | no | ✓ |
| 3 | `description` | string | no | `optional string description = 3` | string | yes | ✓ |
| ... | ... | ... | ... | ... | ... | ... | ... |

**Rules:**
- Every JSON Schema property MUST have a protobuf field
- Every protobuf field MUST have a JSON Schema property
- Types must be equivalent (see mapping table below)
- Required in JSON Schema → NOT optional in protobuf
- Not in `required` array → `optional` in protobuf

### 5.4 Type Mapping: JSON Schema → Protobuf

| JSON Schema | Protobuf | Notes |
|-------------|----------|-------|
| `"type": "string"` | `string` | |
| `"type": "number"` | `double` | Use double, not float |
| `"type": "integer"` | `int32` or `int64` | |
| `"type": "boolean"` | `bool` | |
| `"type": "array"` | `repeated <type>` | |
| `"type": "object"` with properties | `message` | Define nested message |
| `"type": "object"` with `additionalProperties` | `map<string, type>` or `google.protobuf.Struct` | |
| `"enum": [...]` | `enum` | Define enum type |
| `"$ref": "other.schema.json"` | `OtherMessage` | Import and reference |
| `"oneOf": [...]` | `oneof` | Union type |

### 5.5 Naming Convention Mapping

| JSON Schema (camelCase) | Protobuf (snake_case) |
|------------------------|----------------------|
| `providerID` | `provider_id` |
| `baseURL` | `base_url` |
| `apiKey` | `api_key` |
| `createdAt` | `created_at` |

**If any field doesn't map 1:1, STOP and fix the discrepancy.**

---

## Step 6: Add Cross-Reference Table to Documentation

Add a permanent cross-reference table to the proto markdown file:

```markdown
## JSON Schema Cross-Reference

| Protobuf Field | JSON Schema Property | Type | Notes |
|----------------|---------------------|------|-------|
| `id` | `id` | string | Required |
| `provider_id` | `providerID` | string | Naming: snake_case vs camelCase |
| `options` | `options` | Struct | `additionalProperties: true` → flexible |
| `models` | `models` | map | `additionalProperties` with `$ref` |
```

This table serves as permanent documentation of the mapping.

---

## Step 7: Mark Complete

### 7.1 Update Proto Doc Status

In `docs/proto/XX-name.md`:

```markdown
**Status:** ✅ Complete
```

### 7.2 Update README Status Table

In `docs/proto/README.md`:

```markdown
| `<typeName>.schema.json` | `<name>.proto` | ✅ Complete |
```

### 7.3 Remove TODO Section

Delete the TODO section from the proto doc (or mark items complete).

---

## Commit Convention

Use this commit message format:

```
feat: add JSON Schema for <domain> types

- Create <typeName>.schema.json (and related schemas)
- Update docs/proto/XX-name.md with JSON Schema source
- Validate with bun run generate:schemas
- All tests pass

Part of JSON Schema as source of truth initiative
```

---

## Checklist Template

Copy this checklist when starting a new schema:

```markdown
## Schema Development: <DomainName>

### Preparation
- [ ] Identify source TypeScript file(s)
- [ ] Document commit hash and line numbers
- [ ] List all types to be schematized

### Schema Creation
- [ ] Create `<typeName>.schema.json`
- [ ] Add all required fields
- [ ] Add all optional fields
- [ ] Add cross-file `$ref` references
- [ ] Add `$comment` with source reference

### Fresh Eyes Review (DO NOT SKIP)
- [ ] Open TypeScript source and JSON Schema side-by-side
- [ ] Count fields — must match exactly
- [ ] Verify each field name character-by-character
- [ ] Verify each field type matches
- [ ] Verify required vs optional matches
- [ ] Verify enum values match exactly
- [ ] Verify `$ref` paths are correct
- [ ] Read generated validator and compare to original Zod

### Validation
- [ ] `bun run generate:schemas` passes
- [ ] `bun run typecheck` passes (before refactoring)
- [ ] Refactor TypeScript source to use generated validators
- [ ] Delete inline Zod definitions
- [ ] Import from `@generated/validators/*`
- [ ] Maintain backwards compatibility (same export names)
- [ ] `bun run typecheck` passes (after refactoring)
- [ ] `bun test` passes (544+ tests)
- [ ] `bun run build` passes (optional)

### Protobuf 1:1 Cross-Reference (DO NOT SKIP)
- [ ] Open JSON Schema and protobuf doc side-by-side
- [ ] Create verification table for each message
- [ ] Verify EVERY JSON property has a protobuf field
- [ ] Verify EVERY protobuf field has a JSON property
- [ ] Verify types match (using type mapping table)
- [ ] Verify required/optional matches
- [ ] Verify naming convention mapping (camelCase ↔ snake_case)
- [ ] Add permanent cross-reference table to proto doc

### Documentation
- [ ] Update `docs/proto/XX-name.md` source of truth section
- [ ] Update `docs/proto/README.md` status table
- [ ] Change status to ✅ Complete

### Commit
- [ ] Commit with proper message format
```

---

## Example: Session Schema Development

**Target:** `docs/proto/04-session.md`

### Step 1: Find Source

```bash
rg "SessionInfo|TabInfo" submodules/opencode/packages/opencode/src/session/
```

### Step 2: Create Schemas

Files to create:
- `sessionInfo.schema.json`
- `sessionTime.schema.json`
- `tabInfo.schema.json`
- `sessionList.schema.json`

### Step 3: Validate

```bash
cd submodules/opencode
bun run generate:schemas
bun run typecheck
bun test
```

### Step 4-6: Update Docs

Update `docs/proto/04-session.md` and `docs/proto/README.md`.

---

## Reference

- **Existing complete schemas:** `01-model.md`, `02-provider.md`
- **JSON Schema spec:** https://json-schema.org/draft-07/schema
- **Generator script:** `submodules/opencode/script/generate-from-schemas.ts`
- **GitHub Issue:** [anomalyco/opencode#6879](https://github.com/anomalyco/opencode/issues/6879)
