# Architecture Decision Records (ADRs)

This directory contains Architecture Decision Records (ADRs) documenting significant architectural decisions made during the development of the OpenCode Tauri-Blazor desktop client.

---

## What is an ADR?

An Architecture Decision Record (ADR) is a document that captures an important architectural decision along with its context and consequences. ADRs help maintain historical context for why certain technical choices were made.

### ADR Format

Each ADR includes:
- **Status:** Proposed, Accepted, Deprecated, Superseded
- **Context:** The situation and forces at play
- **Decision:** The architectural choice made
- **Consequences:** The results of the decision (positive, negative, risks)
- **Alternatives:** Other options considered and why they were rejected

---

## Universal Constraints

These are **absolute requirements** that apply to ALL architecture decisions:

| Document | Title | Status | Date | Summary |
|----------|-------|--------|------|---------|
| [NO_CUSTOM_JAVASCRIPT_POLICY.md](./NO_CUSTOM_JAVASCRIPT_POLICY.md) | No Custom JavaScript Policy | Universal Constraint | 2026-01-02 | ZERO custom JavaScript allowed - only framework-generated code |

---

## Index of ADRs

| Number | Title | Status | Date | Summary |
|--------|-------|--------|------|---------|
| [0001](./0001-tauri-blazor-desktop-client.md) | Tauri + Blazor WebAssembly Desktop Client | Accepted | 2026-01-02 | Decision to build desktop client using Tauri (Rust) + Blazor WASM (C#) with gRPC for IPC |
| [0002](./0002-thin-tauri-layer-principle.md) | Thin Tauri Layer Principle | Accepted | 2026-01-05 | Architectural principle: Tauri is ONLY for webview hosting, all logic lives in client-core |

---

## Universal Constraints (Must Read First)

### No Custom JavaScript Policy

**Status:** ABSOLUTE requirement for all frontend code

**The Rule:**
> ZERO custom JavaScript files. Only framework-generated JavaScript allowed.

**What this means:**
- ❌ No hand-written `.js` files
- ❌ No custom JavaScript helpers
- ❌ No manual DOM manipulation
- ✅ Only Blazor-generated JS (`_framework/blazor.webassembly.js`)
- ✅ Only framework compiler output (React, SolidJS, etc.)
- ✅ Only unmodified third-party libraries

**Why:** Eliminates entire classes of bugs, reduces maintenance burden, enforces type safety.

**Impact on decisions:** Any ADR requiring custom JavaScript is automatically **REJECTED**.

**See:** [NO_CUSTOM_JAVASCRIPT_POLICY.md](./NO_CUSTOM_JAVASCRIPT_POLICY.md) for full details.

---

## Active ADRs

### ADR-0001: Tauri + Blazor WebAssembly Desktop Client

**Why:** Need rich UI with zero custom JavaScript, while maintaining type safety and cross-platform support. Alternative to egui client.

**Decision:** Use Tauri (Rust webview host) + Blazor WASM (C# UI framework) + gRPC (type-safe IPC).

**Key points:**
- Coexists with egui client (users choose)
- Shared `client-core/` backend logic
- Zero custom JavaScript policy enforced (ABSOLUTE requirement)
- Detailed implementation plan (Phase 1-4, weeks 1-12)
- Validated by Cognexus reference project

**Status:** ✅ Accepted, implementation in progress (Sessions 1-6)

**Note:** Originally created in main OpenCode repository, migrated when Tauri-Blazor client was extracted into standalone project.

---

### ADR-0002: Thin Tauri Layer Principle

**Why:** Need clear separation of concerns between OS integration (Tauri) and application logic (client-core).

**Decision:** Tauri layer is ONLY for webview hosting. All application logic lives in client-core.

**Key points:**
- Enables testing without GUI
- Enables code reuse (CLI tools, alternative GUIs)
- Clear rule: "Is it webview hosting? No? → client-core"
- Tauri is thin glue code (~5 lines per feature)

**Status:** ✅ Foundational principle (enforced from Session 1 onward)

---

## Deprecated ADRs

None yet.

---

## Superseded ADRs

None yet.

---

## Contributing

When creating a new ADR:

1. **Copy the template:** Use [template.md](./template.md) as starting point
2. **Number it sequentially** (next available number: 0003)
3. **Be specific** about context, decision, and consequences
4. **Document alternatives** considered and why rejected
5. **Check universal constraints** (Zero Custom JavaScript, Thin Tauri Layer)
6. **Update this index** with summary and status
7. **Link related ADRs** in "Related ADRs" section

### When to Write an ADR

Write an ADR when:
- ✅ Making a significant architectural decision
- ✅ Choosing between multiple viable alternatives
- ✅ Decision will impact multiple components or sessions
- ✅ Decision is hard to reverse
- ✅ Future team members will ask "Why did we do it this way?"

Don't write an ADR for:
- ❌ Routine implementation details
- ❌ Decisions easily reversible
- ❌ Technology choices with clear best practices
- ❌ Code style preferences (use linter configs instead)

---

## Template

Use [template.md](./template.md) when creating new ADRs. The template includes:
- Standard structure (Context, Decision, Alternatives, Consequences)
- Universal constraints reminder (Zero Custom JavaScript, Thin Tauri Layer)
- Implementation notes section
- References section

## Resources

- **ADR Process:** https://adr.github.io/
- **Template (external):** https://github.com/joelparkerhenderson/architecture-decision-record
- **Examples:** https://github.com/adr/madr/tree/main/docs/decisions

---

## History

- **2026-01-02:** ADR-0001 created (Tauri + Blazor decision)
- **2026-01-05:** ADR-0002 created (Thin Tauri Layer principle)
