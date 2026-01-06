# Documentation Guide

**Quick reference:** Which document should I read?

---

## When Making Architecture Decisions

### 📋 Start Here: ADRs (Architecture Decision Records)

**Location:** `docs/adr/`

**Read these when:**
- ❓ "Why did we choose Tauri + Blazor?"
- ❓ "Why can't I write custom JavaScript?"
- ❓ "Why is all logic in client-core instead of Tauri?"
- ❓ "What were the alternatives?"

**Key Documents:**
- [ADR Index](./docs/adr/README.md) - All decisions
- [No Custom JavaScript Policy](./docs/adr/NO_CUSTOM_JAVASCRIPT_POLICY.md) - Universal constraint
- [ADR-0001: Tauri + Blazor](./docs/adr/0001-tauri-blazor-desktop-client.md) - Technology choice
- [ADR-0002: Thin Tauri Layer](./docs/adr/0002-thin-tauri-layer-principle.md) - Code organization

**Purpose:** Historical context, decision rationale, alternatives considered

---

## When Writing Code

### 💻 Start Here: ARCHITECTURE.md (Implementation Guide)

**Location:** `docs/ARCHITECTURE.md`

**Read this when:**
- ❓ "Where should this code go?"
- ❓ "Is this the right pattern?"
- ❓ "What's an example of good/bad code?"
- ❓ "What anti-patterns should I avoid?"

**Contents:**
- Layer responsibilities (Tauri, client-core, Blazor)
- Decision checklist (4 questions)
- Code examples (good vs bad)
- Anti-patterns to avoid
- Data flow diagrams

**Purpose:** Practical day-to-day coding guidance

---

## When Planning Implementation

### 📅 Start Here: SESSION_PLAN.md

**Location:** `SESSION_PLAN.md` (root)

**Read this when:**
- ❓ "What's been implemented so far?"
- ❓ "What's the plan for Session 4.5?"
- ❓ "How much work is left?"
- ❓ "What token budget do we have?"

**Contents:**
- 6 sessions (1-6) with detailed steps
- Completed work (Sessions 1-4)
- Pending work (Sessions 4.5-6)
- Token estimates per session
- Success criteria

**Purpose:** Project plan, progress tracking, session-by-session breakdown

---

## When Working with Data Models

### 📊 Start Here: Protobuf Documentation

**Location:** `docs/proto/`

**Read this when:**
- ❓ "What fields does ModelInfo have?"
- ❓ "How do I map JSON Schema to Protobuf?"
- ❓ "What's the structure of a Message?"
- ❓ "How were these schemas created?"

**Key Documents:**
- [Proto Index](./docs/proto/README.md) - Overview of all 9 proto domains
- [01-model.md](./docs/proto/01-model.md) through [09-opencode.md](./docs/proto/09-opencode.md) - Domain docs
- [Schema Development Process](./docs/proto/SCHEMA_DEVELOPMENT_PROCESS.md) - Workflow guide

**Purpose:** Data model reference, JSON Schema → Protobuf mapping

---

## Document Hierarchy

```
Decision Making                     Daily Coding
      ↓                                   ↓
docs/adr/                          docs/ARCHITECTURE.md
  ├── 0001-tauri-blazor.md              ├── Layer responsibilities
  ├── 0002-thin-tauri.md                ├── Decision checklist
  └── NO_CUSTOM_JS_POLICY.md            ├── Code examples
        ↓                                └── Anti-patterns
    "Why?"                                      ↓
                                            "How?"
                    ↓
              SESSION_PLAN.md
                    ↓
                "When?"
```

---

## Quick Reference Table

| Question | Document to Read |
|----------|------------------|
| Why did we choose this technology? | ADR-0001 |
| Why is there no custom JavaScript? | NO_CUSTOM_JAVASCRIPT_POLICY.md |
| Why is logic in client-core not Tauri? | ADR-0002 |
| Where should my code go? | ARCHITECTURE.md (Decision Checklist) |
| What's an example of good code? | ARCHITECTURE.md (Examples section) |
| What mistakes should I avoid? | ARCHITECTURE.md (Anti-Patterns) |
| What's the implementation plan? | SESSION_PLAN.md |
| What's been completed? | SESSION_PLAN.md (Sessions 1-4) |
| What data types exist? | docs/proto/README.md |
| How do I map JSON to Protobuf? | docs/proto/01-model.md (etc.) |
| How do I write a new ADR? | docs/adr/template.md |

---

## For New Team Members

**Read in this order:**

1. **README.md** (root) - Project overview
2. **docs/adr/NO_CUSTOM_JAVASCRIPT_POLICY.md** - ABSOLUTE constraint
3. **docs/adr/0001-tauri-blazor-desktop-client.md** - Technology choice (why)
4. **docs/adr/0002-thin-tauri-layer-principle.md** - Code organization (why)
5. **docs/ARCHITECTURE.md** - How to implement the principles
6. **SESSION_PLAN.md** - What's been done, what's next

Then refer to specific docs as needed:
- Writing code? → `docs/ARCHITECTURE.md`
- Data models? → `docs/proto/README.md`
- Making a decision? → `docs/adr/template.md`

---

## For Future You (6 Months from Now)

**"Why can't I just add a quick JavaScript helper?"**
→ Read: `docs/adr/NO_CUSTOM_JAVASCRIPT_POLICY.md`

**"Where should I put this HTTP client code?"**
→ Read: `docs/ARCHITECTURE.md` (Decision Checklist)

**"Why did we choose Tauri instead of Electron?"**
→ Read: `docs/adr/0001-tauri-blazor-desktop-client.md` (Alternatives section)

**"What's the pattern for gRPC services?"**
→ Read: `docs/ARCHITECTURE.md` (Examples section)

---

## Summary

| Document | Purpose | When to Use |
|----------|---------|-------------|
| **ADRs** | Record decisions | Making/reviewing decisions |
| **ARCHITECTURE.md** | Implementation guide | Writing code daily |
| **SESSION_PLAN.md** | Project plan | Planning/tracking work |
| **Proto docs** | Data model reference | Working with types |

**Key Principle:** ADRs explain **why**, ARCHITECTURE.md explains **how**.
